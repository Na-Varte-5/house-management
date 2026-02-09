use yew::prelude::*;

use crate::contexts::LanguageContext;

fn language_label(code: &str) -> &'static str {
    match code {
        "en" => "\u{1f1ec}\u{1f1e7} EN",
        "cs" => "\u{1f1e8}\u{1f1ff} CS",
        _ => "??",
    }
}

#[derive(Properties, PartialEq)]
pub struct LanguageSwitcherProps {
    #[prop_or("form-select form-select-sm bg-dark text-light border-secondary".to_string())]
    pub class: String,
}

#[function_component(LanguageSwitcher)]
pub fn language_switcher(props: &LanguageSwitcherProps) -> Html {
    let lang_ctx = use_context::<LanguageContext>().expect("LanguageContext not found");

    let on_change = {
        let set_language = lang_ctx.set_language.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            set_language.emit(select.value());
        })
    };

    html! {
        <select class={props.class.clone()} onchange={on_change}>
            { for lang_ctx.languages.iter().map(|code| {
                html! {
                    <option value={code.clone()} selected={*code == lang_ctx.language}>
                        { language_label(code) }
                    </option>
                }
            })}
        </select>
    }
}
