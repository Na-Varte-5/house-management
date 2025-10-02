use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    LanguageLoader,
};
use js_sys;
use rust_embed::RustEmbed;
use std::sync::OnceLock;
use unic_langid::LanguageIdentifier;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDocument, HtmlHtmlElement};

// Define supported languages
pub const DEFAULT_LANG: &str = "en";
pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "cs"];

// Embed localization resources
#[derive(RustEmbed)]
#[folder = "locales/"]
struct Localizations;

// Create a static language loader
static LANGUAGE_LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();

// Get the language loader
pub fn language_loader() -> &'static FluentLanguageLoader {
    LANGUAGE_LOADER.get_or_init(|| {
        let loader = fluent_language_loader!();

        // Log available files in the embedded resources
        let files: Vec<String> = Localizations::iter().map(|s| s.to_string()).collect();
        web_sys::console::log_1(&format!("Available files: {:?}", files).into());

        // Try to manually initialize the loader with the default language
        let default_lang: LanguageIdentifier = DEFAULT_LANG.parse().unwrap();

        // Create a dummy loader that doesn't panic on errors
        // This ensures the app can still run even if translations fail to load
        if files.is_empty() {
            web_sys::console::error_1(&"No localization files found".into());
        } else {
            // Try to load the default language
            let localizations = Localizations {};
            if let Err(err) = loader.load_languages(&localizations, &[&default_lang]) {
                web_sys::console::error_1(&format!("Failed to load default language: {:?}", err).into());
            } else {
                web_sys::console::log_1(&"Successfully loaded default language".into());
            }
        }

        loader
    })
}

// Initialize translations based on browser language
pub fn init_translations() {
    // Get the language loader (this initializes it with the default language)
    let loader = language_loader();
    web_sys::console::log_1(&format!("Current language: {}", loader.current_language()).into());

    // Try to get the browser language
    if let Some(window) = window() {
        let navigator = window.navigator();

        // Log the navigator languages for debugging
        web_sys::console::log_1(&format!("Navigator: {:?}", navigator).into());

        // Try to get the languages from the navigator
        match js_sys::try_iter(&navigator.languages()) {
            Ok(Some(languages_iter)) => {
                // Convert JsValue to strings
                let languages: Vec<String> = languages_iter
                    .filter_map(|item| item.ok())
                    .filter_map(|item| item.as_string())
                    .collect();

                web_sys::console::log_1(&format!("Browser languages: {:?}", languages).into());

                if !languages.is_empty() {
                    // Find the first supported language
                    for lang in languages {
                        if SUPPORTED_LANGUAGES.contains(&lang.as_str()) {
                            web_sys::console::log_1(&format!("Found supported language: {}", lang).into());

                            // Try to set the language
                            if set_language(&lang) {
                                web_sys::console::log_1(&format!("Successfully set language to {}", lang).into());
                                break;
                            }
                        }
                    }
                }
            },
            Ok(None) => {
                web_sys::console::log_1(&"No languages found in navigator".into());
            },
            Err(err) => {
                web_sys::console::error_1(&format!("Error getting languages from navigator: {:?}", err).into());
            }
        }
    } else {
        web_sys::console::error_1(&"No window object found".into());
    }
}

// Set the language
pub fn set_language(lang_code: &str) -> bool {
    // Check if the language is supported
    if !SUPPORTED_LANGUAGES.contains(&lang_code) {
        web_sys::console::error_1(&format!("Language {} is not supported", lang_code).into());
        return false;
    }

    let loader = language_loader();
    web_sys::console::log_1(&format!("Setting language to {}", lang_code).into());

    // Parse the language code
    match lang_code.parse::<LanguageIdentifier>() {
        Ok(lang_id) => {
            // Try to load the language
            let localizations = Localizations {};

            // Log available files for debugging
            let files: Vec<String> = Localizations::iter().map(|s| s.to_string()).collect();
            web_sys::console::log_1(&format!("Available files: {:?}", files).into());

            // Check if the language files exist
            let lang_path = format!("{}/", lang_code);
            let has_files = files.iter().any(|f| f.starts_with(&lang_path));

            if !has_files {
                web_sys::console::error_1(&format!("No files found for language {}", lang_code).into());
                return false;
            }

            // Try to load the language
            match loader.load_languages(&localizations, &[&lang_id]) {
                Ok(_) => {
                    web_sys::console::log_1(&format!("Successfully loaded language: {}", lang_code).into());

                    // Update the html lang attribute
                    if let Some(window) = window() {
                        if let Some(document) = window.document() {
                            if let Ok(html_doc) = document.dyn_into::<HtmlDocument>() {
                                if let Some(html_element) = html_doc.document_element() {
                                    if let Ok(html) = html_element.dyn_into::<HtmlHtmlElement>() {
                                        html.set_lang(lang_code);
                                        web_sys::console::log_1(&format!("Set HTML lang attribute to {}", lang_code).into());
                                    }
                                }
                            }
                        }
                    }
                    return true;
                },
                Err(err) => {
                    web_sys::console::error_1(&format!("Failed to load language {}: {:?}", lang_code, err).into());
                    return false;
                }
            }
        },
        Err(err) => {
            web_sys::console::error_1(&format!("Failed to parse language code {}: {:?}", lang_code, err).into());
            return false;
        }
    }
}

// Get the current language
pub fn get_current_language() -> String {
    let loader = language_loader();
    let current_lang = loader.current_language().to_string();
    web_sys::console::log_1(&format!("Current language: {}", current_lang).into());
    current_lang
}

// Check if a language is supported
pub fn is_language_supported(lang_code: &str) -> bool {
    let is_supported = SUPPORTED_LANGUAGES.contains(&lang_code);
    web_sys::console::log_1(&format!("Checking if language {} is supported: {}", lang_code, is_supported).into());
    is_supported
}

// Macro for getting translations
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::language_loader(), $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::language_loader(), $message_id, $($args),*)
    }};
}
