pub mod admin_layout;
pub mod admin_sidebar;
pub mod announcement_editor; // editor component
pub mod announcement_list; // new announcements list component
pub mod announcements; // management composite component
pub mod app_layout;
pub mod auth_dropdown;
pub mod breadcrumb;
pub mod comment_list; // comments component
pub mod error_alert;
pub mod forms; // reusable form input components
pub mod main_sidebar;
pub mod maintenance;
pub mod meters; // meters management components
pub mod navbar;
pub mod properties; // properties management components
pub mod spinner; // exported for reuse // maintenance request components

pub use admin_layout::AdminLayout;
pub use admin_sidebar::AdminSidebar;
pub use app_layout::AppLayout;
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use error_alert::{ErrorAlert, SuccessAlert};
pub use forms::{
    Checkbox, DateTimeInput, FormGroup, NumberInput, Select, SelectOption, TextInput, Textarea,
};
pub use main_sidebar::MainSidebar;
