use crate::components::breadcrumb::{Breadcrumb, BreadcrumbItem};
use crate::components::main_sidebar::MainSidebar;
use crate::routes::Route;
use yew::prelude::*;

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
            <MainSidebar active_route={props.active_route.clone()} />

            <div class="main-content flex-grow-1" style="margin-left: 250px; padding-top: 56px;">
                <div class="container-fluid p-4">
                    if let Some(breadcrumbs) = &props.breadcrumbs {
                        <Breadcrumb items={breadcrumbs.clone()} />
                    }
                    {for props.children.iter()}
                </div>
            </div>
        </div>
    }
}
