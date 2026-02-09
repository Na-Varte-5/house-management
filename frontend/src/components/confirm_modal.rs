use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ConfirmModalProps {
    pub show: bool,
    pub title: String,
    pub message: String,
    pub on_confirm: Callback<()>,
    pub on_cancel: Callback<()>,
    #[prop_or(String::from("Confirm"))]
    pub confirm_label: String,
    #[prop_or(String::from("Cancel"))]
    pub cancel_label: String,
    #[prop_or(String::from("danger"))]
    pub confirm_variant: String,
    #[prop_or(false)]
    pub loading: bool,
}

#[function_component(ConfirmModal)]
pub fn confirm_modal(props: &ConfirmModalProps) -> Html {
    if !props.show {
        return html! {};
    }

    let on_confirm = {
        let cb = props.on_confirm.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_cancel = {
        let cb = props.on_cancel.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let on_backdrop = {
        let cb = props.on_cancel.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    let stop_propagation = Callback::from(|e: MouseEvent| e.stop_propagation());

    let confirm_class = format!("btn btn-{}", props.confirm_variant);

    html! {
        <div class="modal d-block" tabindex="-1" style="background: rgba(0,0,0,0.5);"
             onclick={on_backdrop}>
            <div class="modal-dialog modal-dialog-centered" onclick={stop_propagation}>
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">{&props.title}</h5>
                        <button type="button" class="btn-close" onclick={on_cancel.clone()}
                                disabled={props.loading}></button>
                    </div>
                    <div class="modal-body">
                        <p>{&props.message}</p>
                    </div>
                    <div class="modal-footer">
                        <button type="button" class="btn btn-secondary"
                                onclick={on_cancel} disabled={props.loading}>
                            {&props.cancel_label}
                        </button>
                        <button type="button" class={confirm_class}
                                onclick={on_confirm} disabled={props.loading}>
                            if props.loading {
                                <span class="spinner-border spinner-border-sm me-1"></span>
                            }
                            {&props.confirm_label}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
