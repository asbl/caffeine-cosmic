mod app;
mod backend;
mod config;
mod i18n;
mod notify;
mod service;
mod state;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    i18n::init();

    if !is_cosmic_session() {
        eprintln!("{}", fl!("env-warning"));
    }

    cosmic::applet::run::<app::AppModel>(())
}

fn is_cosmic_session() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|v| v.to_uppercase().contains("COSMIC"))
        .unwrap_or(false)
        || std::env::var("XDG_SESSION_DESKTOP")
            .map(|v| v.to_uppercase().contains("COSMIC"))
            .unwrap_or(false)
}
