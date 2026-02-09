use crate::components::Breadcrumb;
use crate::components::breadcrumb::BreadcrumbItem;
use crate::components::properties::PropertyDetailContent;
use crate::routes::Route;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub apartment_id: u64,
}

#[function_component(MyPropertyDetailPage)]
pub fn my_property_detail_page(props: &Props) -> Html {
    let breadcrumbs = vec![
        BreadcrumbItem {
            label: "My Properties".to_string(),
            route: Some(Route::MyProperties),
        },
        BreadcrumbItem {
            label: format!("Apartment #{}", props.apartment_id),
            route: None,
        },
    ];

    html! {
        <>
            <Breadcrumb items={breadcrumbs} />
            <PropertyDetailContent apartment_id={props.apartment_id} />
        </>
    }
}
