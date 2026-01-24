# Frontend Development Guide

This file provides guidance for working with the frontend codebase. It documents available components, patterns, and conventions to maintain consistency and avoid code duplication.

**Note:** This file is read by Claude Code when working in the frontend directory. The root `/CLAUDE.md` contains general project guidance; this file contains frontend-specific information.

---

## Architecture Overview

**Framework:** Yew (Rust WebAssembly) with yew-router for SPA routing
**Styling:** Bootstrap 5 via CDN (use Bootstrap classes for all UI)
**State Management:** React-style hooks (`use_state`, `use_effect`) + Context API
**API Communication:** Centralized `api_client` from `services/api.rs`

### Directory Structure

```
src/
├── components/      - Reusable UI components
│   ├── properties/  - Properties management components
│   └── ...
├── contexts/        - Global state (AuthContext)
├── pages/          - Route-level page components
│   ├── admin/      - Admin-only pages
│   ├── maintenance/- Maintenance request pages
│   ├── meters/     - Meter management pages
│   └── voting/     - Voting system pages
├── services/       - API client and utilities
├── routes.rs       - Route definitions
└── app.rs          - Root component with router
```

---

## Available Components Library

### Layout Components

#### `<AdminLayout>`
**Purpose:** Layout wrapper for admin/manager pages with sidebar navigation
**Props:**
- `title: String` - Page title displayed above content
- `active_route: Route` - Current route for sidebar highlighting
- `children: Children` - Page content

**Usage:**
```rust
use crate::components::AdminLayout;
use crate::routes::Route;

html! {
    <AdminLayout title="Page Title" active_route={Route::AdminProperties}>
        <div class="container-fluid">
            // Your content here
        </div>
    </AdminLayout>
}
```

#### `<AppLayout>`
**Purpose:** Global layout for authenticated users (navbar + sidebar + content)
**Props:**
- `active_route: Route` - Current route for navigation highlighting
- `breadcrumbs: Option<Vec<BreadcrumbItem>>` - Optional breadcrumb navigation
- `children: Children` - Page content

**Usage:**
```rust
use crate::components::AppLayout;

html! {
    <AppLayout active_route={Route::Home}>
        // Your content here
    </AppLayout>
}
```

#### `<Navbar>`
**Purpose:** Top navigation bar (brand, user dropdown, language selector)
**Props:** None (uses `AuthContext` internally)
**Usage:**
```rust
use crate::components::navbar::navbar;

html! { <Navbar /> }
```

#### `<MainSidebar>`
**Purpose:** Left sidebar navigation for authenticated users
**Props:**
- `active_route: Route` - Current route for active highlighting

#### `<AdminSidebar>`
**Purpose:** Admin-specific sidebar (legacy, prefer MainSidebar)

#### `<Breadcrumb>`
**Purpose:** Breadcrumb navigation for nested pages
**Props:**
- `items: Vec<BreadcrumbItem>` - Breadcrumb items

**Usage:**
```rust
use crate::components::{Breadcrumb, BreadcrumbItem};

let breadcrumbs = vec![
    BreadcrumbItem { label: "Home".to_string(), route: Some(Route::Home) },
    BreadcrumbItem { label: "Properties".to_string(), route: None },
];

html! { <Breadcrumb items={breadcrumbs} /> }
```

---

### Alert Components

#### `<ErrorAlert>`
**Purpose:** Display error messages with dismiss button
**Props:**
- `message: String` - Error message to display
- `on_close: Callback<()>` - Called when user dismisses alert

**Usage:**
```rust
use crate::components::ErrorAlert;

if let Some(msg) = (*error).clone() {
    html! {
        <ErrorAlert
            message={msg}
            on_close={Callback::from(move |_| error.set(None))}
        />
    }
}
```

#### `<SuccessAlert>`
**Purpose:** Display success messages with dismiss button
**Props:**
- `message: String` - Success message
- `on_close: Callback<()>` - Dismiss callback

**Usage:** Same as ErrorAlert

---

### Form Components

**Location:** `src/components/forms/`

Reusable form input components that handle common patterns like validation, labels, help text, and error display. These components eliminate boilerplate and ensure consistent form styling across the application.

**IMPORTANT:** Always use these form components instead of raw HTML inputs for consistency and maintainability.

#### `<TextInput>`
**Purpose:** Text input with label, validation, and help text
**Props:**
- `value: String` - Input value (required)
- `on_change: Callback<String>` - Called when value changes (required)
- `label: Option<String>` - Label text
- `placeholder: Option<String>` - Placeholder text
- `help_text: Option<String>` - Help text shown below input
- `required: bool` - Whether field is required (default: false)
- `disabled: bool` - Whether field is disabled (default: false)
- `input_type: String` - Input type: "text", "email", "password", "url" (default: "text")
- `error: Option<String>` - Validation error message
- `class: String` - Additional CSS classes
- `size: String` - Size: "sm", "lg", or "" (default)

