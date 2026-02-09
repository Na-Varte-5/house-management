// Properties management components
//
// These components provide a modular interface for managing buildings,
// apartments, and owner assignments in the admin properties page.

mod admin_properties_data;
mod apartment_form;
mod apartment_list;
mod building_form;
mod building_list;
mod meter_card_list;
mod owner_management;
mod property_detail_content;
mod property_history_timeline;
mod renter_management;
mod types;

pub use admin_properties_data::AdminPropertiesData;
pub use apartment_form::ApartmentForm;
pub use apartment_list::ApartmentList;
pub use building_form::BuildingForm;
pub use building_list::BuildingList;
pub use meter_card_list::{MeterCardList, MeterWithLastReading};
pub use owner_management::OwnerManagement;
pub use property_detail_content::PropertyDetailContent;
pub use property_history_timeline::{PropertyHistoryEvent, PropertyHistoryTimeline};
pub use renter_management::{RenterInfo, RenterManagement};
pub use types::*;
