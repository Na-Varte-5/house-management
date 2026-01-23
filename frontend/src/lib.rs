pub mod utils {
    pub mod api; // may be unused by tests
    pub mod auth;
    pub mod datetime; // added for localized formatting
}
pub mod components;
pub mod contexts;
pub mod pages;
pub mod routes;
pub mod services; // use components/mod.rs for submodules
pub use components::announcement_list::AnnouncementList;
pub mod i18n; // expose translation helper

// datetime module exposed above
