use yew::prelude::*;
use crate::i18n::{get_current_language, set_language, is_language_supported};
use crate::fl;

// Language switcher component
#[function_component(LanguageSwitcher)]
fn language_switcher() -> Html {
    let current_lang = get_current_language();

    let on_language_change = Callback::from(move |e: Event| {
        let target = e.target_dyn_into::<web_sys::HtmlSelectElement>();
        if let Some(select) = target {
            let value = select.value();
            if is_language_supported(&value) {
                set_language(&value);
                // Force a re-render by reloading the page
                if let Some(window) = web_sys::window() {
                    let _ = window.location().reload();
                }
            }
        }
    });

    html! {
        <div class="language-switcher">
            <label for="language-select">{ fl!("ui-language") }</label>
            <select id="language-select" value={current_lang} onchange={on_language_change}>
                <option value="en">{ fl!("ui-language-en") }</option>
                <option value="cs">{ fl!("ui-language-cs") }</option>
            </select>
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <div class="header">
                <img class="logo" src="https://yew.rs/img/logo.svg" alt="Yew logo" />
                <LanguageSwitcher />
            </div>
            <h1>{ fl!("welcome") }</h1>
            <span class="subtitle">{ fl!("app-name") }</span>

            <div class="content">
                <h2>{ fl!("nav-dashboard") }</h2>
                <p>{ "This is a demo of the internationalization feature." }</p>

                <div class="buttons">
                    <button>{ fl!("button-save") }</button>
                    <button>{ fl!("button-cancel") }</button>
                </div>
            </div>
        </main>
    }
}
