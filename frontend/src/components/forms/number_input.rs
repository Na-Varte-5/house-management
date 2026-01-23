use yew::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Properties, PartialEq)]
pub struct NumberInputProps {
    /// Input field value
    pub value: String,

    /// Callback when value changes
    pub on_change: Callback<String>,

    /// Optional label text
    #[prop_or_default]
    pub label: Option<String>,

    /// Optional placeholder text
    #[prop_or_default]
    pub placeholder: Option<String>,

    /// Optional help text shown below input
    #[prop_or_default]
    pub help_text: Option<String>,

    /// Whether the field is required
    #[prop_or_default]
    pub required: bool,

    /// Whether the field is disabled
    #[prop_or_default]
    pub disabled: bool,

    /// Minimum value
    #[prop_or_default]
    pub min: Option<i64>,

    /// Maximum value
    #[prop_or_default]
    pub max: Option<i64>,

    /// Step value (for increment/decrement)
    #[prop_or_default]
    pub step: Option<String>,

    /// Validation error message
    #[prop_or_default]
    pub error: Option<String>,

    /// Additional CSS classes for the input
    #[prop_or_default]
    pub class: String,

    /// Size variant: "" (default), "sm", "lg"
    #[prop_or_default]
    pub size: String,
}

/// Reusable number input component with label, validation, and constraints
///
/// # Example
/// ```rust
/// html! {
///     <NumberInput
///         label="Construction Year"
///         value={(*year).clone()}
///         on_change={on_year_change}
///         placeholder="2020"
///         min={1800}
///         max={2100}
///         help_text="Enter the year the building was constructed"
///     />
/// }
/// ```
#[function_component(NumberInput)]
pub fn number_input(props: &NumberInputProps) -> Html {
    let oninput = {
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            on_change.emit(input.value());
        })
    };

    let input_class = {
        let mut classes = vec!["form-control"];

        if !props.size.is_empty() {
            classes.push(match props.size.as_str() {
                "sm" => "form-control-sm",
                "lg" => "form-control-lg",
                _ => "form-control",
            });
        }

        if props.error.is_some() {
            classes.push("is-invalid");
        }

        if !props.class.is_empty() {
            classes.push(&props.class);
        }

        classes.join(" ")
    };

    html! {
        <div class="mb-3">
            if let Some(label) = &props.label {
                <label class="form-label">
                    {label}
                    if props.required {
                        <span class="text-danger">{" *"}</span>
                    }
                </label>
            }
            <input
                type="number"
                class={input_class}
                value={props.value.clone()}
                placeholder={props.placeholder.clone().unwrap_or_default()}
                disabled={props.disabled}
                required={props.required}
                min={props.min.map(|v| v.to_string()).unwrap_or_default()}
                max={props.max.map(|v| v.to_string()).unwrap_or_default()}
                step={props.step.clone().unwrap_or_default()}
                {oninput}
            />
            if let Some(error) = &props.error {
                <div class="invalid-feedback d-block">
                    {error}
                </div>
            }
            if let Some(help) = &props.help_text {
                if props.error.is_none() {
                    <div class="form-text">
                        {help}
                    </div>
                }
            }
        </div>
    }
}
