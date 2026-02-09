# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

House Management System is a web application for managing residential properties (Homeowners Associations, HOAs). Built with Rust on both backend (Actix-web) and frontend (Yew/WebAssembly), using MySQL + Diesel ORM.

**Key architecture**: Monorepo with `/api` (backend) and `/frontend` (Yew SPA), sharing Rust ecosystem. Role-based access control (RBAC) with JWT authentication. Soft-delete pattern for buildings and apartments.

## Development Commands

### Test Users

**Available test credentials** (created via seed script):
- `admin@example.com` / `password123` (Admin)
- `manager@example.com` / `password123` (Manager)
- `owner1@example.com` / `password123` (Homeowner)
- `owner2@example.com` / `password123` (Homeowner)
- `renter1@example.com` / `password123` (Renter)

**To seed the database:**
```bash
./scripts/seed.sh
```

**Note:** The first user to register via `/api/v1/auth/register` automatically becomes Admin. Subsequent users default to Homeowner role and must be assigned other roles by an Admin via `POST /api/v1/users/{id}/roles`.

### Running the Application

```bash
# Run backend only (requires MySQL configured in api/.env)
cd api && cargo run

# Run frontend only (requires backend running)
cd frontend && trunk serve

# Run both backend and frontend together
./scripts/dev.sh
```

**Environment setup**: Before first run, create `api/.env` with `DATABASE_URL=mysql://username:password@localhost/house_management`, then run `cd api && diesel setup && diesel migration run`.

### Testing and Quality Checks

**IMPORTANT:** When testing if code compiles (without running the app), use `trunk build` for frontend or `cargo build` for backend. Do NOT use `./scripts/dev.sh` for build testing - it runs servers in background and is meant for development/manual testing only.

```bash
# Run all checks: format, clippy, tests, and build
./scripts/test.sh

# Backend only
cd api
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# Frontend build check (preferred for testing compilation)
cd frontend
rustup target add wasm32-unknown-unknown  # first time only
trunk build  # Use this to test if frontend compiles, NOT dev.sh
```

## Architecture and Patterns

### Backend Structure (`/api`)

**Module organization**: `main.rs` mounts API scopes; domain modules (`users/`, `buildings/`, `apartments/`, `maintenance/`, `announcements/`, `auth/`) contain handlers, models, and business logic.

**Authentication flow**:
- JWT-based with `JwtKeys` from `JWT_SECRET` env var
- `AuthContext` extractor validates Bearer tokens and extracts user claims
- Use `auth_ctx.require_roles(&["Admin", "Manager"])` to enforce RBAC in handlers
- User roles stored in JWT claims; roles checked via `AuthContext.has_any_role()`

**Database patterns**:
- Diesel ORM with MySQL; schema in `src/schema.rs` (auto-generated via `diesel print-schema`)
- Migrations in `api/migrations/`; run automatically on server start
- **Soft-delete convention**: Entities have `is_deleted` BOOL; DELETE endpoints set flag to `true`, separate `/deleted` list endpoints and `/restore` POST endpoints
- Active queries MUST filter `is_deleted = false`

**API design**:
- All routes under `/api/v1/` scope
- JSON request/response bodies
- Error handling via `AppError` enum (maps to HTTP status codes)
- Async handlers with `actix_web::web::Data` for shared state (DbPool, JwtKeys, AppConfig)

**RBAC (Role-Based Access Control) - CRITICAL SECURITY REQUIREMENT**:

**MANDATORY: ALL API endpoints MUST implement proper authentication and authorization checks!**

This is a security-critical requirement. Past security audits found critical vulnerabilities where endpoints exposed sensitive data (including password hashes) to unauthenticated users.

**Checklist for EVERY new API endpoint:**

1. ✅ **Add `AuthContext` parameter** - Unless endpoint is explicitly public (login/register)
2. ✅ **Add role check** - Use `auth.has_any_role(&["Admin", "Manager"])` or equivalent
3. ✅ **Add to OpenAPI docs** - Include `security(("bearer_auth" = []))` in `#[utoipa::path]`
4. ✅ **Document requirements** - State which roles can access in doc comment
5. ✅ **Return proper errors** - Use `AppError::Forbidden` for authorization failures

