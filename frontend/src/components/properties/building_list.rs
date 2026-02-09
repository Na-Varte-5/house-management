use super::types::Building;
use crate::i18n::t;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct BuildingListProps {
    pub buildings: Vec<Building>,
    pub selected_building_id: Option<u64>,
    pub on_select: Callback<u64>,
    pub on_delete: Callback<u64>,
    pub loading: bool,
}

#[function_component(BuildingList)]
pub fn building_list(props: &BuildingListProps) -> Html {
    if props.loading {
        return html! {
            <div class="text-center py-3">
                <div class="spinner-border spinner-border-sm" role="status">
                    <span class="visually-hidden">{t("loading")}</span>
                </div>
            </div>
        };
    }

    if props.buildings.is_empty() {
        return html! {
            <div class="alert alert-info small mb-0">
                {t("properties-no-buildings-create")}
            </div>
        };
    }

    html! {
        <div class="list-group">
            { for props.buildings.iter().map(|b| {
                let is_selected = props.selected_building_id == Some(b.id);
                let item_class = if is_selected {
                    "list-group-item list-group-item-action active d-flex justify-content-between align-items-center"
                } else {
                    "list-group-item list-group-item-action d-flex justify-content-between align-items-center"
                };

                let building_id = b.id;
                let on_click = {
                    let on_select = props.on_select.clone();
                    Callback::from(move |_| on_select.emit(building_id))
                };

                let on_delete = {
                    let on_delete_cb = props.on_delete.clone();
                    Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        on_delete_cb.emit(building_id);
                    })
                };

                html! {
                    <div class={item_class} onclick={on_click} style="cursor: pointer;">
                        <div>
                            <strong>{&b.address}</strong>
                            if let Some(year) = b.construction_year {
                                <span class="text-muted small ms-2">{format!("({})", year)}</span>
                            }
                        </div>
                        if is_selected {
                            <button
                                class="btn btn-sm btn-outline-light"
                                onclick={on_delete}
                                title="Delete building"
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
