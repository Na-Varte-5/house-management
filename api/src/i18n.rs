use fluent::{FluentBundle, FluentResource};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::cell::RefCell;
use unic_langid::LanguageIdentifier;

// Define supported languages
pub const DEFAULT_LANG: &str = "en";
pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "cs"];

// Parse language identifiers
static LANGUAGE_IDS: Lazy<Vec<LanguageIdentifier>> = Lazy::new(|| {
    SUPPORTED_LANGUAGES
        .iter()
        .filter_map(|lang| lang.parse().ok())
        .collect()
});

// Store bundles for each language
type FluentBundles = HashMap<String, FluentBundle<FluentResource>>;

// Thread-local storage for bundles
thread_local! {
    static BUNDLES: RefCell<FluentBundles> = RefCell::new(HashMap::new());
}

// Initialize translations
pub fn init_translations() {
    BUNDLES.with(|bundles_cell| {
        let mut bundles = bundles_cell.borrow_mut();

        for lang in SUPPORTED_LANGUAGES {
            let mut bundle = FluentBundle::new(vec![lang.parse().unwrap()]);

            // Load common translations
            let path = format!("locales/{}/common.ftl", lang);
            if let Ok(source) = read_file(&path) {
                let resource = FluentResource::try_new(source)
                    .expect(&format!("Failed to parse {}", path));
                bundle.add_resource(resource)
                    .expect(&format!("Failed to add resource for {}", path));
            }

            // Add more domain-specific translations here if needed

            bundles.insert(lang.to_string(), bundle);
        }
    });
}

// Read file content
fn read_file(path: &str) -> std::io::Result<String> {
    let full_path = Path::new("api").join(path);
    fs::read_to_string(full_path)
}

// Negotiate the best language based on Accept-Language header
pub fn negotiate_language(accept_language: Option<&str>) -> String {
    if let Some(accept_lang) = accept_language {
        let requested: Vec<LanguageIdentifier> = accept_lang
            .split(',')
            .filter_map(|item| {
                let parts: Vec<&str> = item.split(';').collect();
                parts[0].trim().parse().ok()
            })
            .collect();

        if !requested.is_empty() {
            let supported = LANGUAGE_IDS.as_slice();
            let default: LanguageIdentifier = DEFAULT_LANG.parse().unwrap();

            let result = negotiate_languages(
                &requested,
                supported,
                Some(&default),
                NegotiationStrategy::Filtering,
            );

            if let Some(best) = result.first() {
                return best.to_string();
            }
        }
    }

    DEFAULT_LANG.to_string()
}

// Get translation for a message ID
pub fn get_message(lang: &str, message_id: &str) -> String {
    BUNDLES.with(|bundles_cell| {
        let bundles = bundles_cell.borrow();

        // Try to get the bundle for the requested language
        let bundle = if let Some(bundle) = bundles.get(lang) {
            bundle
        } else {
            // Fallback to default language
            match bundles.get(DEFAULT_LANG) {
                Some(default_bundle) => default_bundle,
                None => {
                    // If no bundles are initialized, return the message ID
                    return message_id.to_string();
                }
            }
        };

        // Try to get the message
        if let Some(message) = bundle.get_message(message_id) {
            if let Some(pattern) = message.value() {
                let mut errors = vec![];
                let result = bundle.format_pattern(pattern, None, &mut errors);
                if errors.is_empty() {
                    return result.to_string();
                }
            }
        }

        // Return message ID if translation not found
        message_id.to_string()
    })
}