**Common RBAC Patterns:**

```rust
// Pattern 1: Admin-only endpoint
pub async fn admin_only_endpoint(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }
    // ... endpoint logic
}

// Pattern 2: Admin or Manager endpoint
pub async fn privileged_endpoint(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }
    // ... endpoint logic
}

// Pattern 3: Resource ownership check (user can only access their own data)
pub async fn get_my_resource(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let resource_id = path.into_inner();
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);

    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Fetch resource
    let resource = /* load from database */;

    // Check ownership or admin/manager privilege
    if resource.user_id != user_id && !auth.has_any_role(&["Admin", "Manager"]) {
        return Err(AppError::Forbidden);
    }

    // ... endpoint logic
}

// Pattern 4: Building/apartment access control (scoped by ownership)
pub async fn building_scoped_endpoint(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let building_id = path.into_inner();
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;

    // Check if user has access to this building
    let is_admin = auth.has_any_role(&["Admin", "Manager"]);
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);

    use crate::auth::building_access::get_user_building_ids;
    let maybe_building_ids = get_user_building_ids(user_id, is_admin, &mut conn)?;

    if let Some(accessible_buildings) = maybe_building_ids {
        if !accessible_buildings.contains(&building_id) {
            return Err(AppError::Forbidden);
        }
    }

    // ... endpoint logic
}

// Pattern 5: Authenticated but no specific role required
pub async fn any_authenticated_user(
    auth: AuthContext,  // Just having AuthContext means auth is required
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    let user_id = auth.claims.sub.parse::<u64>().unwrap_or(0);
    // All authenticated users can access this
    // ... endpoint logic
}
```

**OpenAPI Documentation Pattern:**

```rust
/// Endpoint description
///
/// Detailed description of what this endpoint does.
/// IMPORTANT: Document which roles can access this endpoint!
#[utoipa::path(
    get,
    path = "/api/v1/resource",
    responses(
        (status = 200, description = "Success", body = Resource),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - requires Admin role"),
        (status = 500, description = "Internal server error")
    ),
    tag = "ResourceTag",
    security(("bearer_auth" = []))  // <-- CRITICAL: Always add this!
)]
pub async fn endpoint_handler(
    auth: AuthContext,  // <-- CRITICAL: Always add this (unless public endpoint)!
    // ... other parameters
) -> Result<impl Responder, AppError> {
    // ... implementation
}
```

**Exceptions (Public Endpoints):**

Only these endpoint types should NOT require authentication:
- `POST /api/v1/auth/register` - User registration
- `POST /api/v1/auth/login` - User login
- `GET /api/v1/health` - Health check
- Webhook endpoints that use API key authentication instead of JWT

**Common Mistakes to Avoid:**

❌ **DON'T** create endpoints without `AuthContext` parameter
❌ **DON'T** expose user lists without Admin role check (exposes password hashes!)
❌ **DON'T** allow unrestricted access to building/apartment data
❌ **DON'T** forget to add `security(("bearer_auth" = []))` to OpenAPI docs
❌ **DON'T** return sensitive data to unauthorized users

✅ **DO** always add `AuthContext` parameter
✅ **DO** implement role checks appropriate for the endpoint's sensitivity
✅ **DO** use building access scoping for building/apartment endpoints
✅ **DO** return `AppError::Forbidden` for authorization failures
✅ **DO** document role requirements in endpoint doc comments

**Security Audit History:**

- **2026-01-24**: Fixed 6 critical vulnerabilities where endpoints lacked authentication
  - `GET /api/v1/users` - Exposed ALL users including password hashes (CRITICAL)
  - `GET /api/v1/buildings` - Unauthenticated access to all buildings
  - `GET /api/v1/buildings/{id}` - Unauthenticated building details
  - `GET /api/v1/apartments` - Unauthenticated apartment list
  - `GET /api/v1/buildings/{id}/apartments` - Unauthenticated apartment access
  - `GET /api/v1/apartments/{id}/owners` - Privacy violation (ownership exposure)