**Usage:**
```rust
use crate::components::TextInput;

let email = use_state(String::default);
let email_error = use_state(|| None::<String>);

let on_email_change = {
    let email = email.clone();
    let error = email_error.clone();
    Callback::from(move |value: String| {
        email.set(value.clone());
        // Validate on change
        if !value.contains('@') {
            error.set(Some("Invalid email".to_string()));
        } else {
            error.set(None);
        }
    })
};

html! {
    <TextInput
        label="Email Address"
        value={(*email).clone()}
        on_change={on_email_change}
        input_type="email"
        placeholder="user@example.com"
        required=true
        help_text="We'll never share your email"
        error={(*email_error).clone()}
    />
}
```

#### `<NumberInput>`
**Purpose:** Number input with constraints
**Props:**
- `value: String` - Input value (required)
- `on_change: Callback<String>` - Called when value changes (required)
- `label: Option<String>` - Label text
- `placeholder: Option<String>` - Placeholder text
- `help_text: Option<String>` - Help text
- `required: bool` - Whether field is required (default: false)
- `disabled: bool` - Whether field is disabled (default: false)
- `min: Option<i64>` - Minimum value
- `max: Option<i64>` - Maximum value
- `step: Option<String>` - Step value for increment/decrement
- `error: Option<String>` - Validation error message
- `class: String` - Additional CSS classes
- `size: String` - Size: "sm", "lg", or "" (default)

**Usage:**
```rust
use crate::components::NumberInput;

html! {
    <NumberInput
        label="Construction Year"
        value={(*year).clone()}
        on_change={on_year_change}
        placeholder="2020"
        min={1800}
        max={2100}
        help_text="Year the building was constructed"
    />
}
```

#### `<Select>`
**Purpose:** Dropdown select with options
**Props:**
- `value: String` - Currently selected value (required)
- `on_change: Callback<String>` - Called when selection changes (required)
- `options: Vec<SelectOption>` - Options to display (required)
- `label: Option<String>` - Label text
- `help_text: Option<String>` - Help text
- `required: bool` - Whether field is required (default: false)
- `disabled: bool` - Whether field is disabled (default: false)
- `placeholder: Option<String>` - Placeholder option shown when no value selected
- `error: Option<String>` - Validation error message
- `class: String` - Additional CSS classes
- `size: String` - Size: "sm", "lg", or "" (default)

**SelectOption struct:**
```rust
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self
    pub fn with_disabled(self, disabled: bool) -> Self
}
```

**Usage:**
```rust
use crate::components::{Select, SelectOption};

let voting_method = use_state(|| "SimpleMajority".to_string());

let method_options = vec![
    SelectOption::new("SimpleMajority", "Simple Majority"),
    SelectOption::new("WeightedArea", "Weighted by Area"),
    SelectOption::new("PerSeat", "One Vote Per Seat"),
    SelectOption::new("Consensus", "Consensus Required"),
];

html! {
    <Select
        label="Voting Method"
        value={(*voting_method).clone()}
        on_change={on_method_change}
        options={method_options}
        placeholder="Select a voting method"
        required=true
        help_text="Choose how votes will be counted"
    />
}
```

#### `<DateTimeInput>`
**Purpose:** Date/time input with smart defaults
**Props:**
- `value: String` - Input value in format "YYYY-MM-DDTHH:MM" or "YYYY-MM-DD" (required)
- `on_change: Callback<String>` - Called when value changes (required)
- `label: Option<String>` - Label text
- `help_text: Option<String>` - Help text
- `required: bool` - Whether field is required (default: false)
- `disabled: bool` - Whether field is disabled (default: false)
- `input_type: String` - Type: "date", "datetime-local", "time" (default: "datetime-local")
- `min: Option<String>` - Minimum datetime value
- `max: Option<String>` - Maximum datetime value
- `error: Option<String>` - Validation error message
- `class: String` - Additional CSS classes
- `size: String` - Size: "sm", "lg", or "" (default)

**Helper functions for default values:**
```rust
// Get current datetime as string
fn now_datetime() -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year() as i32;
    let month = (now.get_month() as f64 + 1.0) as i32;
    let day = now.get_date() as i32;
    let hours = now.get_hours() as i32;
    let minutes = now.get_minutes() as i32;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
}

// Add days to current datetime
fn datetime_plus_days(days: f64) -> String {
    let now = js_sys::Date::new_0();
    now.set_date((now.get_date() as f64 + days) as u32);
    let year = now.get_full_year() as i32;
    let month = (now.get_month() as f64 + 1.0) as i32;
    let day = now.get_date() as i32;
    let hours = now.get_hours() as i32;
    let minutes = now.get_minutes() as i32;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}", year, month, day, hours, minutes)
}
```

**Usage:**
```rust
use crate::components::DateTimeInput;

let start_time = use_state(now_datetime);
let end_time = use_state(|| datetime_plus_days(7.0));

html! {
    <DateTimeInput
        label="Start Time"
        value={(*start_time).clone()}
        on_change={on_start_change}
        input_type="datetime-local"
        required=true
        help_text="When should voting begin?"
    />

    <DateTimeInput
        label="End Time"
        value={(*end_time).clone()}
        on_change={on_end_change}
        input_type="datetime-local"
        min={(*start_time).clone()}
        required=true
        help_text="When should voting close?"
    />
}
```

