use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Properties, PartialEq)]
pub struct SelectProps {
    /// Currently selected value
    pub value: String,

    /// Callback when selection changes
    pub on_change: Callback<String>,

    /// Options to display
    pub options: Vec<SelectOption>,

    /// Optional label text
    #[prop_or_default]
    pub label: Option<String>,

    /// Optional help text shown below select
    #[prop_or_default]
    pub help_text: Option<String>,

    /// Whether the field is required
    #[prop_or_default]
    pub required: bool,

    /// Whether the field is disabled
    #[prop_or_default]
    pub disabled: bool,

    /// Optional placeholder option (shown when no value selected)
    #[prop_or_default]
    pub placeholder: Option<String>,

    /// Validation error message
    #[prop_or_default]
    pub error: Option<String>,

    /// Additional CSS classes for the select
    #[prop_or_default]
    pub class: String,

    /// Size variant: "" (default), "sm", "lg"
    #[prop_or_default]
    pub size: String,
}

/// Reusable select dropdown component with label, validation, and help text
///
/// # Example
/// ```rust
/// let options = vec![
///     SelectOption::new("simple", "Simple Majority"),
///     SelectOption::new("weighted", "Weighted by Area"),
///     SelectOption::new("consensus", "Consensus"),
/// ];
///
/// html! {
///     <Select
///         label="Voting Method"
///         value={(*voting_method).clone()}
///         on_change={on_method_change}
///         options={options}
///         placeholder="Select a voting method"
///         required=true
///         help_text="Choose how votes will be counted"
///     />
/// }
/// ```
#[function_component(Select)]
pub fn select(props: &SelectProps) -> Html {
    let onchange = {
        let on_change = props.on_change.clone();
        Callback::from(move |e: Event| {
            let select: HtmlSelectElement = e.target_unchecked_into();
            on_change.emit(select.value());
        })
    };

    let select_class = {
        let mut classes = vec!["form-select"];

        if !props.size.is_empty() {
            classes.push(match props.size.as_str() {
                "sm" => "form-select-sm",
                "lg" => "form-select-lg",
                _ => "form-select",
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
            <select
                class={select_class}
                value={props.value.clone()}
                disabled={props.disabled}
                required={props.required}
                {onchange}
            >
                if let Some(placeholder) = &props.placeholder {
                    <option value="" disabled={true}>
                        {placeholder}
                    </option>
                }
                { for props.options.iter().map(|opt| {
                    html! {
                        <option
                            value={opt.value.clone()}
                            disabled={opt.disabled}
                        >
                            {&opt.label}
                        </option>
                    }
                }) }
            </select>
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