**Key RBAC roles**: Admin, Manager, Homeowner, Renter, HOA Member
- Admin/Manager: create/delete/restore buildings, apartments; assign owners; update maintenance status
- Homeowner/Renter: submit maintenance requests, view own apartments
- Enforcement: `AuthContext::require_roles()` in handlers

### Frontend Structure (`/frontend`)

**Framework**: Yew (Rust WebAssembly) with yew-router for SPA routing
- Components in `src/components/` (reusable: navbar, sidebar, spinners, auth dropdown, error/success alerts)
- Pages in `src/pages/` (routes map to page components)
- Routes defined in `src/routes.rs`; rendered in `src/app.rs` via `<Switch<Route>>`

**Styling**: Bootstrap CSS (CDN-linked in `index.html`); use Bootstrap classes for responsive layout

**Architecture (REFACTORED - Dec 2025)**:
- **AuthContext** (`src/contexts/auth.rs`): Centralized auth state provider with automatic localStorage sync
  - Exposes: `token()`, `user()`, `is_authenticated()`, `has_role()`, `is_admin_or_manager()`
  - All pages use `use_context::<AuthContext>()` instead of direct localStorage access
- **API Service Layer** (`src/services/api.rs`): Typed HTTP client with automatic token injection
  - `api_client(token)` returns configured client with base URL detection
  - Typed errors: `ApiError` enum (NetworkError, Unauthorized, Forbidden, NotFound, BadRequest, ServerError)
  - Auto-detects environment: port 8081 (Trunk dev) → routes to 8080 (backend)
- **Reusable Components**: ErrorAlert, SuccessAlert for consistent user feedback
- **Form Components Library** (`src/components/forms/`): Comprehensive reusable form inputs (Jan 2026)
  - **IMPORTANT**: Always use these components instead of raw HTML inputs for consistency
  - Available components: TextInput, NumberInput, Select, Textarea, DateTimeInput, Checkbox, FormGroup
  - Features: built-in validation, labels, help text, error display, size variants (sm/lg)
  - Full documentation in `frontend/CLAUDE.md` with usage examples
  - Example: `<TextInput label="Email" value={email} on_change={on_email_change} required=true />`

**Auth flow**:
- JWT stored in browser localStorage via AuthContext provider
- AuthContext extracts user info from JWT payload (no separate /me endpoint needed)
- API client automatically includes `Authorization: Bearer <token>` header

**State management**:
- Auth state: Centralized via AuthContext provider
- Component state: Yew hooks (`use_state`, `use_effect`) and props
- API calls: Always use `api_client(token)` from services module

**i18n (MANDATORY - read carefully)**:
- Fluent-based translations; locale files in `frontend/locales/` and `api/locales/`; detect browser language or allow manual selection
- Translation module: `frontend/src/i18n.rs` provides `t(key: &str) -> String` and `t_with_args(key: &str, args: &[(&str, &str)]) -> String`
- **CRITICAL: When adding ANY new user-facing string to the frontend, you MUST:**
  1. Add the translation key to BOTH `frontend/locales/en/frontend.ftl` AND `frontend/locales/cs/frontend.ftl`
  2. Use `t("key")` or `t_with_args("key", &[("var", &val)])` instead of hardcoded English strings
  3. Reuse existing keys when possible — check the FTL files first before creating new ones
  4. Use kebab-case with section prefixes for new keys (e.g., `maintenance-title`, `meters-serial-number`)
  5. Keep both FTL files in sync — never add a key to one without adding it to the other
- **What gets translated**: Display text in RSX (`{"text"}`), component props (`label="text"`), error/success messages shown to users
- **What stays in English**: Data/enum values used for filtering/matching (e.g., status strings compared against API responses), CSS class names, API field names
- **Pattern examples**:
  - RSX text: `{"Loading..."}` → `{t("common-loading")}`
  - Props: `label="Email"` → `label=t("login-email")`
  - With args: `{t_with_args("error-load", &[("error", &e.to_string())])}`
  - FTL format: `error-load = Failed to load: {$error}`
