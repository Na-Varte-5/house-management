use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TextareaProps {
    /// Textarea value
    pub value: String,

    /// Callback when value changes
    pub on_change: Callback<String>,

    /// Optional label text
    #[prop_or_default]
    pub label: Option<String>,

    /// Optional placeholder text
    #[prop_or_default]
    pub placeholder: Option<String>,

    /// Optional help text shown below textarea
    #[prop_or_default]
    pub help_text: Option<String>,

    /// Whether the field is required
    #[prop_or_default]
    pub required: bool,

    /// Whether the field is disabled
    #[prop_or_default]
    pub disabled: bool,

    /// Number of visible text rows
    #[prop_or(3)]
    pub rows: u32,

    /// Maximum character count
    #[prop_or_default]
    pub max_length: Option<u32>,

    /// Validation error message
    #[prop_or_default]
    pub error: Option<String>,

    /// Additional CSS classes for the textarea
    #[prop_or_default]
    pub class: String,

    /// Size variant: "" (default), "sm", "lg"
    #[prop_or_default]
    pub size: String,

    /// Show character counter
    #[prop_or(false)]
    pub show_counter: bool,
}

/// Reusable textarea component with label, validation, and character counter
///
/// # Example
/// ```rust
/// html! {
///     <Textarea
///         label="Description"
///         value={(*description).clone()}
///         on_change={on_description_change}
///         placeholder="Enter a detailed description"
///         rows={5}
///         max_length={500}
///         show_counter=true
///         required=true
///         help_text="Provide context for the proposal"
///     />
/// }
/// ```
#[function_component(Textarea)]
pub fn textarea(props: &TextareaProps) -> Html {
    let oninput = {
        let on_change = props.on_change.clone();
        Callback::from(move |e: InputEvent| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            on_change.emit(textarea.value());
        })
    };

    let textarea_class = {
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

    let char_count = props.value.len();
    let max_chars = props.max_length.unwrap_or(0) as usize;
    let counter_class = if max_chars > 0 && char_count > max_chars {
        "text-danger"
    } else {
        "text-muted"
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
            <textarea
                class={textarea_class}
                value={props.value.clone()}
                placeholder={props.placeholder.clone().unwrap_or_default()}
                disabled={props.disabled}
                required={props.required}
                rows={props.rows.to_string()}
                maxlength={props.max_length.map(|v| v.to_string()).unwrap_or_default()}
                {oninput}
            />
            if let Some(error) = &props.error {
                <div class="invalid-feedback d-block">
                    {error}
                </div>
            }
            <div class="d-flex justify-content-between">
                if let Some(help) = &props.help_text {
                    if props.error.is_none() {
                        <div class="form-text">
                            {help}
                        </div>
                    }
                } else {
                    <div />
                }
                if props.show_counter {
                    <small class={classes!(counter_class, "form-text")}>
                        if let Some(max) = props.max_length {
                            {format!("{} / {}", char_count, max)}
                        } else {
                            {format!("{} characters", char_count)}
                        }
                    </small>
                }
            </div>
        </div>
    }
}
