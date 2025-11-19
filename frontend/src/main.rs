pub mod routes;
pub mod components {
    pub mod auth_dropdown;
    pub mod navbar;
    pub mod spinner; // added
    pub mod announcements;
    pub mod announcement_list; // added
    pub mod announcement_editor; // added
    pub mod comment_list; // added
}
pub mod pages {
    pub mod building_apartments;
    pub mod buildings;
    pub mod home;
    pub mod login;
    pub mod admin; // added
    pub mod manage; // added
    pub mod health; // added
}
pub mod utils {
    pub mod api;
    pub mod auth; // added
    pub mod datetime; // localized datetime formatting
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
