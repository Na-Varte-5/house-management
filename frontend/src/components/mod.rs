pub mod admin_layout;
pub mod admin_sidebar;
pub mod announcement_editor; // editor component (orchestrator)
pub mod announcement_editor_form; // editor form component (presentation layer)
pub mod announcement_list; // new announcements list component
pub mod announcements; // management composite component
pub mod app_layout;
pub mod auth_dropdown;
pub mod breadcrumb;
pub mod comment_list; // comments component
pub mod confirm_modal;
pub mod error_alert;
pub mod forms; // reusable form input components
pub mod language_switcher;
pub mod main_sidebar;
pub mod maintenance;
pub mod meters; // meters management components
pub mod navbar;
pub mod pagination;
pub mod properties; // properties management components
pub mod search_input;
pub mod spinner; // exported for reuse // maintenance request components
pub mod toast;

pub use app_layout::AppLayout;
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use confirm_modal::ConfirmModal;
pub use error_alert::{ErrorAlert, SuccessAlert};
pub use forms::{
    Checkbox, DateTimeInput, FormGroup, NumberInput, Select, SelectOption, TextInput, Textarea,
};
pub use language_switcher::LanguageSwitcher;
pub use main_sidebar::MainSidebar;
pub use pagination::Pagination;
pub use search_input::SearchInput;
pub use toast::{ToastContext, ToastLevel, ToastProvider};
