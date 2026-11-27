use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader, Localizer,
};
use rust_embed::RustEmbed;
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    loader
        .load_fallback_language(&Localizations)
        .expect("Error loading fallback language");
    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args),*)
    }};
}

/// Initialize the localization system
pub fn init() {
    let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations);
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    if let Err(e) = localizer.select(&requested_languages) {
        tracing::warn!("Failed to select language: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn german_locale_formats_timer_labels_and_notification_plurals() {
        let loader = fluent_language_loader!();
        let localizer = DefaultLocalizer::new(&loader, &Localizations);
        localizer.select(&["de-DE".parse().unwrap()]).unwrap();

        assert_eq!(loader.get("timer-manual"), "Eigene Dauer");
        assert_eq!(loader.get("timer-hours-label"), "Stunden");
        assert_eq!(
            loader.get_args("notification-warning-body", HashMap::from([("mins", 1)])),
            "Noch eine Minute."
        );
        assert_eq!(
            loader
                .get_args("notification-warning-body", HashMap::from([("mins", 2)]))
                .replace(['\u{2068}', '\u{2069}'], ""),
            "Noch 2 Minuten."
        );
    }
}
