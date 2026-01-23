use yew::prelude::*;
use web_sys::HtmlInputElement;

#[derive(Properties, PartialEq)]
pub struct TextInputProps {
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

    /// Input type (text, email, password, etc.)
    #[prop_or("text".to_string())]
    pub input_type: String,

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

/// Reusable text input component with label, validation, and help text
///
/// # Example
/// ```rust
/// html! {
///     <TextInput
///         label="Email Address"
///         value={(*email).clone()}
///         on_change={on_email_change}
///         input_type="email"
///         placeholder="user@example.com"
///         required=true
///         help_text="We'll never share your email"
///         error={(*email_error).clone()}
///     />
/// }
/// ```
#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
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
                type={props.input_type.clone()}
                class={input_class}
                value={props.value.clone()}
                placeholder={props.placeholder.clone().unwrap_or_default()}
                disabled={props.disabled}
                required={props.required}
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
