use yew::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Properties, PartialEq)]
pub struct CheckboxProps {
    /// Whether the checkbox is checked
    pub checked: bool,

    /// Callback when checked state changes
    pub on_change: Callback<bool>,

    /// Label text displayed next to checkbox
    pub label: String,

    /// Optional help text shown below checkbox
    #[prop_or_default]
    pub help_text: Option<String>,

    /// Whether the field is disabled
    #[prop_or_default]
    pub disabled: bool,

    /// Validation error message
    #[prop_or_default]
    pub error: Option<String>,

    /// Additional CSS classes for the checkbox container
    #[prop_or_default]
    pub class: String,

    /// Use switch style instead of checkbox
    #[prop_or(false)]
    pub switch: bool,

    /// Display inline (for multiple checkboxes in a row)
    #[prop_or(false)]
    pub inline: bool,

    /// Unique ID for the checkbox (required for label association)
    pub id: String,
}

/// Reusable checkbox component with label, validation, and switch variant
///
/// # Example
/// ```rust
/// html! {
///     <Checkbox
///         id="checkbox-homeowner"
///         label="Homeowner"
///         checked={*role_homeowner}
///         on_change={on_homeowner_change}
///         help_text="Allow homeowners to vote"
///     />
///
///     <Checkbox
///         id="checkbox-notifications"
///         label="Enable notifications"
///         checked={*notifications}
///         on_change={on_notifications_change}
///         switch=true
///     />
/// }
/// ```
#[function_component(Checkbox)]
pub fn checkbox(props: &CheckboxProps) -> Html {
    let onchange = {
        let on_change = props.on_change.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            on_change.emit(input.checked());
        })
    };

    let container_class = {
        let mut classes = vec!["form-check"];

        if props.switch {
            classes.push("form-switch");
        }

        if props.inline {
            classes.push("form-check-inline");
        }

        if !props.class.is_empty() {
            classes.push(&props.class);
        }

        classes.join(" ")
    };

    html! {
        <div class="mb-3">
            <div class={container_class}>
                <input
                    type="checkbox"
                    class="form-check-input"
                    id={props.id.clone()}
                    checked={props.checked}
                    disabled={props.disabled}
                    {onchange}
                />
                <label class="form-check-label" for={props.id.clone()}>
                    {&props.label}
                </label>
                if props.error.is_some() {
                    <div class="invalid-feedback d-block">
                        {props.error.as_ref().unwrap()}
                    </div>
                }
            </div>
            if let Some(help) = &props.help_text {
                if props.error.is_none() {
                    <div class="form-text ms-4">
                        {help}
                    </div>
                }
            }
        </div>
    }
}