#### `<Textarea>`
**Purpose:** Multiline text input with character counter
**Props:**
- `value: String` - Textarea value (required)
- `on_change: Callback<String>` - Called when value changes (required)
- `label: Option<String>` - Label text
- `placeholder: Option<String>` - Placeholder text
- `help_text: Option<String>` - Help text
- `required: bool` - Whether field is required (default: false)
- `disabled: bool` - Whether field is disabled (default: false)
- `rows: u32` - Number of visible rows (default: 3)
- `max_length: Option<u32>` - Maximum character count
- `error: Option<String>` - Validation error message
- `class: String` - Additional CSS classes
- `size: String` - Size: "sm", "lg", or "" (default)
- `show_counter: bool` - Show character counter (default: false)

**Usage:**
```rust
use crate::components::Textarea;

html! {
    <Textarea
        label="Description"
        value={(*description).clone()}
        on_change={on_description_change}
        placeholder="Enter a detailed description"
        rows={5}
        max_length={500}
        show_counter=true
        required=true
        help_text="Provide context for the proposal"
    />
}
```

#### `<Checkbox>`
**Purpose:** Checkbox or switch with label
**Props:**
- `id: String` - Unique ID for the checkbox (required for label association)
- `checked: bool` - Whether checkbox is checked (required)
- `on_change: Callback<bool>` - Called when checked state changes (required)
- `label: String` - Label text (required)
- `help_text: Option<String>` - Help text shown below checkbox
- `disabled: bool` - Whether field is disabled (default: false)
- `error: Option<String>` - Validation error message
- `class: String` - Additional CSS classes
- `switch: bool` - Use switch style instead of checkbox (default: false)
- `inline: bool` - Display inline for multiple checkboxes in a row (default: false)

**Usage:**
```rust
use crate::components::Checkbox;

let role_homeowner = use_state(|| true);
let role_renter = use_state(|| false);

html! {
    <Checkbox
        id="checkbox-homeowner"
        label="Homeowner"
        checked={*role_homeowner}
        on_change={on_homeowner_change}
        help_text="Allow homeowners to vote"
    />

    <Checkbox
        id="checkbox-renter"
        label="Renter"
        checked={*role_renter}
        on_change={on_renter_change}
        help_text="Allow renters to vote"
    />

    // Switch variant
    <Checkbox
        id="toggle-notifications"
        label="Enable email notifications"
        checked={*notifications}
        on_change={on_notifications_change}
        switch=true
    />
}
```

#### `<FormGroup>`
**Purpose:** Wrapper for grouping related form fields
**Props:**
- `children: Children` - Form fields to group (required)
- `title: Option<String>` - Group title/heading
- `description: Option<String>` - Group description
- `class: String` - Additional CSS classes
- `spacing: String` - Spacing: "compact", "spacious", or "" (default)

**Usage:**
```rust
use crate::components::FormGroup;

html! {
    <form onsubmit={on_submit}>
        <FormGroup
            title="Basic Information"
            description="Enter the proposal details"
        >
            <TextInput
                label="Title"
                value={(*title).clone()}
                on_change={on_title_change}
                required=true
            />
            <Textarea
                label="Description"
                value={(*description).clone()}
                on_change={on_description_change}
                required=true
            />
        </FormGroup>

        <FormGroup title="Voting Settings">
            <Select
                label="Method"
                value={(*method).clone()}
                on_change={on_method_change}
                options={methods}
                required=true
            />
            <DateTimeInput
                label="Start Time"
                value={(*start_time).clone()}
                on_change={on_start_change}
            />
        </FormGroup>

        <button type="submit" class="btn btn-primary">
            {"Create Proposal"}
        </button>
    </form>
}
```

---

### Utility Components

#### `<Spinner>`
**Purpose:** Loading spinner with optional text
**Usage:**
```rust
use crate::components::spinner::Spinner;

if loading {
    html! { <Spinner /> }
}
```

**Bootstrap Alternative:**
```rust
// Small inline spinner
html! {
    <div class="spinner-border spinner-border-sm" role="status">
        <span class="visually-hidden">{"Loading..."}</span>
    </div>
}
```

---

### Properties Management Components

**Location:** `src/components/properties/`

These components work together to manage buildings, apartments, and owners.

#### `<BuildingList>`
**Purpose:** Display selectable list of buildings with delete option
**Props:**
- `buildings: Vec<Building>` - Buildings to display
- `selected_building_id: Option<u64>` - Currently selected building ID
- `on_select: Callback<u64>` - Called when building is clicked
- `on_delete: Callback<u64>` - Called when delete button clicked
- `loading: bool` - Show loading state

**Usage:**
```rust
use crate::components::properties::{BuildingList, Building};

html! {
    <BuildingList
        buildings={(*buildings).clone()}
        selected_building_id={*selected_building}
        on_select={on_select_building}
        on_delete={on_delete_building}
        loading={*loading}
    />
}
```

#### `<ApartmentList>`
**Purpose:** Display selectable list of apartments with delete option
**Props:**
- `apartments: Vec<Apartment>` - Apartments to display
- `selected_apartment_id: Option<u64>` - Currently selected
- `on_select: Callback<u64>` - Selection callback
- `on_delete: Callback<u64>` - Delete callback
- `loading: bool` - Loading state
- `show: bool` - Whether to display (hides with message if false)

