# House Management System - TODO Tracker

This file tracks progress and planned work. It maps to the design doc and Copilot instructions. Please keep items small and check them off as we complete them.

Legend: [ ] planned, [x] done, [~] in progress, [!] blocked

## Repo & Tooling
- [x] Create initial repo structure (api/, frontend/, docs/, locales/) 
- [x] Add i18n scaffolding (backend + frontend placeholders)
- [x] Add scripts: scripts/dev.sh (runs API + FE), scripts/test.sh (fmt/clippy/test + trunk build)
- [x] CI: GitHub Actions building backend and frontend
- [x] Docker files for backend

## Backend (Actix + Diesel)
- [x] Health endpoint with i18n message
- [x] Users basic CRUD (list, create)
- [x] Buildings: list/create, Diesel models + migrations
- [x] Apartments: list/create, list by building, Diesel models + migrations
- [x] RBAC roles and permissions enforcement (middleware, route guards) (initial enforcement on mutating endpoints)
- [x] Authentication (JWT) with registration/login endpoints
- [x] Maintenance Requests: models, endpoints (create/list), status update + history audit, attachments CRUD (upload/download/delete/restore), assignment endpoints (assign/unassign), RBAC refined (creator/assigned/apartment owner checks)
- [x] Maintenance Requests: status transition validation map PENDING (currently allows any transition)
- [ ] Maintenance Requests tests: create/list/update status, attachment constraints (size/type), history entries, assignment workflow
- [~] Voting: proposals, votes, results, weight strategies (PerSeat, ByApartmentSize, Custom) - tables created, API pending
- [ ] Documents: upload/retrieve (multipart), storage
- [ ] Messaging: WebSocket endpoint and/or REST
- [ ] Visitors/Access logs
- [ ] Analytics endpoints
- [ ] Unit/integration tests for API modules

## Database (MySQL + Diesel)
- [x] Users, roles, user_roles migrations
- [x] Buildings, apartments migrations
- [x] Apartment owners join table (apartment_owners)
- [x] Maintenance-related tables (maintenance_requests, maintenance_request_attachments, maintenance_request_history, assigned_to column)
- [x] Voting tables (proposals, votes, proposal_results)
- [ ] Financial tables: bills, payments, accounts
- [ ] Events tables
- [ ] Documents metadata table
- [ ] Messages table
- [ ] Visitors table

## Frontend (Yew + Bootstrap)
- [x] App bootstrap and router
- [x] Home page displaying health
- [x] Buildings page (list/create)
- [x] Apartments page per building (list/create)
- [x] Integrate Bootstrap 5 via CDN and set up base layout (Navbar, Footer)
- [x] Refactor into modular components/pages structure (components/, pages/, utils/, routes.rs)
- [x] Login via Navbar dropdown with inline Register; auth state persisted in LocalStorage (JWT)
- [x] Fix dropdown closing on Create account (use data-bs-auto-close="outside" + stopPropagation)
- [ ] Global state (user, token, language)
- [ ] i18n implementation and language switcher
- [ ] UI pages for maintenance, financials, events, voting, documents, messaging, visitors
- [ ] Charts (analytics)
- [ ] Maintenance requests page: list, filter by status, create form, attachment upload
- [ ] Voting page: list active/past proposals, vote submission UI

## Documentation
- [x] docs/design.md: initial design
- [ ] Update design doc with implemented MVP endpoints and data model details
- [x] .github/copilot-instructions.md: align with current repo layout and chosen libs
- [x] README: add scripts usage and local dev notes

## Session Follow-up (Soft Delete & Manager Page)
- [ ] Backend test: soft-delete & restore buildings endpoints (list active vs deleted, restore removes from deleted)
- [ ] Backend test: soft-delete & restore apartments endpoints
- [ ] Backend test: RBAC guard denies non-Admin/Manager access to create/delete/restore building/apartment
- [ ] Backend test: apartment owner assignment add/remove idempotency (duplicate add ignored)
- [ ] Backend test: public users listing limited fields (no password hash exposure)
- [x] Frontend: extract spinner into reusable component
- [ ] Frontend: debounce user search input (avoid excessive network calls; 250ms delay)
- [ ] Frontend: pagination or virtual scroll for large user lists in owner assignment panel
- [ ] Frontend: optimistic UI update for delete/restore (revert on failure)
- [ ] Frontend: Admin page for user management (enable/disable users, assign/remove roles)
- [ ] Frontend: Navbar user dropdown shows role badges (Admin/Manager) + quick link to Manager/Admin pages
- [ ] Frontend: accessibility improvements (modal focus trap, aria-labels on delete/restore buttons)
- [ ] Frontend: unify API calls in typed service layer (error mapping, retry for 5xx)
- [ ] Frontend test: render Manager page with mocked data (owners list, spinners) using wasm-bindgen-test
- [ ] Frontend test: ensure restore removes item from deleted list immediately
- [ ] Add index on buildings.is_deleted & apartments.is_deleted for faster filtered queries
- [ ] Clippy pass: remove unused mut warnings (manage.rs)
- [ ] Add cargo integration test harness with test DB (transactions rolled back per test)
- [ ] Seed script for local dev (sample buildings, apartments, users)
- [ ] Add CI job to run new backend + frontend tests separately
- [ ] Document soft-delete conventions in design.md
- [ ] Add guard unit tests for AuthContext.has_any_role
- [ ] Create test fixtures helper (create_building(), create_apartment(), assign_owner())
- [ ] Backend test: maintenance request create/status transition saves history
- [ ] Backend test: maintenance attachment upload rejects >10MB or disallowed mime

