use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DateTimeInputProps {
    /// Input field value (format: "YYYY-MM-DDTHH:MM" for datetime-local, "YYYY-MM-DD" for date)
    pub value: String,

    /// Callback when value changes
    pub on_change: Callback<String>,

    /// Optional label text
    #[prop_or_default]
    pub label: Option<String>,

    /// Optional help text shown below input
    #[prop_or_default]
    pub help_text: Option<String>,

    /// Whether the field is required
    #[prop_or_default]
    pub required: bool,

    /// Whether the field is disabled
    #[prop_or_default]
    pub disabled: bool,

    /// Input type: "date", "datetime-local", "time"
    #[prop_or("datetime-local".to_string())]
    pub input_type: String,

    /// Minimum datetime value
    #[prop_or_default]
    pub min: Option<String>,

    /// Maximum datetime value
    #[prop_or_default]
    pub max: Option<String>,

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

/// Reusable date/time input component with label, validation, and help text
///
/// # Example
/// ```rust
/// html! {
///     <DateTimeInput
///         label="Start Time"
///         value={(*start_time).clone()}
///         on_change={on_start_change}
///         input_type="datetime-local"
///         required=true
///         help_text="When should voting begin?"
///     />
/// }
/// ```
///
/// # Helper function for default datetime
/// ```rust
/// use yew::prelude::*;
///
/// // Get current datetime as string
/// fn now_datetime() -> String {
///     let now = js_sys::Date::new_0();
///     let year = now.get_full_year() as i32;
///     let month = (now.get_month() as f64 + 1.0) as i32;
///     let day = now.get_date() as i32;
///     let hours = now.get_hours() as i32;
///     let minutes = now.get_minutes() as i32;
///     format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
/// }
///
/// // Add days to current datetime
/// fn datetime_plus_days(days: f64) -> String {
///     let now = js_sys::Date::new_0();
///     now.set_date((now.get_date() as f64 + days) as u32);
///     let year = now.get_full_year() as i32;
///     let month = (now.get_month() as f64 + 1.0) as i32;
///     let day = now.get_date() as i32;
///     let hours = now.get_hours() as i32;
///     let minutes = now.get_minutes() as i32;
///     format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
/// }
///
/// let start_time = use_state(now_datetime);
/// let end_time = use_state(|| datetime_plus_days(7.0));
/// ```
#[function_component(DateTimeInput)]
pub fn datetime_input(props: &DateTimeInputProps) -> Html {
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
                disabled={props.disabled}
                required={props.required}
                min={props.min.clone().unwrap_or_default()}
                max={props.max.clone().unwrap_or_default()}
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