#### `<OwnerManagement>`
**Purpose:** Complete owner assignment interface (list + search + assign/remove)
**Props:**
- `owners: Vec<UserInfo>` - Current owners
- `all_users: Vec<UserInfo>` - All available users for assignment
- `user_query: String` - Search query
- `on_query_change: Callback<String>` - Search input callback
- `on_assign: Callback<u64>` - Assign owner callback
- `on_remove: Callback<u64>` - Remove owner callback
- `loading: bool` - Loading state
- `show: bool` - Whether to display

#### `<BuildingForm>`
**Purpose:** Form for creating new buildings
**Props:**
- `address: String` - Current address input value
- `year: String` - Current year input value
- `on_address_change: Callback<String>` - Address input callback
- `on_year_change: Callback<String>` - Year input callback
- `on_submit: Callback<()>` - Form submit callback
- `submitting: bool` - Disable during submission

#### `<ApartmentForm>`
**Purpose:** Form for creating new apartments
**Props:**
- `number: String` - Apartment number input
- `size: String` - Size input
- `on_number_change: Callback<String>` - Number input callback
- `on_size_change: Callback<String>` - Size input callback
- `on_submit: Callback<()>` - Submit callback
- `submitting: bool` - Disable during submission
- `show: bool` - Only show if building is selected

---

### Announcement Components

#### `<AnnouncementList>`
**Purpose:** Display list of public announcements with expand/collapse for comments
**Props:** None (loads data internally)
**Usage:**
```rust
use crate::components::announcement_list::AnnouncementList;

html! { <AnnouncementList /> }
```

#### `<AnnouncementEditor>` (Orchestrator)
**Purpose:** Orchestrator component that manages state and API calls for announcement editing
**Architecture:** Follows orchestrator pattern - manages state, loads data, handles API calls, creates callbacks
**Props:**
- `on_created: Callback<AnnouncementFull>` - Called when new announcement created
- `on_updated: Callback<AnnouncementFull>` - Called when existing announcement updated
- `on_published: Callback<AnnouncementFull>` - Called when announcement published
- `existing: Option<AnnouncementFull>` - Existing announcement to edit (None for create mode)
- `on_cancel: Callback<()>` - Called when user cancels editing

**Features:**
- Loads buildings and apartments for targeting
- Computes memoized markdown preview
- Handles form validation
- Supports scheduled publishing and expiration dates
- Role-based visibility controls

**Usage:**
```rust
use crate::components::announcement_editor::{AnnouncementEditor, AnnouncementFull};

// Create new announcement
html! {
    <AnnouncementEditor
        on_created={on_created}
        on_cancel={on_cancel}
    />
}

// Edit existing announcement
html! {
    <AnnouncementEditor
        existing={Some(announcement.clone())}
        on_updated={on_updated}
        on_cancel={on_cancel}
    />
}
```

#### `<AnnouncementEditorForm>` (Presentation)
**Purpose:** Pure presentation component for announcement editing form
**Architecture:** Receives all data via props, emits all events via callbacks - no state management or API calls
**Props:**
- Data props: `title`, `body_md`, `public_flag`, `pinned_flag`, `comments_enabled`, `publish_at`, `expire_at`, `selected_roles`, `selected_building`, `selected_apartment`, `buildings`, `apartments`, `preview_html`
- UI state props: `saving`, `error`, `is_editing`, `publish_now_id`
- Callback props: `on_title_change`, `on_body_md_change`, `on_public_change`, `on_pinned_change`, `on_comments_change`, `on_publish_at_change`, `on_expire_at_change`, `on_roles_change`, `on_building_change`, `on_apartment_change`, `on_submit`, `on_publish_now`, `on_cancel`

**Note:** This component is typically used internally by `<AnnouncementEditor>`. Only use directly if you need custom orchestration logic.

#### `<CommentList>`
**Purpose:** Display and manage comments on announcements
**Props:**
- `announcement_id: u64` - ID of announcement to show comments for
- `comments_enabled: bool` - Whether comments are enabled for this announcement

**Features:**
- Load and display comments
- Post new comments (authenticated users)
- Delete/restore/purge comments (Admin/Manager only)
- Toggle show deleted comments (Admin/Manager only)

**Usage:**
```rust
use crate::components::comment_list::CommentList;

html! {
    <CommentList
        announcement_id={announcement.id}
        comments_enabled={announcement.comments_enabled}
    />
}
```

---

### Authentication Components

#### `<AuthDropdown>`
**Purpose:** Login/register form in dropdown menu (used in Navbar)
**Note:** Handles its own state, no props needed

---

## State Management Patterns

### AuthContext

**Purpose:** Centralized authentication state
**Location:** `src/contexts/auth.rs`

**Available Methods:**
- `is_authenticated() -> bool` - Check if user is logged in
- `token() -> Option<&str>` - Get JWT token
- `user() -> Option<&UserClaims>` - Get user info from token
- `has_role(&str) -> bool` - Check if user has specific role
- `is_admin_or_manager() -> bool` - Check for elevated permissions
- `logout` - Logout callback (use `.emit(())`)

**Usage:**
```rust
use crate::contexts::AuthContext;

let auth = use_context::<AuthContext>().expect("AuthContext not found");

if !auth.is_authenticated() {
    return html! { <div>{"Please log in"}</div> };
}

if !auth.is_admin_or_manager() {
    return html! { <div>{"Access denied"}</div> };
}

let token = auth.token().map(|t| t.to_string());
```

