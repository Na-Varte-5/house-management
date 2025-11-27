use yew_router::prelude::*;

#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/buildings")]
    Buildings,
    #[at("/buildings/:id/apartments")]
    BuildingApartments { id: u64 },
    #[at("/login")]
    Login,
    #[at("/admin")]
    Admin,
    #[at("/admin/announcements")]
    AdminAnnouncements,
    #[at("/admin/properties")]
    AdminProperties,
    #[at("/manage")]
    Manage,
    #[at("/health")]
    Health,
}
