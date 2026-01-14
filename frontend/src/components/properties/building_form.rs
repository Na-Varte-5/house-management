use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BuildingFormProps {
    pub address: String,
    pub year: String,
    pub on_address_change: Callback<String>,
    pub on_year_change: Callback<String>,
    pub on_submit: Callback<()>,
    pub submitting: bool,
}

#[function_component(BuildingForm)]
pub fn building_form(props: &BuildingFormProps) -> Html {
    let on_submit = {
        let on_submit_cb = props.on_submit.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            on_submit_cb.emit(());
        })
    };

    html! {
        <form onsubmit={on_submit}>
            <h6 class="small fw-semibold mb-2">{"Create New Building"}</h6>
            <div class="mb-2">
                <input
                    type="text"
                    class="form-control form-control-sm"
                    placeholder="Address"
                    value={props.address.clone()}
                    disabled={props.submitting}
                    oninput={{
                        let on_change = props.on_address_change.clone();
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
                    class="form-control form-control-sm"
                    placeholder="Construction Year (optional)"
                    value={props.year.clone()}
                    disabled={props.submitting}
                    oninput={{
                        let on_change = props.on_year_change.clone();
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
                disabled={props.submitting || props.address.trim().is_empty()}
            >
                if props.submitting {
                    <span class="spinner-border spinner-border-sm me-1" role="status"></span>
                }
                {"Create Building"}
            </button>
        </form>
    }
}
