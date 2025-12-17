# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

House Management System is a web application for managing residential properties (Homeowners Associations, HOAs). Built with Rust on both backend (Actix-web) and frontend (Yew/WebAssembly), using MySQL + Diesel ORM.

**Key architecture**: Monorepo with `/api` (backend) and `/frontend` (Yew SPA), sharing Rust ecosystem. Role-based access control (RBAC) with JWT authentication. Soft-delete pattern for buildings and apartments.

## Development Commands

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

```bash
# Run all checks: format, clippy, tests, and build
./scripts/test.sh

# Backend only
cd api
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# Frontend build check
cd frontend
rustup target add wasm32-unknown-unknown  # first time only
trunk build
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

**Auth flow**:
- JWT stored in browser localStorage via AuthContext provider
- AuthContext extracts user info from JWT payload (no separate /me endpoint needed)
- API client automatically includes `Authorization: Bearer <token>` header

**State management**:
- Auth state: Centralized via AuthContext provider
- Component state: Yew hooks (`use_state`, `use_effect`) and props
- API calls: Always use `api_client(token)` from services module

**i18n**: Fluent-based translations; locale files in `frontend/locales/` and `api/locales/`; detect browser language or allow manual selection

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
- Maintenance requests: submission, status tracking (Open/InProgress/Resolved), audit history
- Announcements: create, list, pin, comments (Admin/Manager roles)
- Health check endpoint (`/api/v1/health`) with i18n

### Upcoming Features (Partially Implemented)

**Maintenance Requests**:
- Tables: `maintenance_requests`, `maintenance_request_attachments`, `maintenance_request_history`
- File uploads via multipart (`POST /api/v1/requests/{id}/attachments`); stored under `STORAGE_DIR` (configurable in `AppConfig`)
- Constraints: max 10MB, allowed MIME types: `image/*`, `application/pdf`
- Status changes logged to history table with audit trail

**Voting System** (planned):
- Weight strategies: PerSeat (1 vote each), ByApartmentSize (weight by `size_sq_m`), Custom (override table)
- Tables: `proposals`, `votes`, `proposal_custom_weights`
- Result calculation: aggregate weights, simple majority initially

## Code Style and Conventions

**Rust preferences**:
- Use async/await for all handlers; avoid blocking calls
- Prefer `Result<T, AppError>` for error handling
- Module comments with `///` for public functions
- Follow standard Rust naming: snake_case for functions/variables, CamelCase for types

**RBAC enforcement**: Centralized via `AuthContext` checks in handlers; avoid inline role strings, prefer constants or helper functions

**Error responses**: Return JSON with meaningful messages; use appropriate HTTP status codes (400 bad request, 401 unauthorized, 403 forbidden, 404 not found, 500 internal error)

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
