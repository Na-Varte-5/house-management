use crate::components::Breadcrumb;
use crate::components::breadcrumb::BreadcrumbItem;
use crate::components::maintenance::MaintenanceDetailContent;
use crate::i18n::t;
use crate::routes::Route;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: u64,
}

#[function_component(MaintenanceDetailPage)]
pub fn maintenance_detail_page(props: &Props) -> Html {
    let breadcrumbs = vec![
        BreadcrumbItem {
            label: t("nav-maintenance"),
            route: Some(Route::Maintenance),
        },
        BreadcrumbItem {
            label: format!("Request #{}", props.id),
            route: None,
        },
    ];

    html! {
        <>
            <Breadcrumb items={breadcrumbs} />
            <MaintenanceDetailContent id={props.id} />
        </>
    }
}
