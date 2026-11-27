use crate::backend::CaffeineBackend;
use crate::config::CaffeineConfig;
use crate::notify;
use crate::state::{CaffeineState, TimerSelection};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;
use tracing::{error, info};
use zbus::{interface, object_server::SignalEmitter, proxy};

pub const DBUS_NAME: &str = "com.github.oussama_berchi.cosmic_caffeine";
pub const DBUS_PATH: &str = "/com/github/oussama_berchi/cosmic_caffeine";
pub const DBUS_INTERFACE: &str = "com.github.oussama_berchi.cosmic_caffeine.Manager";

#[derive(Clone)]
pub struct CaffeineService {
    backend: CaffeineBackend,
    state: Arc<Mutex<(CaffeineState, u64)>>,
    /// Only the service runs the timer, so each notification is shown once
    /// no matter how many panels show the applet.
    timer: Option<Arc<AbortOnDrop>>,
}

struct AbortOnDrop(JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl CaffeineService {
    pub fn new() -> Self {
        Self {
            backend: CaffeineBackend::new(),
            state: Arc::new(Mutex::new((CaffeineState::inactive(), 0))),
            timer: None,
        }
    }
}

fn idx_to_selection(idx: u32) -> TimerSelection {
    match idx {
        0 => TimerSelection::Infinity,
        1 => TimerSelection::FiveMins,
        2 => TimerSelection::TenMins,
        3 => TimerSelection::ThirtyMins,
        4 => TimerSelection::OneHour,
        5 => TimerSelection::TwoHours,
        6 => TimerSelection::ThreeHours,
        7 => TimerSelection::FourHours,
        _ => TimerSelection::Manual,
    }
}

#[proxy(
    interface = "com.github.oussama_berchi.cosmic_caffeine.Manager",
    default_service = "com.github.oussama_berchi.cosmic_caffeine",
    default_path = "/com/github/oussama_berchi/cosmic_caffeine"
)]
pub trait CaffeineManager {
    async fn set_state(
        &self,
        active: bool,
        selection_idx: u32,
        manual_mins: u32,
    ) -> zbus::Result<()>;

    async fn get_state(&self) -> zbus::Result<CaffeineState>;

    async fn get_timer_state(&self) -> zbus::Result<(CaffeineState, u64)>;
}

#[interface(name = "com.github.oussama_berchi.cosmic_caffeine.Manager")]
impl CaffeineService {
    async fn set_state(
        &mut self,
        active: bool,
        selection_idx: u32,
        manual_mins: u32,
        #[zbus(signal_emitter)] ctxt: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        info!(
            "D-Bus Request: SetState(active={}, idx={}, mins={})",
            active, selection_idx, manual_mins
        );

        let config = CaffeineConfig::load();
        self.timer = None;

        let (new_state, total) = if active {
            let selection = idx_to_selection(selection_idx);

            let manual_hours = if selection == TimerSelection::Manual {
                config.manual_hours
            } else {
                0
            };
            let duration =
                selection.duration_secs(Some(manual_mins as u64), Some(manual_hours as u64));

            let reason = format!("Caffeine enabled: {}", selection.label());

            let inhibit_mode = config.inhibit_mode;
            if let Err(e) = self.backend.inhibit(&reason, inhibit_mode).await {
                error!("Failed to inhibit via D-Bus: {}", e);
                notify::notify_error(&e);
                // Any earlier lock has been released, so report inactive.
                self.publish(CaffeineState::inactive(), 0, &ctxt).await;
                return Err(zbus::fdo::Error::Failed(e));
            }

            let expiry = duration.map(|total| SystemTime::now() + Duration::from_secs(total));
            let expiry_ts =
                expiry.map(|at| at.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());

            if let (Some(total), Some(expiry)) = (duration, expiry) {
                notify::notify_enabled_with_time(total / 60);
                self.start_timer(total, expiry, config.warning_mins, ctxt.to_owned());
            } else {
                notify::notify_enabled();
            }

            (
                CaffeineState::active(selection, expiry_ts),
                duration.unwrap_or(0),
            )
        } else {
            self.backend.uninhibit().await;
            notify::notify_disabled();
            (CaffeineState::inactive(), 0)
        };

        self.publish(new_state, total, &ctxt).await;
        Ok(())
    }

    async fn get_state(&self) -> CaffeineState {
        self.get_timer_state().await.0
    }

    async fn get_timer_state(&self) -> (CaffeineState, u64) {
        if let Ok(lock) = self.state.lock() {
            *lock
        } else {
            error!("Failed to acquire lock on state");
            (CaffeineState::inactive(), 0)
        }
    }
}

impl CaffeineService {
    async fn publish(&self, new_state: CaffeineState, total: u64, ctxt: &SignalEmitter<'_>) {
        publish_state(&self.state, new_state, total, ctxt).await;
    }

    fn start_timer(
        &mut self,
        total: u64,
        expiry: SystemTime,
        warning_mins: u32,
        ctxt: SignalEmitter<'static>,
    ) {
        let backend = self.backend.clone();
        let state = self.state.clone();
        let threshold = u64::from(warning_mins) * 60;
        // No warning if the whole timer is shorter than the warning period.
        let mut warned = threshold == 0 || total <= threshold;

        let handle = tokio::spawn(async move {
            // Compare against the wall clock: tokio's clock stops during suspend.
            while let Ok(left) = expiry.duration_since(SystemTime::now()) {
                if !warned && left.as_secs() <= threshold {
                    warned = true;
                    notify::notify_warning(warning_mins);
                }
                tokio::time::sleep(left.min(Duration::from_secs(1))).await;
            }
            info!("Timer expired, disabling caffeine");
            backend.uninhibit().await;
            notify::notify_timer_expired();
            publish_state(&state, CaffeineState::inactive(), 0, &ctxt).await;
        });
        self.timer = Some(Arc::new(AbortOnDrop(handle)));
    }
}

async fn publish_state(
    state: &Mutex<(CaffeineState, u64)>,
    new_state: CaffeineState,
    total: u64,
    ctxt: &SignalEmitter<'_>,
) {
    match state.lock() {
        Ok(mut lock) => *lock = (new_state, total),
        Err(_) => error!("Failed to acquire lock on state"),
    }
    if let Err(e) = ctxt.emit(DBUS_INTERFACE, "StateChanged", &new_state).await {
        error!("Failed to emit signal: {}", e);
    }
}
