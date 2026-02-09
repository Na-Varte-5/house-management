use yew::prelude::*;

use crate::i18n::{available_languages, current_language, set_language as i18n_set_language};

/// Context that provides reactive language state to the component tree.
///
/// When the language changes, all components consuming this context will re-render,
/// causing `t()` / `t_with_args()` calls to return updated translations.
#[derive(Clone, PartialEq)]
pub struct LanguageContext {
    /// Current language code (e.g., "en", "cs")
    pub language: String,
    /// Available language codes
    pub languages: Vec<String>,
    /// Callback to change the active language
    pub set_language: Callback<String>,
}

#[derive(Properties, PartialEq)]
pub struct LanguageProviderProps {
    #[prop_or_default]
    pub children: Html,
}

/// Provider component that wraps the app tree and makes language switching reactive.
///
/// Place this high in the component tree (e.g., in `App`) so that language changes
/// trigger re-renders of all descendant components that use `t()`.
#[function_component(LanguageProvider)]
pub fn language_provider(props: &LanguageProviderProps) -> Html {
    let language = use_state(|| current_language());

    let set_lang = {
        let language = language.clone();
        Callback::from(move |lang: String| {
            i18n_set_language(&lang);
            language.set(lang);
        })
    };

    let context = LanguageContext {
        language: (*language).clone(),
        languages: available_languages(),
        set_language: set_lang,
    };

    html! {
        <ContextProvider<LanguageContext> context={context}>
            {props.children.clone()}
        </ContextProvider<LanguageContext>>
    }
}
