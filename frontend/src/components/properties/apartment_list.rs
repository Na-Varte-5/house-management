use yew::prelude::*;
use super::types::Apartment;

#[derive(Properties, PartialEq)]
pub struct ApartmentListProps {
    pub apartments: Vec<Apartment>,
    pub selected_apartment_id: Option<u64>,
    pub on_select: Callback<u64>,
    pub on_delete: Callback<u64>,
    pub loading: bool,
    pub show: bool,
}

#[function_component(ApartmentList)]
pub fn apartment_list(props: &ApartmentListProps) -> Html {
    if !props.show {
        return html! {
            <div class="alert alert-info small mb-0">
                <i class="bi bi-info-circle me-2"></i>
                {"Select a building to view its apartments"}
            </div>
        };
    }

    if props.loading {
        return html! {
            <div class="text-center py-3">
                <div class="spinner-border spinner-border-sm" role="status">
                    <span class="visually-hidden">{"Loading..."}</span>
                </div>
            </div>
        };
    }

    if props.apartments.is_empty() {
        return html! {
            <div class="alert alert-info small mb-0">
                {"No apartments in this building. Create one using the form below."}
            </div>
        };
    }

    html! {
        <div class="list-group">
            { for props.apartments.iter().map(|apt| {
                let is_selected = props.selected_apartment_id == Some(apt.id);
                let item_class = if is_selected {
                    "list-group-item list-group-item-action active d-flex justify-content-between align-items-center"
                } else {
                    "list-group-item list-group-item-action d-flex justify-content-between align-items-center"
                };

                let apartment_id = apt.id;
                let on_click = {
                    let on_select = props.on_select.clone();
                    Callback::from(move |_| on_select.emit(apartment_id))
                };

                let on_delete = {
                    let on_delete_cb = props.on_delete.clone();
                    Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        on_delete_cb.emit(apartment_id);
                    })
                };

                html! {
                    <div class={item_class} onclick={on_click} style="cursor: pointer;">
                        <div>
                            <strong>{&apt.number}</strong>
                            if let Some(size) = apt.size_sq_m {
                                <span class="text-muted small ms-2">{format!("{:.1} m²", size)}</span>
                            }
                        </div>
                        if is_selected {
                            <button
                                class="btn btn-sm btn-outline-light"
                                onclick={on_delete}
                                title="Delete apartment"
                            >
                                <i class="bi bi-trash"></i>
                            </button>
                        }
                    </div>
                }
            }) }
        </div>
    }
}
