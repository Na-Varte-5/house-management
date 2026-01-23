use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FormGroupProps {
    /// Child elements (form fields)
    pub children: Children,

    /// Optional title/heading for the group
    #[prop_or_default]
    pub title: Option<String>,

    /// Optional description text
    #[prop_or_default]
    pub description: Option<String>,

    /// Additional CSS classes
    #[prop_or_default]
    pub class: String,

    /// Spacing variant: "" (default mb-3), "compact" (mb-2), "spacious" (mb-4)
    #[prop_or_default]
    pub spacing: String,
}

/// Wrapper component for grouping related form fields with consistent spacing
///
/// # Example
/// ```rust
/// html! {
///     <form>
///         <FormGroup
///             title="Basic Information"
///             description="Enter the proposal details"
///         >
///             <TextInput
///                 label="Title"
///                 value={(*title).clone()}
///                 on_change={on_title_change}
///             />
///             <Textarea
///                 label="Description"
///                 value={(*description).clone()}
///                 on_change={on_description_change}
///             />
///         </FormGroup>
///
///         <FormGroup title="Voting Settings">
///             <Select
///                 label="Method"
///                 value={(*method).clone()}
///                 on_change={on_method_change}
///                 options={methods}
///             />
///         </FormGroup>
///     </form>
/// }
/// ```
#[function_component(FormGroup)]
pub fn form_group(props: &FormGroupProps) -> Html {
    let container_class = {
        let mut classes = vec![];

        let spacing = match props.spacing.as_str() {
            "compact" => "mb-2",
            "spacious" => "mb-4",
            _ => "mb-3",
        };
        classes.push(spacing);

        if !props.class.is_empty() {
            classes.push(&props.class);
        }

        classes.join(" ")
    };

    html! {
        <div class={container_class}>
            if let Some(title) = &props.title {
                <div class="mb-2">
                    <h6 class="fw-semibold mb-1">{title}</h6>
                    if let Some(desc) = &props.description {
                        <small class="text-muted">{desc}</small>
                    }
                </div>
            }
            { for props.children.iter() }
        </div>
    }
}