- Failing to add i18n entries when adding new strings will cause the app to display raw translation keys to users

**API communication**:
- **ALWAYS** use `api_client(token)` from `src/services/api.rs`
- **NEVER** use raw `reqwasm` calls directly
- Methods: `get<T>()`, `post<T, R>()`, `put<T, R>()`, `delete()`, `delete_no_response()`, `post_empty<T>()`

### Database Migrations

**Creating migrations**:
```bash
cd api
diesel migration generate <name>  # creates up.sql and down.sql
# Edit up.sql with schema changes, down.sql with rollback
diesel migration run              # apply pending migrations
```

**Migration conventions**:
- Timestamp-prefixed directories in `api/migrations/`
- Use `DEFAULT false` for `is_deleted` columns
- Add indexes for foreign keys and frequently filtered columns (e.g., `is_deleted`)

### Implemented Features

- User management with RBAC (roles: Admin, Manager, Homeowner, Renter, HOA Member)
- JWT authentication (login via `/api/v1/login`)
- Buildings and apartments CRUD with soft-delete
- Apartment-owner assignment (many-to-many via `apartment_owners` join table)
- Manager page UI: create buildings/apartments, assign owners, show deleted toggle, restore
- **Maintenance Requests System**: Full-featured request tracking with enriched data and comprehensive audit history
  - Tables: `maintenance_requests`, `maintenance_request_attachments`, `maintenance_request_history`
  - **Enriched API responses**: All endpoints return apartment numbers, building addresses, and user names (not just IDs)
    - `GET /api/v1/requests` returns `MaintenanceRequestEnriched` with apartment/building context
    - `GET /api/v1/requests/{id}` returns `MaintenanceRequestDetail` with full user names for creator and assignee
    - `PUT /api/v1/requests/{id}` returns enriched detail after updates
    - `GET /api/v1/requests/{id}/history` returns `MaintenanceRequestHistoryEnriched` with user names
  - **Comprehensive audit history**: All changes (status, priority, assignment) logged to history table
    - Status changes: "Changed status from Open to InProgress"
    - Priority changes: "Priority changed from Medium to High"
    - Assignment changes: "Assigned to [User Name]" or "Reassigned from [Old User] to [New User]"
    - All history entries include user name and formatted timestamp
  - **Frontend improvements**: User-friendly display with proper names and formatted dates
    - Dates formatted as "Jan 14, 2026 at 10:30" instead of raw timestamps
    - Dropdown defaults show current values (status, priority, assigned user)
    - All users (not just Admin/Manager) see proper names in history
  - File uploads via multipart (`POST /api/v1/requests/{id}/attachments`); stored under `STORAGE_DIR` (configurable in `AppConfig`)
  - Constraints: max 10MB, allowed MIME types: `image/*`, `application/pdf`
  - **Important**: Manual field selection in join queries to avoid Diesel deserialization issues with nullable foreign keys
  - RBAC: Admin/Manager can update all fields; Homeowner/Renter can only view their own requests
- Announcements: create, list, pin, comments (Admin/Manager roles)
- **Voting System**: proposal creation, voting with weighted methods, result tallying
  - Tables: `proposals`, `votes`, `proposal_results`
  - Voting methods: `SimpleMajority` (yes > no), `WeightedArea` (weight by apartment size), `PerSeat` (1 vote each), `Consensus` (requires unanimous approval)
  - Vote choices: Yes, No, Abstain
  - Proposal statuses: Scheduled, Open, Closed, Tallied
  - Eligibility: Role-based (Admin, Manager, Homeowner, Renter, HOA Member) - configurable per proposal
  - RBAC: Only Admin/Manager can create proposals and tally results; eligible users can vote
  - Frontend: List proposals (`/voting`), view details & cast vote (`/voting/:id`), create proposal (`/voting/new`)
  - Backend API: `GET /api/v1/proposals`, `POST /api/v1/proposals`, `GET /api/v1/proposals/:id`, `POST /api/v1/proposals/:id/vote`, `POST /api/v1/proposals/:id/tally`
