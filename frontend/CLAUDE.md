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
**Purpose:** Display list of announcements
**Usage:**
```rust
use crate::components::announcement_list::AnnouncementList;

html! { <AnnouncementList /> }
```

#### `<AnnouncementEditor>`
**Purpose:** Rich editor for creating/editing announcements

#### `<CommentList>`
**Purpose:** Display and manage comments on announcements

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
