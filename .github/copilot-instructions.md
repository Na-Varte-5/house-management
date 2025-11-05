House/Apartment Management Platform (Rust)
User Management

Roles & Permissions: Implement a role-based access control (RBAC) system where each user can hold multiple roles (Admin, Homeowner, Renter, Management, HOA Member). Define permission sets for each role (e.g. Administrators manage all data, Homeowners view their properties, Renters have limited access) and enforce these in API handlers.

Authentication: Use JWT (JSON Web Tokens) for stateless auth
auth0.com
. Provide endpoints for user registration and login that issue a signed JWT on success. Store user credentials securely (hashed passwords) and include user role claims in the token.

Authorization: Protect API routes by validating the JWT on each request. Build middleware or guards that check the token’s validity and user roles before allowing access to protected resources. Allow admins to create and manage users and assign them multiple roles.

Property Hierarchy

Data Models: Create database tables/models for Buildings (Houses) and Apartments/Units. One building can have many apartments (one-to-many), and each apartment can link to multiple owners (many-to-many) via a join table. For example: buildings(id, address, year, ...), apartments(id, building_id, size, ...), and apartment_owners(apartment_id, owner_id).

APIs: Implement CRUD endpoints for buildings and apartments. Include endpoints to list all buildings, view apartments in a building, and assign/remove owners to apartments. Each building record should store basic info (address, construction year) and optional documents (e.g. floor plans). Each apartment record should include size, specifications, and link to its owner(s). Use Diesel ORM to model these relations and migrations to create tables.

Maintenance & Complaints

Request Submission: Allow users (Homeowners/Renters) to submit maintenance or complaint requests. Define request types (e.g., urgent repair, regular maintenance, improvement, complaint) and include fields like title, description, and priority.

Tracking & Workflow: Track each request’s status (e.g., Open, In Progress, Resolved). Assign requests to responsible staff members. Include endpoints to update status and add resolution notes. For example, POST /api/v1/requests to create a request and PUT /api/v1/requests/{id}/status to update it. The frontend should display status and allow filtering by open/completed.

Assignment & History: Enable management personnel to view and assign requests. Store maintenance history in each apartment’s record. Record all status changes and resolution details to build an audit trail of maintenance actions.

Billing & Financials

Cost Calculation: Support multiple billing formulas to split shared costs. Examples include per-person costs (e.g. garbage removal), area-based costs (e.g. repair fund), equal split among units, or custom formulas. Implement logic on the backend to generate each month’s charges per apartment based on the selected method.

Payments & Reporting: Track invoices and payments for each unit. Maintain payment history with timestamps. Provide APIs to record payments and fetch outstanding balances. Generate financial reports summarizing income, expenses, and budgets. For instance, endpoints like GET /api/v1/financials/reports returning JSON summaries (total collected, pending dues) and GET /api/v1/owners/{id}/payments for owner-specific history.

Receipts & History: Include endpoints to download receipts or export CSV of charges. Frontend should display a table of charges and payments per unit and aggregate data (e.g., “total paid vs. expected”). Use Diesel migrations to create tables like bills, payments, and accounts.

Event Calendar

Event Entities: Create an Event model with fields for title, description, date/time, location, and recurrence rules. Include event types (maintenance, community meeting, special event, etc.).

Calendar Features: Implement endpoints to create, update, list, and delete events. Support recurring events (weekly meetings, recurring maintenance) and reminders. The frontend calendar component should allow users to navigate by month/week, click events to view details, and see notifications for upcoming events.

Community Scheduling: Allow filtering events by building or type. For example, GET /api/v1/events?building=1 to show only events in a particular building. Ensure time zones and date formats are handled consistently.

Voting System

Proposals & Voting: Allow administrators or HOA members to create proposals (e.g., policy changes) and configure voting parameters (start/end times, eligible voters). Support different voting styles: simple majority, weighted voting (e.g., by apartment size or share) and consensus-based rules.

