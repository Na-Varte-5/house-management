use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ApartmentFormProps {
    pub number: String,
    pub size: String,
    pub on_number_change: Callback<String>,
    pub on_size_change: Callback<String>,
    pub on_submit: Callback<()>,
    pub submitting: bool,
    pub show: bool,
}

#[function_component(ApartmentForm)]
pub fn apartment_form(props: &ApartmentFormProps) -> Html {
    if !props.show {
        return html! {
            <div class="alert alert-info small mb-0">
                <i class="bi bi-info-circle me-2"></i>
                {"Select a building to create apartments"}
            </div>
        };
    }

    let on_submit = {
        let on_submit_cb = props.on_submit.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            on_submit_cb.emit(());
        })
    };

    html! {
        <form onsubmit={on_submit}>
            <h6 class="small fw-semibold mb-2">{"Create New Apartment"}</h6>
            <div class="mb-2">
                <input
                    type="text"
                    class="form-control form-control-sm"
                    placeholder="Apartment Number"
                    value={props.number.clone()}
                    disabled={props.submitting}
                    oninput={{
                        let on_change = props.on_number_change.clone();
                        Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            on_change.emit(input.value());
                        })
                    }}
                />
            </div>
            <div class="mb-2">
                <input
                    type="number"
                    step="0.1"
                    class="form-control form-control-sm"
                    placeholder="Size (m²) - optional"
                    value={props.size.clone()}
                    disabled={props.submitting}
                    oninput={{
                        let on_change = props.on_size_change.clone();
                        Callback::from(move |e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            on_change.emit(input.value());
                        })
                    }}
                />
            </div>
            <button
                type="submit"
                class="btn btn-primary btn-sm w-100"
                disabled={props.submitting || props.number.trim().is_empty()}
            >
                if props.submitting {
                    <span class="spinner-border spinner-border-sm me-1" role="status"></span>
                }
                {"Create Apartment"}
            </button>
        </form>
    }
}
