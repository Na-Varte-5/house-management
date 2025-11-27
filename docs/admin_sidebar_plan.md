# Admin / Manager Sidebar & Layout Plan

This document captures the plan and remaining steps for refactoring the admin/manager UI into a sidebar-based layout and de-cluttering the top navbar.

## Current State (already implemented)

1. **Sidebar infrastructure**
   - `frontend/src/components/admin_sidebar.rs` (`AdminSidebar`):
     - Visible only for users with `Admin` or `Manager` roles (based on `current_user()` claims).
     - Contains links:
       - "User Management" → `Route::Admin` (Admins only).
       - "Buildings & Apartments" → `Route::Manage` (Admins & Managers).
     - Highlights the active entry based on `active_route` prop.

   - `frontend/src/components/admin_layout.rs` (`AdminLayout`):
     - Performs a privilege check (Admin or Manager) using `current_user()`.
     - Renders a responsive Bootstrap layout:
       - Left column (md+): vertical sidebar (`AdminSidebar`) in a narrow `col-md-3 col-lg-2`.
       - Mobile: sidebar hidden by default with an "Admin menu" button that toggles a Bootstrap `collapse`.
       - Right column: main content with optional `title` rendered as an `<h2>`.

   - `frontend/src/components/mod.rs` exports both components:
     - `pub use admin_sidebar::AdminSidebar;`
     - `pub use admin_layout::AdminLayout;`

2. **Pages using the new layout**

   - `frontend/src/pages/admin.rs` (`AdminPage`):
     - Still enforces `is_admin` guard.
     - Uses `AdminLayout` with `title = "Admin - User Management"` and `active_route = Route::Admin`.
     - Existing user/role management table and behavior are unchanged; they now live inside a card within the layout.

   - `frontend/src/pages/manage.rs` (`ManagePage`):
     - Still enforces `can_manage` (Admin or Manager).
     - Uses `AdminLayout` with `title = "Manager"` and `active_route = Route::Manage`.
     - Currently combines **manager-focused functionality** in one page:
       - **Announcements section** at the top via `AnnouncementsManage`.
       - **Buildings CRUD** + soft delete/restore + "Show Deleted" toggle.
       - **Apartments CRUD** + soft delete/restore.
       - **Owner assignment UI** with searchable list of public users.
       - **Deleted entities** sections (buildings and apartments).
     - All handlers, effects, and API calls for these areas are preserved.

   - **New dedicated admin pages & routes**
     - `frontend/src/routes.rs`:
       - `Route::AdminAnnouncements` mapped to `/admin/announcements`.
       - `Route::AdminProperties` mapped to `/admin/properties`.
     - `frontend/src/pages/admin_announcements.rs` (`AdminAnnouncementsPage`):
       - Enforces `can_manage` (Admin or Manager).
       - Uses `AdminLayout` with `title = "Announcements"` and `active_route = Route::AdminAnnouncements`.
       - Renders `<AnnouncementsManage />` as the main content, providing a focused view for creating, editing, listing, soft-deleting, and restoring announcements.
     - `frontend/src/pages/admin_properties.rs` (`AdminPropertiesPage`):
       - Enforces `can_manage` (Admin or Manager).
       - Uses `AdminLayout` with `title = "Properties"` and `active_route = Route::AdminProperties`.
       - Currently being refactored to host the **Buildings & Apartments** management UI (copied from `ManagePage`), without announcements.

3. **Announcements management UX**

   - `frontend/src/components/announcements.rs` (`AnnouncementsManage`):
     - Fetches active and deleted announcements.
     - Supports pinning, soft delete/restore, publish now, and toggling comments.
     - Renders a list with status badges (Pinned, Scheduled, Expired) and audience badges (public/private, roles, building/apartment scopes).
     - **New announcement flow**:
       - A "New announcement" button in the header toggles an inline editor card.
       - Inline editor uses `AnnouncementEditor` (`announcement_editor.rs`) and closes on successful create or cancel.
     - Deleted announcements are listed separately with Restore/Purge actions.