Voting Process: Provide APIs to submit votes (e.g., POST /api/v1/votes). Calculate results at the end of the voting period according to the chosen method. Store each user’s vote and mark whether they have voted. Ensure only authorized roles can vote.

Results & History: Display voting results (passed/failed, percentages) and keep historical records of all votes and outcomes. Frontend pages should list active and past votes. Include summary charts or stats in the admin dashboard if desired.

Document Management

File Uploads: Implement endpoints to upload and retrieve documents (PDFs, images). Use Actix’s multipart support (actix-multipart) to handle file uploads in the /api server. For example, POST /api/v1/documents with form data.

Storage & Access: Store files in a designated directory or cloud storage, and save metadata (filename, URL, owner) in the database. Associate documents with entities (e.g., a building’s floor plans, HOA bylaws).

Access Control: Enforce access rules: only users with the right roles (e.g., Admin, Manager, or associated homeowner) can view certain documents. Use JWT-based checks on download endpoints. Frontend should allow browsing and viewing (or downloading) available documents securely.

Messaging System

Real-Time Chat: Provide a message center for owner-to-management communication. For real-time chat, use WebSockets – Actix Web has built-in support via the actix-ws crate
actix.rs
. Implement a WebSocket endpoint (e.g. /ws/messages) where authenticated users can join chat rooms or personal chats.

Persistent Messaging: Alternatively or additionally, support REST APIs to send and fetch messages (e.g., POST /api/v1/messages, GET /api/v1/messages?user=1). Store messages in a messages table with sender, receiver, timestamp, and content. Use Yew components to display chat threads.

Notifications: Notify users of new messages (via WebSocket pushes or polling). Include features like unread count. Ensure message delivery is secure and only accessible to intended recipients.

News Board / Announcements

Post System: Create a news/announcements feature with posts (title, content, author, date). Support pinning important notices. Only certain roles (Admin/Manager) can create or pin posts.

Commenting: Optionally allow comments on posts. If so, have a comments table linked to posts and users.

APIs: Endpoints for listing, viewing, creating, and editing posts and comments (with permission checks). Ensure posts are displayed in chronological or pinned order on the frontend.

Analytics Dashboard

Data Aggregation: Build backend endpoints that aggregate data (e.g., average maintenance requests per month, payment compliance rates by building) to feed an analytics dashboard.

Charts & Visuals: Use Yew-compatible chart libraries for frontend graphs. For example, use visualize-yew
docs.rs
or yew-chart
github.com
to render bar charts, line graphs, pie charts, etc.

Dashboard Components: Create dashboard pages showing trends (e.g., maintenance requests over time, payment status across apartments). Each graph should have tooltips/legends. Ensure charts are responsive.

Access Log & Visitor Management

Visitor Tracking: Allow logging of guest entries. Create a model (e.g., visitors) with fields for visitor name, host apartment, check-in/out times. Provide endpoints to add and list logs.

PIN/QR Access: Implement optional features for controlled access: generate one-time PIN codes or QR codes for guests. For example, use the qrcode crate to create a QR image representing an access token
docs.rs
. Verify PIN or scanned QR on entry (this would be a custom implementation depending on hardware).

Logs & Reports: Store each entry attempt. Create a page to view recent visitor logs and filter by apartment or date. This enhances building security.

Mobile Responsiveness

Frontend Layout: Use Bootstrap CSS (already chosen) for responsive design. Ensure all components (forms, tables, navbars) adapt to mobile screens using Bootstrap’s grid and utilities.

Touch-Friendly Controls: Design buttons and form fields large enough for touch. Use Bootstrap’s responsive classes (e.g., col-sm-, col-md-) so pages render well on phones and tablets. Test on various viewports.

Testing: Verify the Yew app works on mobile browsers. Consider a mobile-first CSS approach to ensure usability on small screens.

Multilingual Support

Supported Languages: Structure the app for i18n with English and Czech as defaults. Ensure all UI text (labels, messages) is translatable.

