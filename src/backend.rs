use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use ashpd::desktop::inhibit::{InhibitFlags, InhibitProxy};
use ashpd::desktop::Request;
use ashpd::enumflags2::make_bitflags;
use zbus::zvariant::OwnedFd;

use crate::config::InhibitMode;

/// Idle is inhibited through the XDG portal. Suspend goes to logind directly,
/// because portal backends such as the GTK one (used on COSMIC) only support
/// idle inhibition.
#[zbus::proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Login1Manager {
    fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> zbus::Result<OwnedFd>;
}

#[derive(Clone, Default)]
pub struct CaffeineBackend {
    state: Arc<Mutex<BackendState>>,
}

#[derive(Default)]
struct BackendState {
    idle: Option<Request<()>>,
    /// logind releases the lock as soon as this descriptor is closed.
    sleep: Option<OwnedFd>,
}

impl CaffeineBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces any active inhibition. Either every lock required by `mode`
    /// is taken, or none is.
    pub async fn inhibit(&self, reason: &str, mode: InhibitMode) -> Result<(), String> {
        let mut state = self.state.lock().await;
        Self::release(&mut state).await;

        let sleep = if matches!(mode, InhibitMode::Suspend | InhibitMode::Both) {
            Some(Self::inhibit_sleep(reason).await?)
        } else {
            None
        };
        let idle = if matches!(mode, InhibitMode::Idle | InhibitMode::Both) {
            Some(Self::inhibit_idle(reason).await?)
        } else {
            None
        };

        *state = BackendState { idle, sleep };
        info!("Inhibition active, mode: {:?}", mode);
        Ok(())
    }

    pub async fn uninhibit(&self) {
        Self::release(&mut *self.state.lock().await).await;
    }

    async fn release(state: &mut BackendState) {
        state.sleep = None;
        if let Some(request) = state.idle.take() {
            if let Err(e) = request.close().await {
                warn!("Failed to close portal request: {}", e);
            }
        }
    }

    async fn inhibit_idle(reason: &str) -> Result<Request<()>, String> {
        let error = |e: ashpd::Error| format!("Failed to inhibit idle: {e}");
        let proxy = InhibitProxy::new().await.map_err(error)?;
        let request = proxy
            .inhibit(None, make_bitflags!(InhibitFlags::{Idle}), reason)
            .await
            .map_err(error)?;
        request.response().map_err(error)?;
        Ok(request)
    }

    async fn inhibit_sleep(reason: &str) -> Result<OwnedFd, String> {
        let error = |e: zbus::Error| format!("Failed to inhibit suspend: {e}");
        let connection = zbus::Connection::system().await.map_err(error)?;
        let proxy = Login1ManagerProxy::new(&connection).await.map_err(error)?;
        proxy
            .inhibit("sleep", "Caffeine", reason, "block")
            .await
            .map_err(error)
    }
}