4. **Validation**
   - Frontend build/tests have been run via:
     - `./scripts/test.sh frontend` (where available).
     - Manual `trunk build` runs after substantive UI changes.
   - Static error checks on edited frontend files are clean; remaining warnings are about unused helpers and imports only.

---

## Phase A – Sidebar & Layout Polish

Goal: Refine the existing sidebar and layout to align with the rest of the app (i18n, shared role helpers, minor UX polish).

### A1. I18n for sidebar labels

- [ ] Replace hard-coded labels in `AdminSidebar` with translation keys, e.g.:
  - "Management" → `t("sidebar-management")`
  - "User Management" → `t("sidebar-user-management")`
  - "Buildings & Apartments" → `t("sidebar-buildings-apartments")`
  - (Optionally) dedicated labels for new routes when the sidebar is extended:
    - `sidebar-admin-announcements`
    - `sidebar-admin-properties`
- [ ] Add these keys to both language files:
  - `frontend/locales/en/frontend.ftl`
  - `frontend/locales/cs/frontend.ftl`
- [ ] Optionally add short descriptions/tooltips if desired.

### A2. Shared role helpers

Currently, role checks are duplicated in:
- Navbar
- `AdminSidebar`
- `AdminLayout`
- `AdminPage` / `ManagePage` / `AdminAnnouncementsPage` / `AdminPropertiesPage`

Refactor to use shared helpers:

- [ ] Add helper functions to `frontend/src/utils/auth.rs` (or a new `utils/roles.rs`):
  - `fn is_admin(user: &User) -> bool`
  - `fn is_manager_or_admin(user: &User) -> bool`
  - `fn is_privileged(user: &User) -> bool` (Admin or Manager)
- [ ] Update:
  - `Navbar` to use these helpers when deciding what to show.
  - `AdminSidebar` to compute `is_admin` and `is_manager_or_admin` via helpers.
  - `AdminLayout` to use `is_privileged`.
  - `AdminPage`, `ManagePage`, `AdminAnnouncementsPage`, and `AdminPropertiesPage` access guards to use the same helpers.

This keeps role logic centralized and easier to evolve.

### A3. Visual/UX tweaks

- [ ] Consider adding a small subtitle in `AdminLayout` that indicates the user role (e.g., "Admin" / "Manager"), or a badge near the title.
- [ ] Adjust spacing if needed:
  - Verify `mt-*`/`mb-*` classes feel right under the global navbar.
  - Avoid excessive nested margins inside cards.

---

## Phase B – Navbar Cleanup

Goal: Reduce privileged clutter in the top navbar and delegate complex admin navigation to the sidebar.

### B1. Replace `Admin` and `Manage` links with a single "Dashboard" link

File: `frontend/src/components/navbar.rs`

- [ ] For users with `is_manager_or_admin(user)`:
  - Remove the separate `Route::Admin` and `Route::Manage` nav links.
  - Add a single link, for example:

    ```rust
    <Link<Route> to={Route::Manage} classes="nav-link">
        { t("nav-dashboard") }
    </Link<Route>>
    ```

- [ ] Add `nav-dashboard` translation key to:
  - `frontend/locales/en/frontend.ftl`
  - `frontend/locales/cs/frontend.ftl`

This makes `/manage` the central privileged entry point. The sidebar then handles navigation between specific admin areas (user management, announcements, properties, etc.).

### B2. (Optional) Move dashboard link into user dropdown instead

If we want the main navbar even cleaner:

- [ ] Optionally remove the dashboard link from the top nav entirely.
- [ ] Inside the user dropdown (where name/email and roles are displayed), conditionally add a small dashboard item for privileged users:

  ```rust
  if is_manager_or_admin(&u) {
      <Link<Route> to={Route::Manage} classes="dropdown-item small">
          { t("nav-dashboard") }
      </Link<Route>>
      <div class="dropdown-divider"></div>
  }
  ```

This turns admin access into a user-profile affordance rather than a top-level navigation item.

---

## Phase C – Decompose & Enhance Admin / Manage Content

Goal: Make admin and manager pages less visually dense and more modular, while taking advantage of the new dedicated routes.