Internationalization Library: Use a crate like yew-i18n
docs.rs
or similar to load translations. For example, define a context/provider that supplies language strings to components.

Implementation: Keep translation files (JSON or TOML) for each language. Detect user’s browser language or allow manual selection. Wrap UI text in translation hooks. Provide a language switcher in the settings.

Technical Stack & Organization

Repository Layout: Create /api directory for backend (Actix) and /frontend for Yew. Use a Rust Cargo workspace if needed.

Backend (Actix-web): Use Actix-web for the REST API. Organize code into modules (e.g. users.rs, roles.rs, buildings.rs, events.rs). In main.rs, mount scopes (e.g. web::scope("/api/v1")).

Database (Diesel): Use MySQL with Diesel ORM. Write Diesel migrations (diesel migration generate ...) for all tables. Diesel’s versioned migrations automatically track applied changes
diesel.rs
. For example, one migration to create users, another for roles, apartments, etc.

Authentication Middleware: Implement JWT middleware (e.g., with actix-web-httpauth or custom guards) to decode and validate tokens on each request. Embed user roles in the JWT claims for easy checks.

API Design: Version all routes (e.g. /api/v1/...). Return JSON responses. Use Actix’s built-in error handling (ResponseError) to map Rust errors into HTTP statuses consistently.

Asynchronous Operations: Write all handlers as async fn. Use .await for DB calls. Avoid blocking calls.

Logging & Errors: Log requests and errors. Return meaningful HTTP status codes (400 for bad request, 401 unauthorized, etc.). Provide JSON error bodies.

Frontend (Yew): Use Yew (latest stable) for the frontend. Initialize the project with Trunk or wasm-pack.

Routing: Add yew-router for SPA routing
yew.rs
. Define routes for pages (Login, Dashboard, Maintenance, etc.) and wrap in <BrowserRouter>.

Components: Build reusable components (forms, tables, modals). Store global state (authenticated user, JWT token, current language) using Yew’s context or agent. Use reqwasm or fetch to call the backend APIs and handle JSON.

Styling: Import Bootstrap CSS. Use Bootstrap classes for layout (e.g., <div class="container">, <div class="row">). Ensure forms and buttons use Bootstrap styling.

Auth Flow: On login, save JWT (e.g. in LocalStorage). Include it in the Authorization header for all API calls. Redirect unauthorized users to login.

Documentation & Comments: Add clear module and function comments so Copilot (or future developers) can understand intended behavior. For example, use /// doc comments above each handler describing its purpose and expected data.

CI/CD: Set up automated tests for both backend (using cargo test) and frontend. Consider GitHub Actions to lint, build, and deploy. Containerize the backend (Dockerfile) if deploying to cloud.

Use cargo-mcp for building and testing.

## Current Implementation Status (Session Sync)

The repository now includes:

- Soft-delete (is_deleted flag) for `buildings` and `apartments` with list + restore endpoints.
- Endpoints added: `/buildings/deleted`, `/buildings/{id}/restore`, `/apartments/deleted`, `/apartments/{id}/restore` plus DELETE routes performing soft-delete.
- Manager page in Yew consolidates: building creation, apartment creation, owner assignment (searchable list of public users), deletion confirmation modal, show deleted toggle, restore buttons, and loading spinners.
- Owner assignment uses `GET /api/v1/apartments/{id}/owners`, `POST /api/v1/apartments/{id}/owners`, `DELETE /api/v1/apartments/{id}/owners/{user_id}`; duplicates are ignored gracefully.
- UI shows spinners for buildings/apartments/owners/deleted lists and updates in-place after delete/restore.
- Role-based access enforced (Admin/Manager) for create, delete, restore, and owner assignment endpoints.

When proposing future code:
- Prefer adding tests for soft-delete/restore flows and RBAC guards.
- Preserve separation of active vs deleted entities in UI.
- Avoid permanent deletes; continue using logical deletion unless explicitly changed.
- Suggest debounced search, pagination, and optimistic updates only after current behavior covered by tests.
