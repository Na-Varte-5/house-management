use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorAlertProps {
    pub message: String,
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,
}

#[function_component(ErrorAlert)]
pub fn error_alert(props: &ErrorAlertProps) -> Html {
    let on_close_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            if let Some(callback) = &on_close {
                callback.emit(());
            }
        })
    };

    html! {
        <div class="alert alert-danger alert-dismissible fade show" role="alert">
            <strong>{"Error: "}</strong> {&props.message}
            if props.on_close.is_some() {
                <button type="button" class="btn-close" onclick={on_close_click} aria-label="Close"></button>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SuccessAlertProps {
    pub message: String,
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,
}

#[function_component(SuccessAlert)]
pub fn success_alert(props: &SuccessAlertProps) -> Html {
    let on_close_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            if let Some(callback) = &on_close {
                callback.emit(());
            }
        })
    };

    html! {
        <div class="alert alert-success alert-dismissible fade show" role="alert">
            {&props.message}
            if props.on_close.is_some() {
                <button type="button" class="btn-close" onclick={on_close_click} aria-label="Close"></button>
            }
        </div>
    }
}
