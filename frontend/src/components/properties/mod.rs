// Properties management components
//
// These components provide a modular interface for managing buildings,
// apartments, and owner assignments in the admin properties page.

mod types;
mod building_list;
mod apartment_list;
mod owner_management;
mod building_form;
mod apartment_form;

pub use types::*;
pub use building_list::BuildingList;
pub use apartment_list::ApartmentList;
pub use owner_management::OwnerManagement;
pub use building_form::BuildingForm;
pub use apartment_form::ApartmentForm;
