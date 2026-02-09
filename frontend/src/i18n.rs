use fluent_bundle::{FluentBundle, FluentResource};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;
use web_sys::window;

static _CURRENT_LANG: Lazy<LanguageIdentifier> = Lazy::new(|| "en".parse().unwrap());
thread_local! {
    static BUNDLES: HashMap<String, FluentBundle<FluentResource>> = {
        let mut map = HashMap::new();
        for (lang, source) in [
            ("en", include_str!("../locales/en/frontend.ftl")),
            ("cs", include_str!("../locales/cs/frontend.ftl")),
        ] {
            let res = FluentResource::try_new(source.to_string()).expect("Valid FTL");
            let langid: LanguageIdentifier = lang.parse().unwrap();
            let mut bundle = FluentBundle::new(vec![langid]);
            if let Err(_e) = bundle.add_resource(res) { /* ignore duplicate or error */ }
            map.insert(lang.to_string(), bundle);
        }
        map
    };
    static CURRENT_LANG_CODE: RefCell<String> = RefCell::new("en".to_string());
}

pub fn t(key: &str) -> String {
    BUNDLES.with(|bundles| {
        let lang = CURRENT_LANG_CODE.with(|l| l.borrow().clone());
        if let Some(bundle) = bundles.get(&lang) {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    let value = bundle.format_pattern(pattern, None, &mut errors);
                    return value.to_string();
                }
            }
        }
        if lang != "en" {
            if let Some(bundle) = bundles.get("en") {
                if let Some(msg) = bundle.get_message(key) {
                    if let Some(pattern) = msg.value() {
                        let mut errors = vec![];
                        let value = bundle.format_pattern(pattern, None, &mut errors);
                        return value.to_string();
                    }
                }
            }
        }
        key.to_string()
    })
}

pub fn t_with_args(key: &str, args: &[(&str, &str)]) -> String {
    use fluent_bundle::FluentArgs;
    BUNDLES.with(|bundles| {
        let lang = CURRENT_LANG_CODE.with(|l| l.borrow().clone());
        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, fluent_bundle::FluentValue::from(*v));
        }
        if let Some(bundle) = bundles.get(&lang) {
            if let Some(msg) = bundle.get_message(key) {
                if let Some(pattern) = msg.value() {
                    let mut errors = vec![];
                    let value = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
                    return value.to_string();
                }
            }
        }
        if lang != "en" {
            if let Some(bundle) = bundles.get("en") {
                if let Some(msg) = bundle.get_message(key) {
                    if let Some(pattern) = msg.value() {
                        let mut errors = vec![];
                        let value = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
                        return value.to_string();
                    }
                }
            }
        }
        key.to_string()
    })
}

pub fn init_translations() {
    BUNDLES.with(|_| {
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(lang)) = storage.get_item("app.lang") {
                    set_language(&lang);
                }
            }
        }
    });
}

pub fn set_language(lang: &str) {
    BUNDLES.with(|bundles| {
        if bundles.contains_key(lang) {
            CURRENT_LANG_CODE.with(|c| *c.borrow_mut() = lang.to_string());
            if let Some(win) = window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    let _ = storage.set_item("app.lang", lang);
                }
            }
        }
    });
}

pub fn current_language() -> String {
    CURRENT_LANG_CODE.with(|c| c.borrow().clone())
}
pub fn available_languages() -> Vec<String> {
    BUNDLES.with(|b| b.keys().cloned().collect())
}
