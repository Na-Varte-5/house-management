use yew::prelude::*;

/// Reusable Bootstrap spinner.
/// Props:
/// - color: Bootstrap contextual color suffix (e.g. "secondary", "primary"), defaults to "secondary".
/// - small: whether to use the small variant (adds spinner-border-sm), defaults true.
/// - class: extra classes appended to the outer div.
/// - center: if true wraps spinner in a flex/center container for easy centering.
#[derive(Properties, PartialEq)]
pub struct SpinnerProps {
    #[prop_or(String::from("secondary"))]
    pub color: String,
    #[prop_or(true)]
    pub small: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or(false)]
    pub center: bool,
    #[prop_or(String::from("Loading"))]
    pub label: String,
}

#[function_component(Spinner)]
pub fn spinner(props: &SpinnerProps) -> Html {
    let size_class = if props.small {
        " spinner-border-sm"
    } else {
        ""
    };
    let color_class = format!(" text-{}", props.color);
    let mut classes = format!("spinner-border{}{}", size_class, color_class);
    if !props.class.is_empty() {
        classes.push_str(" ");
        classes.push_str(&props.class.to_string());
    }
    let spinner = html! {<div class={classes} role="status"><span class="visually-hidden">{props.label.clone()}</span></div>};
    if props.center {
        html! {<div class="d-flex justify-content-center align-items-center">{spinner}</div>}
    } else {
        spinner
    }
}
