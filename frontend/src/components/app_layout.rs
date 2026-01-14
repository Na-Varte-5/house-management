use yew::prelude::*;
use crate::routes::Route;
use crate::components::main_sidebar::MainSidebar;
use crate::components::breadcrumb::{Breadcrumb, BreadcrumbItem};

#[derive(Properties, PartialEq)]
pub struct AppLayoutProps {
    pub active_route: Route,
    pub children: Children,
    #[prop_or_default]
    pub breadcrumbs: Option<Vec<BreadcrumbItem>>,
}

#[function_component(AppLayout)]
pub fn app_layout(props: &AppLayoutProps) -> Html {
    html! {
        <div class="d-flex">
            // Fixed left sidebar
            <MainSidebar active_route={props.active_route.clone()} />

            // Main content area with left margin to account for fixed sidebar
            <div class="flex-grow-1" style="margin-left: 250px; padding-top: 56px;">
                <div class="container-fluid p-4">
                    // Optional breadcrumbs
                    if let Some(breadcrumbs) = &props.breadcrumbs {
                        <Breadcrumb items={breadcrumbs.clone()} />
                    }

                    // Page content
                    {for props.children.iter()}
                </div>
            </div>
        </div>
    }
}