### Component State

**Use Yew hooks for component-local state:**

```rust
// Simple state
let count = use_state(|| 0);
count.set(5); // Update

// Complex state
let items = use_state(|| Vec::<Item>::new());
items.set(new_vec); // Replace entire vec

// Multiple state variables
let loading = use_state(|| false);
let error = use_state(|| None::<String>);
let data = use_state(|| None::<Data>);
```

### Side Effects

**Use `use_effect_with` for data loading:**

```rust
use_effect_with(dependency.clone(), move |_| {
    wasm_bindgen_futures::spawn_local(async move {
        // Async work here
    });
    || () // Cleanup function
});
```

**Common patterns:**

```rust
// Load data on mount
{
    let data = data.clone();
    let token = token.clone();

    use_effect_with((), move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let client = api_client(token.as_deref());
            if let Ok(result) = client.get::<Data>("/endpoint").await {
                data.set(result);
            }
        });
        || ()
    });
}

// Load data when dependency changes
{
    let apartments = apartments.clone();
    let selected_building = selected_building.clone();

    use_effect_with(selected_building.clone(), move |_| {
        if let Some(bid) = *selected_building {
            wasm_bindgen_futures::spawn_local(async move {
                // Load apartments for building
            });
        }
        || ()
    });
}
```

---

## API Communication

### ALWAYS Use api_client

**❌ NEVER DO THIS:**
```rust
use reqwasm::http::Request;

let response = Request::get("/api/v1/endpoint")
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await?;
```

**✅ ALWAYS DO THIS:**
```rust
use crate::services::api_client;

let client = api_client(token.as_deref());
let data = client.get::<MyType>("/endpoint").await?;
```

**⚠️ IMPORTANT: URL Prefixing**

The `api_client` automatically adds the `/api/v1/` prefix to all endpoints. **Do NOT include it in your endpoint paths.**

```rust
// ❌ WRONG - duplicate prefix
client.get::<Data>("/api/v1/users").await?;  // Results in /api/v1/api/v1/users

// ✅ CORRECT - api_client adds /api/v1/ automatically
client.get::<Data>("/users").await?;  // Results in /api/v1/users
```

**Exception:** When using direct browser navigation (e.g., `window.open_with_url()` for CSV exports), you MUST include the full `/api/v1/` prefix since it bypasses the api_client.

### API Client Methods

**Location:** `src/services/api.rs`

```rust
// GET request
let data: MyType = client.get::<MyType>("/endpoint").await?;

// POST request with body
let response: ResponseType = client.post::<RequestType, ResponseType>(
    "/endpoint",
    &request_data
).await?;

// PUT request
let updated: MyType = client.put::<UpdateData, MyType>(
    "/endpoint/123",
    &update_data
).await?;

// DELETE with response
let deleted: MyType = client.delete::<MyType>("/endpoint/123").await?;

// DELETE without response (204 No Content)
client.delete_no_response("/endpoint/123").await?;

// POST with body but no response (201 Created, 204 No Content)
client.post_no_response("/endpoint", &request_data).await?;

// POST without request body
let result: MyType = client.post_empty::<MyType>("/endpoint").await?;
```

### Error Handling

```rust
use crate::services::ApiError;

match client.get::<Data>("/endpoint").await {
    Ok(data) => {
        // Handle success
    }
    Err(ApiError::Unauthorized) => {
        error.set(Some("Please log in".to_string()));
    }
    Err(ApiError::Forbidden) => {
        error.set(Some("Access denied".to_string()));
    }
    Err(ApiError::NotFound) => {
        error.set(Some("Resource not found".to_string()));
    }
    Err(e) => {
        error.set(Some(format!("Error: {}", e)));
    }
}
```

---

## Styling Guidelines

### Use Bootstrap 5 Classes

**DO NOT write custom CSS.** Use Bootstrap utility classes for all styling.

#### Layout

```rust
// Container
html! { <div class="container">...</div> }
html! { <div class="container-fluid">...</div> }

// Grid
html! {
    <div class="row">
        <div class="col-md-4">{"Column 1"}</div>
        <div class="col-md-4">{"Column 2"}</div>
        <div class="col-md-4">{"Column 3"}</div>
    </div>
}

// Spacing
class="mb-3"    // margin-bottom: 1rem
class="mt-4"    // margin-top: 1.5rem
class="p-2"     // padding: 0.5rem
class="px-3"    // padding-left and padding-right: 1rem
class="py-4"    // padding-top and padding-bottom: 1.5rem
```

#### Cards

```rust
html! {
    <div class="card">
        <div class="card-header">
            <h5 class="mb-0">{"Card Title"}</h5>
        </div>
        <div class="card-body">
            {"Card content"}
        </div>
    </div>
}
```

#### Buttons

```rust
// Primary action
html! { <button class="btn btn-primary">{"Submit"}</button> }

// Secondary action
html! { <button class="btn btn-secondary">{"Cancel"}</button> }

// Danger (delete)
html! { <button class="btn btn-danger">{"Delete"}</button> }

// Sizes
class="btn btn-sm"    // Small
class="btn btn-lg"    // Large

// Outline variant
class="btn btn-outline-primary"
```