- Health check endpoint (`/api/v1/health`) with i18n
- **Water Meter Reading System**: Remote meter readings with webhook integration for automated data ingestion
  - Tables: `meters`, `meter_readings`, `webhook_api_keys`
  - Support for multiple meter types per apartment: ColdWater, HotWater, Gas, Electricity
  - Webhook integration with per-integration API key authentication (stored hashed in database)
  - Manual reading entry as fallback (Admin/Manager only)
  - CSV export of readings with date range filtering
  - Calibration tracking: due date monitoring, color-coded status badges (overdue/due soon/valid)
  - Meter replacement workflow: update serial number while preserving all historical readings (same meter ID)
  - RBAC: Admin/Manager can register/edit meters and add readings; Owners/Renters can view meters (with `is_visible_to_renters` toggle)
  - Frontend: Unified meter management page (`/admin/meters`) with tabs for list view and registration form
  - Meter detail page with reading history table, manual entry form, edit/replace meter form, CSV export
  - Backend API: Full CRUD for meters, readings, and webhook API keys
  - **Note**: Physical meter replacement (calibration) causes reading value reset to 0; charts/analytics (not yet implemented) will need to handle discontinuities

### Upcoming Features

**Water Meter Analytics & Visualization** (planned):
- Historical usage charts (line graphs): daily, weekly, monthly, yearly aggregations
- Period comparisons: compare current month to previous month/year
- Usage statistics: average daily usage, total usage per period, min/max readings
- Handle meter replacement discontinuities: detect serial number changes, calculate consumption deltas, segment/normalize data
- Meter replacement event tracking: dedicated table to record old serial number and final reading at replacement time
- PDF report generation for usage summaries
- Usage alerts/notifications: email/in-app alerts for unusual consumption (configurable thresholds)
- Caching layer (Redis) for latest readings to optimize read-heavy queries
- Yearly billing tally export with all apartments and annual totals

## Code Style and Conventions

**Rust preferences**:
- Use async/await for all handlers; avoid blocking calls
- Prefer `Result<T, AppError>` for error handling
- Module comments with `///` for public functions
- Follow standard Rust naming: snake_case for functions/variables, CamelCase for types

**RBAC enforcement**: Centralized via `AuthContext` checks in handlers; avoid inline role strings, prefer constants or helper functions

**Error responses**: Return JSON with meaningful messages; use appropriate HTTP status codes (400 bad request, 401 unauthorized, 403 forbidden, 404 not found, 500 internal error)

**Frontend form development**:
- Always use form components from `frontend/src/components/forms/` instead of raw HTML inputs (see `frontend/CLAUDE.md` for full component library documentation)
- If a form input pattern is missing, create a new reusable component in `frontend/src/components/forms/` for future reuse instead of implementing it inline

**Frontend architecture principles (Yew/Rust)**:
- **Single Responsibility Principle (CRITICAL)**: Each component should have ONE clear purpose
  - ❌ BAD: A 750-line page component that handles both registration forms AND list views with filters
  - ✅ GOOD: Split into separate components: orchestrator page (145 lines) + register form component (323 lines) + list component (363 lines)
- **Component size limits**:
  - Page components (routes): Max ~200 lines. If larger, split into sub-components
  - Reusable components: Max ~400 lines. If larger, split into smaller components
  - If a file exceeds these limits, it likely violates Single Responsibility Principle
- **When to split components**:
  - Component has multiple distinct responsibilities (e.g., form + list + filters)
  - Component handles multiple unrelated concerns (e.g., registration + editing + deletion)
  - File is growing beyond 300-400 lines
  - Logic could be reused elsewhere
  - Testing would benefit from isolation
- **Component organization**:
  - Page components (routes) in `src/pages/` - orchestrate child components, handle routing
  - Reusable components in `src/components/` - focused, testable, reusable
  - Domain-specific components in `src/components/domain/` (e.g., `meters/`, `properties/`)
  - Form inputs in `src/components/forms/` - standardized, validated inputs
