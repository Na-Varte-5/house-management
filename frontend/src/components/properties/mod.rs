// Properties management components
//
// These components provide a modular interface for managing buildings,
// apartments, and owner assignments in the admin properties page.

mod apartment_form;
mod apartment_list;
mod building_form;
mod building_list;
mod owner_management;
mod types;

pub use apartment_form::ApartmentForm;
pub use apartment_list::ApartmentList;
pub use building_form::BuildingForm;
pub use building_list::BuildingList;
pub use owner_management::OwnerManagement;
pub use types::*;
