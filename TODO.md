# House Management System - TODO Tracker

This file tracks progress and planned work. Tasks are organized by priority tier based on user feedback (2026-01-14).

Legend: [ ] planned, [x] done, [~] in progress, [!] blocked

---

## PRIORITY TIER 1: Critical UX & Foundation ✅ COMPLETED (2026-01-14)
**Goal:** Fix immediate usability issues, establish multi-HOA RBAC foundation
**Estimated effort:** 1-2 weeks | **Impact:** Makes app immediately more usable

### Navigation & Layout Improvements
- [x] Switch to left sidebar navigation (user links stay top-right)
- [x] Add active route highlighting in navbar
- [x] Add breadcrumb navigation for nested pages
- [x] Create Home/Dashboard page with statistics and activity feed (quick stats cards, recent activity, action items)
- [x] Fix navbar overlay issue on public pages (added padding-top for unauthenticated views)
- [x] Fix logout flow (navigate to home instead of reload to avoid 401 errors)

### Multi-HOA RBAC Foundation
- [x] Add building_id scoping to all RBAC checks (users only see their buildings)
- [x] Create building_members via apartment_owners + apartment_renters + building_managers
- [x] Add building_id to voting proposals and filter by user's buildings
- [x] Filter maintenance requests, meters, announcements by user's building access
- [x] Update all list endpoints to respect building-scoped access
- [x] Create building_managers table migration
- [x] Create apartment_renters table migration
- [x] Implement get_user_building_ids() helper for RBAC
- [x] Add building manager CRUD endpoints (assign, remove, list)

### Properties Dashboard UX Fixes
- [x] Add visual selection indicators for buildings and apartments (Bootstrap .active class)
- [x] Hide/disable assign owner panel until apartment is selected
- [x] Show placeholder message when no apartment selected
- [x] Conditional display of owner management sections

### Form & Date Handling Improvements
- [x] Add building scope dropdown to proposal creation form
- [x] Add sensible date defaults (proposals: start=now, end=+7 days)
- [x] Add '(optional)' labels to non-required form fields (building scope)
- [x] Add help text for building scope selection

---

## PRIORITY TIER 2: Owner Experience Core (Critical Missing Functionality)
**Goal:** Address "owners don't see value yet" - make app useful for regular users
**Estimated effort:** 2-3 weeks | **Impact:** Provides value to non-admin users

### Owner Property Management
- [ ] Create 'My Properties' view filtering apartments owned/rented by user
- [ ] Add tenant management system:
  - [ ] Create apartment_renters table (id, apartment_id, user_id, start_date, end_date, is_active)
  - [ ] Invite renter endpoint (by email, sends registration link or creates account)
  - [ ] Set rental period (start/end dates)
  - [ ] Revoke access (soft-expire relationship, remove renter role if no other apartments)
  - [ ] View current and past tenants per apartment
- [ ] Implement auto-role assignment logic:
  - [ ] User assigned as apartment owner → gets Homeowner role
  - [ ] User assigned as renter → gets Renter role
  - [ ] When last apartment assignment removed → role auto-revoked (check other apartments first)
- [ ] Create property history/timeline view (maintenance requests, tenant changes, updates)

