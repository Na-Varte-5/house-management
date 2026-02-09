use crate::components::properties::AdminPropertiesData;
use crate::contexts::AuthContext;
use crate::i18n::t;
use yew::prelude::*;

#[function_component(AdminPropertiesPage)]
pub fn admin_properties_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");

    if !auth.is_admin_or_manager() {
        return html! {
            <div class="container mt-4">
                <div class="alert alert-danger">
                    <strong>{t("admin-access-denied")}</strong>
                    <p class="mb-0 small">{t("admin-need-permission")}</p>
                </div>
            </div>
        };
    }

    html! {
        <>
            <h2 class="mb-3">{t("manage-properties-title")}</h2>
            <AdminPropertiesData />
        </>
    }
}