## Notes
- Keep modules/components small. Split handlers by domain and avoid monolithic files.
- Prefer async DB operations via thread pool (r2d2 is already set up).
- Next priority suggested: Frontend auth (login), global state, and i18n; then maintenance module.
- Maintenance attachments stored under STORAGE_DIR with UUID filenames; soft-delete flag for metadata (physical file removal deferred)
- Voting weight strategy chosen per proposal; if Custom, populate override table before opening voting window.
- Progress 2025-11-17 (Session 1): Maintenance module backend fully implemented (Phases 1-3 complete except transition validation)
  
  **Phase 1 ✅ COMPLETED - Data & Config Foundation:**
  - Migrations: assigned_to column (nullable FK to users) + voting tables (proposals, votes, proposal_results)
  - Models/Schema: MaintenanceRequest.assigned_to, Proposal/Vote/ProposalResult structs, VotingMethod/VoteChoice/ProposalStatus enums
  - Config module: attachments_base_path, max_attachment_size_bytes (10MB default), allowed_mime_types
  - Error handling: Extended AppError (AttachmentTooLarge, InvalidMimeType, NotFound)
  
  **Phase 2 ✅ COMPLETED - Maintenance Attachments Endpoints:**
  - All 7 attachment endpoints in maintenance/attachments.rs:
    1. POST /requests/{id}/attachments - Upload with multipart
    2. GET /requests/{id}/attachments - List active attachments
    3. GET /requests/{id}/attachments/deleted - List soft-deleted (Admin/Manager only)
    4. GET /requests/{id}/attachments/{att_id} - Get metadata
    5. GET /requests/{id}/attachments/{att_id}/download - Download file
    6. DELETE /requests/{id}/attachments/{att_id} - Soft-delete
    7. POST /requests/{id}/attachments/{att_id}/restore - Restore
  - Upload features: UUID filenames, magic byte mime detection (infer crate), size validation, atomic writes, filename sanitization, per-request directories
  - Download: Correct Content-Type & Content-Disposition headers with original filename
  - RBAC refined with helpers (load_request, user_owns_apartment, compute_perms)
  - View permission: creator OR assigned_to OR apartment owner OR Admin/Manager
  - Modify permission: creator OR assigned_to OR Admin/Manager
  
  **Phase 3 ⚠️ PARTIALLY COMPLETED - Assignment & History:**
  - Assignment endpoints: PUT /assign (validates user exists), DELETE /assign (unassigns)
  - list_requests enhanced: includes requests where user is assigned_to
  - Status update & history audit already functional (from previous work)
  - OUTSTANDING: Status transition validation map (currently allows any transition)
  
  **Technical Details:**
  - Dependencies added: actix-multipart 0.6, uuid 1.0 (v4), infer 0.15, futures-util 0.3
  - Tests: Placeholder api/tests/attachments_upload.rs (ignored); comprehensive tests deferred to Phase 18
  - Frontend: minimal maintenance.rs created but module resolution issue; workaround is integrating into ManagePage
  - Documentation: rest-api.http extended with all maintenance/attachment examples
  - Metrics: ~800 LOC added, 13 endpoints (6 request + 7 attachment), 3 RBAC helpers, 2 migrations
  - Build status: ✅ GREEN (workspace builds successfully)
  
  **API Surface Added:**
  - Maintenance: GET/POST /requests, PUT /status, GET /history, PUT/DELETE /assign
  - Attachments: POST/GET/DELETE upload, list, metadata, download, restore
  
  **Next Steps:**
  - Immediate: Implement status transition validation map, document state machine in design.md
  - Short-term: Phase 4 voting backend, integrate maintenance UI into ManagePage
  - Medium-term: Phase 18 comprehensive testing (attachment constraints, RBAC, assignment workflow)
