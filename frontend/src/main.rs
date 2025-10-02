pub mod routes;
pub mod components {
    pub mod auth_dropdown;
    pub mod navbar;
}
pub mod pages {
    pub mod building_apartments;
    pub mod buildings;
    pub mod home;
    pub mod login;
}
pub mod utils {
    pub mod auth;
}
mod app;
mod i18n;

use app::App;
use i18n::init_translations;

fn main() {
    // Initialize translations
    init_translations();

    // Render the app
    yew::Renderer::<App>::new().render();
}