#### Forms

```rust
html! {
    <div class="mb-3">
        <label class="form-label">{"Field Label"}</label>
        <input
            type="text"
            class="form-control"
            placeholder="Enter value"
        />
        <div class="form-text">{"Help text"}</div>
    </div>
}

// Validation states
class="form-control is-valid"
class="form-control is-invalid"

// Select
html! {
    <select class="form-select">
        <option>{"Option 1"}</option>
        <option>{"Option 2"}</option>
    </select>
}
```

#### Alerts

```rust
// Info
html! { <div class="alert alert-info">{"Info message"}</div> }

// Success
html! { <div class="alert alert-success">{"Success!"}</div> }

// Warning
html! { <div class="alert alert-warning">{"Warning"}</div> }

// Danger
html! { <div class="alert alert-danger">{"Error!"}</div> }

// Dismissible
html! {
    <div class="alert alert-info alert-dismissible fade show">
        {"Message"}
        <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
    </div>
}
```

#### Lists

```rust
// Basic list group
html! {
    <ul class="list-group">
        <li class="list-group-item">{"Item 1"}</li>
        <li class="list-group-item">{"Item 2"}</li>
    </ul>
}

// Interactive list (clickable items)
html! {
    <div class="list-group">
        <div class="list-group-item list-group-item-action">{"Item 1"}</div>
        <div class="list-group-item list-group-item-action active">{"Item 2 (selected)"}</div>
    </div>
}
```

#### Bootstrap Icons

**Icons available via `<i>` tags:**

```rust
html! { <i class="bi bi-trash"></i> }      // Delete icon
html! { <i class="bi bi-pencil"></i> }     // Edit icon
html! { <i class="bi bi-plus"></i> }       // Add icon
html! { <i class="bi bi-check"></i> }      // Check icon
html! { <i class="bi bi-x"></i> }          // Close icon
html! { <i class="bi bi-info-circle"></i> } // Info icon
```

**Icon CDN is included in `index.html`**

---

## Common Patterns

### Data Loading Pattern

```rust
#[function_component(MyPage)]
pub fn my_page() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let data = use_state(|| Vec::<Item>::new());
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let token = auth.token().map(|t| t.to_string());

    // Load data on mount
    {
        let data = data.clone();
        let loading = loading.clone();
        let error = error.clone();
        let token = token.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let client = api_client(token.as_deref());
                match client.get::<Vec<Item>>("/items").await {
                    Ok(list) => data.set(list),
                    Err(e) => error.set(Some(format!("Failed to load: {}", e))),
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Render
    if *loading {
        return html! { <Spinner /> };
    }

    if let Some(err) = (*error).clone() {
        return html! { <ErrorAlert message={err} on_close={/*...*/} /> };
    }

    html! {
        <div>
            { for data.iter().map(|item| html! { <div>{&item.name}</div> }) }
        </div>
    }
}
```

### Form Submission Pattern

```rust
let submitting = use_state(|| false);

let on_submit = {
    let submitting = submitting.clone();
    let error = error.clone();
    let success = success.clone();
    let token = token.clone();

    Callback::from(move |e: SubmitEvent| {
        e.prevent_default();

        // Validation
        if input.trim().is_empty() {
            error.set(Some("Field required".into()));
            return;
        }

        let submitting = submitting.clone();
        let error = error.clone();
        let success = success.clone();
        let token = token.clone();

        error.set(None);
        success.set(None);
        submitting.set(true);

        wasm_bindgen_futures::spawn_local(async move {
            let client = api_client(token.as_deref());
            match client.post::<_, Value>("/endpoint", &payload).await {
                Ok(_) => success.set(Some("Success!".to_string())),
                Err(e) => error.set(Some(format!("Error: {}", e))),
            }
            submitting.set(false);
        });
    })
};
```

### List with Selection Pattern

```rust
let selected_id = use_state(|| None::<u64>);

let on_select = {
    let selected = selected_id.clone();
    Callback::from(move |id: u64| selected.set(Some(id)))
};

html! {
    <div class="list-group">
        { for items.iter().map(|item| {
            let is_selected = *selected_id == Some(item.id);
            let class = if is_selected {
                "list-group-item list-group-item-action active"
            } else {
                "list-group-item list-group-item-action"
            };

            let item_id = item.id;
            let on_click = {
                let on_select = on_select.clone();
                Callback::from(move |_| on_select.emit(item_id))
            };

            html! {
                <div class={class} onclick={on_click} style="cursor: pointer;">
                    {&item.name}
                </div>
            }
        }) }
    </div>
}
```

---

## Component Best Practices

### 1. Always Use Props for Reusability

**❌ Bad - Hardcoded, not reusable:**
```rust
#[function_component(UserList)]
pub fn user_list() -> Html {
    let users = use_state(|| Vec::new());
    // Load users internally...
    html! { /* render */ }
}
```

**✅ Good - Accepts data via props:**
```rust
#[derive(Properties, PartialEq)]
pub struct UserListProps {
    pub users: Vec<User>,
    pub on_select: Callback<u64>,
}

#[function_component(UserList)]
pub fn user_list(props: &UserListProps) -> Html {
    // Just render, no data loading
    html! { /* render */ }
}
```

### 2. Use Callbacks for Events