### C1. Introduce an `AdminSectionCard` helper

- [ ] New component: `frontend/src/components/admin_section_card.rs`.
- [ ] Suggested props:
  - `title: String`
  - `subtitle: Option<String>`
  - `initially_collapsed: bool` (optional; default `false`)
  - `children: Children`
- [ ] Behavior:
  - Renders a Bootstrap `card`.
  - Card header contains title and a small toggle button (or clickable header) to collapse/expand the body.
  - Collapse handling can be implemented either with Yew state or Bootstrap `collapse`.
- [ ] Export via `components/mod.rs`.

### C2. Apply `AdminSectionCard` to manager/admin pages

Now that we have **separate routes** for announcements and properties, we can simplify each page:

- [ ] `AdminAnnouncementsPage`:
  - Optionally wrap `<AnnouncementsManage />` in an `AdminSectionCard` titled "Announcements Management".
  - Keep the page focused only on announcements.

- [ ] `AdminPropertiesPage`:
  - Move the buildings card (form + list + Show Deleted toggle + deleted buildings section) into one `AdminSectionCard` titled "Buildings".
  - Move the apartments/owners card into another `AdminSectionCard` titled "Apartments & Owners".

- [ ] `ManagePage` (combined view):
  - Decide whether to:
    - Keep it as a "Manager Dashboard" that shows a high-level overview (maybe condensed versions of announcements and properties), or
    - Gradually migrate managers to use the dedicated `/admin/announcements` and `/admin/properties` routes instead.

This preserves behavior but makes each screen more focused and scannable.

### C3. (Later) Split large functionality into additional dedicated routes/pages

If admin areas grow further:

- [ ] Add new routes to `frontend/src/routes.rs`, for example:
  - `Route::MaintenanceAdmin` (e.g., `/admin/maintenance`)
  - `Route::FinancialAdmin` (e.g., `/admin/financials`)
- [ ] Create pages under `frontend/src/pages/`:
  - `maintenance_admin.rs` for maintenance/complaints workflows.
  - `financials_admin.rs` for billing & financial reports.
- [ ] Update `AdminSidebar` to include these new routes as additional links.

For now, we have:
- `AdminPage` for user management.
- `AdminAnnouncementsPage` for announcements.
- `AdminPropertiesPage` for buildings/apartments.

---

## Phase D – Future Privileged Modules Hook-in

When implementing additional high-privilege features from the overall house-management design, reuse the `AdminLayout` + `AdminSidebar` pattern:

### D1. Maintenance & Complaints Admin View

- [ ] New page: `frontend/src/pages/maintenance_admin.rs`.
- [ ] UI: list and filter maintenance/complaint requests, assign staff, update status, view history.
- [ ] Wrap with `AdminLayout` (e.g., title `"Maintenance / Requests"`, `active_route = Route::MaintenanceAdmin`).
- [ ] Add a corresponding sidebar entry (visible to Admin/Manager).

### D2. Billing & Financials Admin View

- [ ] New page: `frontend/src/pages/financials_admin.rs`.
- [ ] UI: per-building/unit charges, invoices, payments, aggregate reports.
- [ ] Wrap with `AdminLayout`.
- [ ] Add sidebar entry.

### D3. Voting, Visitors, Analytics

- [ ] For each module (voting system, visitor management, analytics dashboard), follow the same pattern:
  - New route in `routes.rs`.
  - New page under `frontend/src/pages/` using `AdminLayout`.
  - New link in `AdminSidebar` conditioned on appropriate roles.

---

## How to Continue

When resuming work:

- For quick, low-risk improvements:
  - Start with **Phase A** (i18n + role helper functions) and then **Phase B** (navbar cleanup).
- If the manager page already feels too dense:
  - Continue with **Phase C2** and introduce `AdminSectionCard` to visually separate functionality and lean more on the new `AdminAnnouncementsPage` and `AdminPropertiesPage`.
- As new backend features are added (maintenance, billing, voting, etc.):
  - Use **Phase D** as the pattern to plug them cleanly into the existing sidebar layout.
