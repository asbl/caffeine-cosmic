use notify_rust::{Notification, Urgency};
use std::thread;
use tracing::{debug, warn};

use crate::config::CaffeineConfig;
use crate::fl;

const APP_ID: &str = "com.github.cosmic-caffeine";

fn show_notification(title: &str, body: &str, urgency: Urgency, timeout: i32) {
    if !CaffeineConfig::load().show_notifications {
        return;
    }

    let title = title.to_string();
    let body = body.to_string();

    thread::spawn(move || {
        if let Err(e) = Notification::new()
            .appname("Caffeine")
            .summary(&title)
            .body(&body)
            .icon("oussama-berchi-caffeine-cosmic")
            .hint(notify_rust::Hint::DesktopEntry(APP_ID.to_string()))
            .urgency(urgency)
            .timeout(timeout)
            .show()
        {
            warn!("Failed to show notification: {}", e);
        } else {
            debug!("Showed notification: {}", title);
        }
    });
}

pub fn notify_enabled() {
    let title = fl!("notification-enabled-title");
    let body = fl!("notification-enabled-body");
    show_notification(&title, &body, Urgency::Low, 3000);
}

pub fn notify_enabled_with_time(mins: u64) {
    let title = fl!("notification-enabled-title");
    let body = fl!("notification-enabled-timed-body", mins = (mins as f64));
    show_notification(&title, &body, Urgency::Low, 3000);
}

pub fn notify_disabled() {
    let title = fl!("notification-disabled-title");
    let body = fl!("notification-disabled-body");
    show_notification(&title, &body, Urgency::Low, 3000);
}

pub fn notify_timer_expired() {
    let title = fl!("notification-timer-expired-title");
    let body = fl!("notification-timer-expired-body");
    show_notification(&title, &body, Urgency::Normal, 5000);
}

pub fn notify_warning(remaining_mins: u32) {
    let title = fl!("notification-warning-title");
    let body = fl!("notification-warning-body", mins = remaining_mins);
    show_notification(&title, &body, Urgency::Normal, 5000);
}

pub fn notify_error(details: &str) {
    let title = fl!("notification-error-title");
    let body = format!("{} {}", fl!("notification-error-body"), details);
    show_notification(&title, &body, Urgency::Normal, 5000);
}