- **Yew best practices**:
  - Use props for data, callbacks for events
  - Keep state management close to where it's used
  - Use `use_effect_with` for side effects with explicit dependencies
  - Always handle loading and error states
  - Provide empty state feedback
  - Use Bootstrap classes for styling (no custom CSS)

**Testing strategy** (planned):
- Backend integration tests with test database (transaction rollback per test)
- RBAC matrix tests (verify denial for unauthorized roles)
- Attachment validation tests (size/MIME constraints)
- Soft-delete query tests

**Avoid**:
- Hard deletes (use soft-delete pattern)
- Storing sensitive data in JWT (only user ID and roles)
- Blocking database calls (always use async)
- Exposing internal error details to API responses
- Using `Model::as_select()` in join queries with nullable foreign keys (causes "could not convert slice to array" errors)

**Diesel ORM patterns**:
- **Issue with joins and nullable FKs**: When using `as_select()` with joined queries, Diesel can fail to deserialize nullable foreign key fields
- **Solution**: Manually select individual fields in join queries instead of using `as_select()`
  ```rust
  // BAD - can cause deserialization errors with nullable FKs
  let result: (MaintenanceRequest, String) = maintenance_requests
      .inner_join(apartments)
      .select((MaintenanceRequest::as_select(), apartments::number))
      .first(&mut conn)?;

  // GOOD - explicitly select each field
  let result: (u64, u64, Option<u64>, String) = maintenance_requests
      .inner_join(apartments)
      .select((mr::id, mr::apartment_id, mr::assigned_to, apt::number))
      .first(&mut conn)?;
  ```
- **Foreign key validation**: Always validate user IDs exist in database before inserting into history tables to avoid FK constraint violations

## Important Implementation Notes

**Soft-delete pattern**: When deleting entities, set `is_deleted = true` instead of removing rows. Provide separate endpoints for listing deleted entities and restoring them. All active queries MUST include `WHERE is_deleted = false`.

**Owner assignment**: `apartment_owners` join table; add endpoint is idempotent (ignores duplicates); remove endpoint cascades gracefully (no error if not found).

**File attachments**: Store metadata in database; physical files on disk under `STORAGE_DIR/<entity_type>/<entity_id>/<uuid>`. Soft-delete metadata first; physical cleanup in background job.

**Authentication middleware**: JWT validation happens in `AuthContext::from_request`; extracted automatically by Actix when used as handler parameter.

**Database connection pool**: `DbPool` (Diesel r2d2) injected via `web::Data`; get connection with `pool.get()?` in handlers.

**API versioning**: All routes prefixed with `/api/v1/`; future versions use `/api/v2/` scope.

**Frontend routing**: Yew router uses `#` hash-based routing by default; routes defined in `Route` enum and rendered in `App` component.

**Component reusability**: Extract common UI patterns (spinners, modals, form inputs) into `src/components/`; pass data via props.

**Locale files**: Store translations in `locales/<lang>/<module>.ftl`; use Fluent syntax for pluralization and variables.

**Error handling on frontend**: Display user-friendly error messages; avoid showing raw API error details.

## Configuration

**Backend environment variables** (`api/.env`):
- `DATABASE_URL`: MySQL connection string
- `JWT_SECRET`: Secret key for JWT signing (change in production)
- `API_PORT`: Server port (default: 8080)
- `STORAGE_DIR`: Base path for file uploads (default: `./storage`)

**Frontend build**: Trunk.toml configures proxy to backend during development; `trunk serve` proxies `/api` to `http://127.0.0.1:8080`.

## Copilot/AI Instructions Summary

(From `.github/copilot-instructions.md`):
- Follow Rust best practices; use Yew for frontend, Actix-web for backend
- Implement RBAC with role checks in handlers
- Use JWT for authentication (stored in localStorage on frontend)
- Prefer soft-delete over hard-delete
- Use Diesel migrations for schema changes
- Bootstrap CSS for styling
- Keep translation files for i18n (English and Czech defaults)
- Document all public APIs with `///` comments
- Avoid permanent deletes; test RBAC denial cases
