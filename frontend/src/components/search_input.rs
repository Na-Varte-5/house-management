use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchInputProps {
    pub value: String,
    pub on_change: Callback<String>,
    #[prop_or(String::from("Search..."))]
    pub placeholder: String,
}

#[function_component(SearchInput)]
pub fn search_input(props: &SearchInputProps) -> Html {
    let on_input = {
        let cb = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            cb.emit(input.value());
        })
    };

    let on_clear = {
        let cb = props.on_change.clone();
        Callback::from(move |_: MouseEvent| cb.emit(String::new()))
    };

    html! {
        <div class="input-group input-group-sm" style="max-width: 300px;">
            <span class="input-group-text bg-white">
                <i class="bi bi-search text-muted"></i>
            </span>
            <input
                type="text"
                class="form-control border-start-0"
                placeholder={props.placeholder.clone()}
                value={props.value.clone()}
                oninput={on_input}
            />
            if !props.value.is_empty() {
                <button class="btn btn-outline-secondary" type="button" onclick={on_clear}>
                    <i class="bi bi-x"></i>
                </button>
            }
        </div>
    }
}