**Always pass callbacks for user actions:**

```rust
#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub label: String,
    pub on_click: Callback<()>,
    #[prop_or(false)]
    pub disabled: bool,
}
```

### 3. Provide Loading States

**Always show loading feedback:**

```rust
if props.loading {
    return html! {
        <div class="text-center py-3">
            <div class="spinner-border spinner-border-sm" role="status">
                <span class="visually-hidden">{"Loading..."}</span>
            </div>
        </div>
    };
}
```

### 4. Use Semantic Props

**Use descriptive, specific prop names:**

```rust
// ❌ Bad
pub struct Props {
    pub show: bool,  // Show what?
    pub data: Vec<String>,  // What kind of data?
}

// ✅ Good
pub struct Props {
    pub show_deleted_items: bool,
    pub building_addresses: Vec<String>,
}
```

### 5. Handle Empty States

**Always provide feedback when lists are empty:**

```rust
if props.items.is_empty() {
    return html! {
        <div class="alert alert-info">
            {"No items found. Create one using the form below."}
        </div>
    };
}
```

---

## Creating New Components

### Checklist

When creating a new component:

1. ✅ Check if similar component already exists (see list above)
2. ✅ Use Bootstrap classes for styling (no custom CSS)
3. ✅ Accept data via props (don't load data internally)
4. ✅ Use callbacks for events
5. ✅ Provide loading state
6. ✅ Handle empty states
7. ✅ Use meaningful prop names
8. ✅ Add to appropriate module (components/ or components/domain/)
9. ✅ Export from mod.rs
10. ✅ **Document in this file!**

### Template

```rust
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MyComponentProps {
    /// Description of what this prop does
    pub required_prop: String,

    /// Optional prop with default
    #[prop_or_default]
    pub optional_prop: bool,

    /// Callback for user action
    pub on_action: Callback<()>,
}

/// Brief description of component purpose
#[function_component(MyComponent)]
pub fn my_component(props: &MyComponentProps) -> Html {
    // Handle loading state
    // Handle empty state
    // Render content

    html! {
        <div class="card">
            {&props.required_prop}
        </div>
    }
}
```

---

## Routing

### Route Definitions

**Location:** `src/routes.rs`

```rust
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/login")]
    Login,

    #[at("/buildings")]
    Buildings,

    #[at("/admin/properties")]
    AdminProperties,

    // Add new routes here
}
```

### Navigation

```rust
use yew_router::prelude::*;
use crate::routes::Route;

// Get navigator
let navigator = use_navigator().unwrap();

// Navigate programmatically
navigator.push(&Route::Home);

// Link component
html! {
    <Link<Route> to={Route::Buildings}>{"View Buildings"}</Link<Route>>
}
```

---

## Architecture Principles & Best Practices

### Single Responsibility Principle (CRITICAL)

**Each component should have ONE clear, focused purpose.**

#### ❌ BAD: Monolithic Component

```rust
// meters/management.rs - 750 lines
// Handles BOTH registration form AND list view AND filters
#[function_component(MeterManagementPage)]
pub fn meter_management_page() -> Html {
    // State for registration form
    let buildings = use_state(...);
    let apartments = use_state(...);
    let meter_type = use_state(...);
    let serial_number = use_state(...);

    // State for list view
    let all_meters = use_state(...);
    let search_query = use_state(...);
    let filter_building = use_state(...);

    // 700+ lines of mixed concerns...
}
```

**Problems:**
- Hard to test (must test registration AND listing together)
- Hard to reuse (registration form tied to this page)
- Hard to maintain (changes to one concern affect the other)
- Hard to understand (too many responsibilities)

#### ✅ GOOD: Separated Components

```rust
// pages/meters/management.rs - 145 lines (orchestrator)
#[function_component(MeterManagementPage)]
pub fn meter_management_page() -> Html {
    let active_tab = use_state(|| Tab::List);

    html! {
        <AdminLayout>
            if matches!(*active_tab, Tab::List) {
                <MeterList buildings={buildings} on_error={on_error} />
            } else {
                <MeterRegisterForm on_success={on_success} on_error={on_error} />
            }
        </AdminLayout>
    }
}

// components/meters/register_form.rs - 323 lines
// ONLY handles meter registration

// components/meters/list.rs - 363 lines
// ONLY handles meter listing with filters
```

**Benefits:**
- ✅ Each component is testable in isolation
- ✅ Components are reusable elsewhere
- ✅ Clear, focused responsibilities
- ✅ Changes to one don't affect the other

### Component Size Guidelines

**Hard limits to enforce Single Responsibility:**

| Component Type | Max Lines | Action if Exceeded |
|---------------|-----------|-------------------|
| Page components (routes) | ~200 lines | Split into sub-components |
| Reusable components | ~400 lines | Split into smaller components |
| Form components | ~300 lines | Extract sub-forms or fields |

**If a file exceeds these limits, it likely violates Single Responsibility Principle.**

### When to Split Components

Split a component when ANY of these apply:

1. **Multiple distinct responsibilities**
   - Example: Registration form + list view + filters
   - Solution: Create separate components for each

2. **Multiple unrelated concerns**
   - Example: Create + Edit + Delete all in one component
   - Solution: Separate components or at least separate functions

3. **File growing beyond 300-400 lines**
   - This is a red flag - review responsibilities

4. **Logic could be reused elsewhere**
   - Extract to reusable component

5. **Testing would benefit from isolation**
   - Hard to test? Split it up.

6. **State management is complex**
   - Too many `use_state` calls? Split responsibilities.

### Component Organization

```
src/
├── pages/              # Route-level orchestrators (max ~200 lines)
│   ├── meters/
│   │   └── management.rs   # Orchestrates tab switching
│   └── voting/
│       └── new.rs          # Orchestrates form submission
│
├── components/         # Reusable, focused components
│   ├── forms/          # Form input components (<300 lines each)
│   │   ├── text_input.rs
│   │   └── select.rs
│   │
│   ├── meters/         # Domain-specific meter components
│   │   ├── register_form.rs  # ONLY registration (~323 lines)
│   │   └── list.rs           # ONLY listing (~363 lines)
│   │
│   └── properties/     # Domain-specific property components
│       ├── building_list.rs
│       └── apartment_form.rs
```

**Rules:**
- **Pages** = Thin orchestrators that compose components
- **Components** = Focused, reusable, single-purpose
- **Domain folders** = Group related components by feature

### Yew/Rust Best Practices

#### 1. Props for Data, Callbacks for Events

```rust
#[derive(Properties, PartialEq)]
pub struct MyComponentProps {
    // Data in
    pub items: Vec<Item>,
    pub loading: bool,

    // Events out
    pub on_select: Callback<u64>,
    pub on_delete: Callback<u64>,
}
```

#### 2. Keep State Close to Usage

**❌ Bad: State in parent when only child needs it**
```rust
// Parent
let search_query = use_state(String::default);  // Only used by child!

html! { <ChildComponent search_query={search_query} /> }
```

**✅ Good: State in component that uses it**
```rust
// Child component manages its own search state
#[function_component(ChildComponent)]
pub fn child_component(props: &Props) -> Html {
    let search_query = use_state(String::default);
    // Use it here
}
```

#### 3. Explicit Effect Dependencies

```rust
// ✅ Good: Explicit dependency
use_effect_with(selected_building_id.clone(), move |building_id| {
    // Load apartments when building changes
    || ()
});

// ❌ Bad: Using () when you have dependencies
use_effect_with((), move |_| {
    // Uses selected_building_id but doesn't declare it!
    || ()
});
```

#### 4. Always Handle States

Every component should handle:
- ✅ **Loading state** - Show spinner or skeleton
- ✅ **Error state** - Show error alert
- ✅ **Empty state** - Show helpful message
- ✅ **Success state** - Show data

```rust
if *loading {
    return html! { <Spinner /> };
}

if let Some(err) = (*error).clone() {
    return html! { <ErrorAlert message={err} on_close={clear_error} /> };
}

if items.is_empty() {
    return html! { <div class="alert alert-info">{"No items found"}</div> };
}

// Render items...
```

#### 5. Component Composition Over Complexity

**❌ Bad: One complex component**
```rust
#[function_component(ComplexPage)]
pub fn complex_page() -> Html {
    // 50 state variables
    // 100 callbacks
    // 500 lines of HTML
}
```

**✅ Good: Composed from simple components**
```rust
#[function_component(SimplePage)]
pub fn simple_page() -> Html {
    html! {
        <PageLayout>
            <FilterPanel filters={filters} on_change={on_filter_change} />
            <ItemList items={filtered_items} on_select={on_select} />
            <ItemDetail item={selected_item} />
        </PageLayout>
    }
}
```

### Code Review Checklist

Before committing a component, verify:

- [ ] Component has ONE clear responsibility
- [ ] File is under size limits (~200 for pages, ~400 for components)
- [ ] State is managed at the right level
- [ ] Props and callbacks are well-typed
- [ ] Loading/error/empty states are handled
- [ ] Component could be tested in isolation
- [ ] Logic could be reused if needed
- [ ] Bootstrap classes used (no custom CSS)
- [ ] Form components used (not raw HTML inputs)

---

## Avoiding Common Mistakes

### ❌ Don't Duplicate Components

**Before creating a new component, check:**
1. This file's component library
2. `src/components/` directory
3. Properties components in `src/components/properties/`

### ❌ Don't Use Raw reqwasm

**Always use `api_client` from services module**

### ❌ Don't Write Custom CSS

**Use Bootstrap utility classes instead**

### ❌ Don't Ignore Loading/Error States

**Always provide feedback to users**

### ❌ Don't Hardcode API URLs

**Use relative paths - api_client handles the base URL**

```rust
// ❌ Bad
client.get("http://localhost:8080/api/v1/users")

// ✅ Good
client.get("/users")
```

---

## Questions?

When unsure about:
- **Component exists?** → Search this file's component library
- **How to style?** → Check Bootstrap documentation or examples above
- **API patterns?** → See API Communication section above
- **State management?** → See State Management Patterns above

**When in doubt, look at existing similar pages for patterns!**

---

## Contributing

When you add new reusable components:
1. Add them to appropriate location (`components/` or `components/domain/`)
2. Export from `mod.rs`
3. **Document them in this file with usage examples**
4. Follow the patterns established above

This keeps the component library discoverable and prevents duplication!
