use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;

#[derive(Clone, PartialEq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub route: Option<Route>,
}

#[derive(Properties, PartialEq)]
pub struct BreadcrumbProps {
    pub items: Vec<BreadcrumbItem>,
}

#[function_component(Breadcrumb)]
pub fn breadcrumb(props: &BreadcrumbProps) -> Html {
    if props.items.is_empty() {
        return html! {};
    }

    html! {
        <nav aria-label="breadcrumb" class="mb-3">
            <ol class="breadcrumb">
                {for props.items.iter().enumerate().map(|(idx, item)| {
                    let is_last = idx == props.items.len() - 1;

                    html! {
                        <li class={classes!("breadcrumb-item", is_last.then(|| "active"))}>
                            if let Some(route) = &item.route {
                                <Link<Route> to={route.clone()}>
                                    {&item.label}
                                </Link<Route>>
                            } else {
                                <span>{&item.label}</span>
                            }
                        </li>
                    }
                })}
            </ol>
        </nav>
    }
}