### Multi-ownership Support
- [ ] Allow multiple owners per apartment (apartment_owners already supports this, just need UI)
- [ ] Add ownership_percentage field to apartment_owners table (optional, for legal/voting weight)
- [ ] Update voting logic: when multiple owners per apartment, last vote wins (overwrites previous owner's vote)
- [ ] Add UI to show all owners on apartment detail page

---

## PRIORITY TIER 3: Manager & Maintenance Enhancements
**Goal:** Improve daily operations, enable delegation
**Estimated effort:** 2 weeks | **Impact:** Better workflows for property management

### Building Managers
- [ ] Create building_managers table (id, building_id, user_id, created_at) - many-to-many
- [ ] Implement building-scoped manager permissions (managers only see assigned buildings)
- [ ] Add manager assignment UI in admin properties dashboard
- [ ] Update RBAC checks: Manager role respects building assignments
- [ ] Add "Assign Manager" button similar to "Assign Owner"

### Maintenance Comments & Collaboration
- [ ] Create maintenance_request_comments table (id, request_id, user_id, comment_text, created_at)
- [ ] Add comments API endpoints:
  - [ ] POST /api/v1/requests/{id}/comments (create comment)
  - [ ] GET /api/v1/requests/{id}/comments (list comments)
- [ ] RBAC for comments: creator, assignee, apartment owners, Admin/Manager can comment
- [ ] Display comments in history feed on maintenance detail page (mixed with status changes)
- [ ] Extend attachment permissions: creator, assignee, apartment owners can upload (not just admin)
- [ ] Implement owner escalation workflow:
  - [ ] Renter submits request → initially assigned to owner
  - [ ] Owner can escalate to manager (change assignment)
  - [ ] Manager handles or reassigns

---

## PRIORITY TIER 4: Major New Modules (Large Features)
**Goal:** Add critical HOA operations functionality
**Estimated effort:** 4-6 weeks total | **Impact:** Essential for real-world HOA management

### Payment/Billing System
- [ ] Design payment schema:
  - [ ] fee_structures table (id, building_id, fee_type, amount, calculation_method, is_active)
  - [ ] fee_types: monthly, special_assessment, meter_based, per_person, custom
  - [ ] calculation_methods: per_apartment, per_square_meter, per_person, usage_based, fixed_amount
  - [ ] invoices table (id, apartment_id, user_id, period_start, period_end, total_amount, status, due_date)
  - [ ] invoice_line_items table (id, invoice_id, description, amount, fee_structure_id)
  - [ ] payments table (id, invoice_id, user_id, amount, payment_method, transaction_id, paid_at)
- [ ] Create configurable fee structures (per apartment, per person, per meter, custom)
- [ ] Implement monthly fee calculation engine (aggregate fees per apartment)
- [ ] Add special assessments (one-time charges, can target all or specific apartments)
- [ ] Create payment tracking UI:
  - [ ] Owner view: "My Invoices" (paid/overdue status)
  - [ ] Admin view: payment dashboard (overdue apartments, total collected)
- [ ] Add payment history and export (CSV, PDF reports)
- [ ] Email notifications for payment due/overdue

### Document Management System
- [ ] Create documents table:
  - [ ] Columns: id, building_id, title, description, file_path, file_size, mime_type, uploaded_by, visibility_scope, category, created_at
  - [ ] visibility_scope enum: all_residents, owners_only, specific_apartments, managers_only, admin_only
  - [ ] category enum: bylaws, minutes, financial, contracts, rules, maintenance, other
- [ ] Implement visibility scopes (filter documents based on user role and apartment assignments)
- [ ] Add document upload UI (admin/manager):
  - [ ] Multipart upload with metadata form
  - [ ] Select visibility scope and category
  - [ ] Optional: assign to specific apartments
- [ ] Create document library page:
  - [ ] List view with filtering (category, date range)
  - [ ] Search by title/description
  - [ ] Download with audit trail (track who downloaded when)
- [ ] Add reasonable file size limits (50MB per file, 500MB per building, configurable in AppConfig)
- [ ] Backend: store under STORAGE_DIR/documents/{building_id}/{uuid}

---

## PRIORITY TIER 5: Communication & Polish (Nice-to-Have)
**Goal:** Quality-of-life improvements, notifications, analytics
**Estimated effort:** 3-4 weeks | **Impact:** Enhances user experience

### Email Notification System
- [ ] Set up email service integration (SMTP configuration in .env: SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS)
- [ ] Create notification_preferences table (id, user_id, event_type, email_enabled, created_at)
- [ ] Event types: new_announcement, vote_closing_soon, maintenance_status_changed, maintenance_comment_added, payment_due, payment_overdue
- [ ] Implement email templates (Handlebars or Tera):
  - [ ] New announcement notification
  - [ ] Vote closing soon reminder (24h before close_time)
  - [ ] Maintenance status changed
  - [ ] Maintenance comment added
  - [ ] Payment due reminder (7 days before due_date)
  - [ ] Payment overdue notice
- [ ] Add notification triggers in backend (background job or inline after mutations)
- [ ] Create notification preferences UI for users (checkboxes per event type)

### Additional Features
- [ ] Add announcement comments/discussions (similar to maintenance comments)
- [ ] Create audit log system:
  - [ ] audit_log table (id, user_id, entity_type, entity_id, action, old_value, new_value, created_at)
  - [ ] Track: building/apartment edits, owner assignments, role changes, all mutations
  - [ ] Admin-only page to view audit trail with filtering
- [ ] Implement data export:
  - [ ] Apartments CSV (number, building, size, owners, renters)
  - [ ] Owner contacts CSV (name, email, phone, apartments)
  - [ ] Payment history CSV (date range filter)
- [ ] Improve mobile responsiveness:
  - [ ] Test on mobile devices (sidebar collapses to hamburger menu)
  - [ ] Optimize table layouts for small screens (horizontal scroll or stack)
  - [ ] Touch-friendly controls (larger buttons, tap targets)
- [ ] Add water meter usage charts and analytics:
  - [ ] Line charts: daily/weekly/monthly/yearly usage
  - [ ] Period comparisons (current month vs last month/year)
  - [ ] Usage statistics (avg daily, total, min/max)
  - [ ] Handle meter replacement discontinuities (detect serial number change, segment data)
  - [ ] PDF report generation for usage summaries
- [ ] Usage alerts/notifications:
  - [ ] Configurable thresholds per meter type
  - [ ] Email/in-app alert when usage exceeds threshold

---

## COMPLETED FEATURES ✅

### Repo & Tooling
- [x] Create initial repo structure (api/, frontend/, docs/, locales/)
- [x] Add i18n scaffolding (backend + frontend placeholders)
- [x] Add scripts: scripts/dev.sh (runs API + FE), scripts/test.sh (fmt/clippy/test + trunk build)
- [x] CI: GitHub Actions building backend and frontend
- [x] Docker files for backend
- [x] Seed script for local dev (./scripts/seed.sh with test users)

### Backend (Actix + Diesel)
- [x] Health endpoint with i18n message
- [x] Users basic CRUD (list, create)
- [x] Buildings: list/create, Diesel models + migrations, soft-delete
- [x] Apartments: list/create, list by building, Diesel models + migrations, soft-delete
- [x] RBAC roles and permissions enforcement (AuthContext, route guards)
- [x] Authentication (JWT) with registration/login endpoints
- [x] Maintenance Requests: full system with enriched responses
  - [x] Models, endpoints (create/list with enriched data)
  - [x] Status update + comprehensive history audit (status, priority, assignment changes)
  - [x] Attachments CRUD (upload/download/delete/restore) with size/MIME validation
  - [x] Assignment endpoints (assign/unassign with FK validation)
  - [x] RBAC refined (creator/assigned/apartment owner checks)
- [x] Announcements: create, list, pin, comments (Admin/Manager roles)
- [x] Voting system: full implementation
  - [x] Proposals, votes, results tables
  - [x] Voting methods: SimpleMajority, WeightedArea, PerSeat, Consensus
  - [x] Vote choices: Yes, No, Abstain
  - [x] Proposal statuses: Scheduled, Open, Closed, Tallied
  - [x] RBAC: Admin/Manager create/tally, eligible users vote
- [x] Water Meter Reading System: full implementation
  - [x] Meters, meter_readings, webhook_api_keys tables
  - [x] Multiple meter types per apartment (ColdWater, HotWater, Gas, Electricity)
  - [x] Webhook integration with API key auth
  - [x] Manual reading entry, CSV export
  - [x] Calibration tracking, meter replacement workflow
  - [x] RBAC: Admin/Manager manage, Owners/Renters view (is_visible_to_renters toggle)

### Database (MySQL + Diesel)
- [x] Users, roles, user_roles migrations
- [x] Buildings, apartments migrations (with is_deleted for soft-delete)
- [x] Apartment owners join table (apartment_owners)
- [x] Maintenance-related tables (maintenance_requests, maintenance_request_attachments, maintenance_request_history, assigned_to column)
- [x] Announcements tables (announcements, announcement_comments)
- [x] Voting tables (proposals, votes, proposal_results)
- [x] Water meter tables (meters, meter_readings, webhook_api_keys)

### Frontend (Yew + Bootstrap)
- [x] App bootstrap and router
- [x] Home page (announcements feed)
- [x] Buildings page (list/create)
- [x] Apartments page per building (list/create)
- [x] Integrate Bootstrap 5 via CDN and set up base layout (Navbar, Footer)
- [x] Refactor into modular components/pages structure (components/, pages/, routes.rs)
- [x] Login via Navbar dropdown with inline Register
- [x] AuthContext: centralized auth state provider with localStorage sync
- [x] API Service Layer: typed HTTP client (api_client) with automatic token injection
- [x] Reusable components: ErrorAlert, SuccessAlert, Spinner, AdminSidebar
- [x] Manager page (admin sidebar, properties management)
- [x] Maintenance requests pages: list, detail with history, new request form
- [x] Voting pages: list proposals, view detail & cast vote, create proposal
- [x] Water meters page: unified meter management with tabs (list, registration, detail with readings)

### Documentation
- [x] docs/design.md: initial design
- [x] .github/copilot-instructions.md: align with current repo layout
- [x] README: add scripts usage and local dev notes
- [x] CLAUDE.md: comprehensive project overview and architecture guide

---

## TESTING & QUALITY (Ongoing)

### Backend Tests (Pending)
- [ ] Maintenance request tests: create/list/update status, attachment constraints (size/type), history entries, assignment workflow
- [ ] Backend test: soft-delete & restore buildings endpoints
- [ ] Backend test: soft-delete & restore apartments endpoints
- [ ] Backend test: RBAC guard denies non-Admin/Manager access to create/delete/restore
- [ ] Backend test: apartment owner assignment add/remove idempotency
- [ ] Backend test: public users listing limited fields (no password hash exposure)
- [ ] Backend test: maintenance attachment upload rejects >10MB or disallowed MIME
- [ ] Backend test: maintenance request create/status transition saves history
- [ ] Add guard unit tests for AuthContext.has_any_role
- [ ] Create test fixtures helper (create_building(), create_apartment(), assign_owner())
- [ ] Add cargo integration test harness with test DB (transactions rolled back per test)

### Frontend Tests (Pending)
- [ ] Frontend test: render Manager page with mocked data using wasm-bindgen-test
- [ ] Frontend test: ensure restore removes item from deleted list immediately

### Performance & Optimization
- [ ] Add index on buildings.is_deleted & apartments.is_deleted for faster filtered queries
- [ ] Frontend: debounce user search input (250ms delay)
- [ ] Frontend: pagination or virtual scroll for large user lists in owner assignment panel
- [ ] Frontend: optimistic UI update for delete/restore (revert on failure)
- [ ] Clippy pass: remove unused mut warnings

### Code Quality
- [ ] Add CI job to run new backend + frontend tests separately
- [ ] Unit/integration tests for all API modules
- [ ] Document soft-delete conventions in design.md
- [ ] Update design doc with all implemented endpoints and data models

### Accessibility & UX
- [ ] Frontend: accessibility improvements (modal focus trap, aria-labels on buttons)
- [ ] Frontend: Navbar user dropdown shows role badges (Admin/Manager)
- [ ] Frontend: Admin page for user management (enable/disable users, assign/remove roles)

---

## FUTURE CONSIDERATIONS (Not Prioritized)

- [ ] Messaging: WebSocket endpoint and/or REST API for direct messages
- [ ] Visitors/Access logs: track building access, visitor registration
- [ ] Analytics dashboard: usage statistics, trends, reports
- [ ] Events/Calendar: schedule meetings, reserve common spaces
- [ ] Mobile app (native or PWA)
- [ ] SMS/Push notifications (in addition to email)
- [ ] Payment gateway integration (Stripe, bank transfer APIs)
- [ ] Redis caching layer for latest meter readings

---

## NOTES & ARCHITECTURE DECISIONS

### Multi-HOA Architecture
- System supports multiple HOAs via building-scoped access control
- RBAC roles are global, but permissions filtered by building_id
- Users can belong to multiple buildings (as owner, renter, or manager)
- Building access derived from: apartment_owners, apartment_renters, building_managers
- Voting proposals scoped to building_id (only users with access to that building can vote)

### Voting Logic (Multiple Owners)
- Multiple owners allowed per apartment (apartment_owners many-to-many)
- When multiple owners vote on same proposal: last vote wins (overwrites previous owner's vote for that apartment)
- Vote weight determined by proposal's voting_method (SimpleMajority, WeightedArea, PerSeat, Consensus)

### Manager Permissions
- Managers are building-scoped (building_managers join table)
- Manager role users only see buildings they're assigned to
- One manager can manage multiple buildings
- Multiple managers can be assigned to one building

### Maintenance Escalation Workflow
- Renter submits request → initially visible to apartment owners
- Owner can escalate (reassign) to building manager
- Manager handles or reassigns to specific maintenance personnel

### Payment Structure
- Monthly fees configurable per building with multiple calculation methods:
  - Per apartment (flat fee)
  - Per square meter (area-based)
  - Per person (occupant count)
  - Usage-based (meter readings)
  - Fixed amount (custom)
- Special assessments: one-time charges, can target all or specific apartments
- Invoice generation: aggregate fees per apartment for billing period

### Document Visibility
- Documents have building_id scope
- Visibility levels: all_residents, owners_only, specific_apartments, managers_only, admin_only
- Examples:
  - House rules: all_residents
  - Financial reports: owners_only
  - Manager contracts: managers_only
  - Legal documents: admin_only

### Notification Preferences
- User-configurable per event type (email on/off)
- Start with email notifications only (SMTP)
- Future: SMS, push notifications (requires additional infrastructure)

### Technical Patterns
- Soft-delete: Set is_deleted=true, provide restore endpoints, filter active queries
- Diesel ORM: Avoid Model::as_select() in joins with nullable FKs (manual field selection)
- Foreign key validation: Always validate user IDs exist before inserting to avoid FK constraint violations
- File storage: Local filesystem under STORAGE_DIR (future: S3/object storage)
- JWT auth: Token in localStorage, AuthContext provider, automatic API client injection
- Error handling: AppError enum → HTTP status codes, user-friendly messages

### Code Style
- Rust: async/await, Result<T, AppError>, snake_case functions, CamelCase types
- RBAC: Centralized checks in handlers via AuthContext
- Frontend: Use api_client() from services/api.rs (never raw reqwasm)
- Testing: Transaction rollback per test, RBAC matrix tests, attachment validation tests
