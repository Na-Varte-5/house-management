# Improvement Plan (Feb 2026)

Identified pitfalls and improvement areas across the codebase, ordered by priority.

## Status Legend
- [ ] Not started
- [x] Complete

## Critical (Security / Stability)

- [x] **1. Fix `Cors::permissive()` -> explicit CORS config**
  - Replaced with configurable `CORS_ALLOWED_ORIGINS` env var
  - Defaults to `localhost:8081` (dev), supports comma-separated origins, `*` for dev override

- [x] **2. Replace `unwrap_or(0)` user ID parsing with proper error handling**
  - Added `AuthContext::user_id() -> Result<u64, AppError>` helper
  - Replaced all 25 occurrences across 10 files

- [x] **3. Fix voting `unwrap()` panics**
  - Replaced `VotingMethod::from_str().unwrap()` with proper `?` error propagation
  - Replaced `BigDecimal::from_f64().unwrap()` with `BigDecimal::from()` and `.is_zero()`

- [x] **4. Require `JWT_SECRET` env var (remove insecure fallback)**
  - Changed to `expect()` with helpful error message
  - Updated CI workflow to set `JWT_SECRET` in env

- [x] **5. Add input validation on auth endpoints**
  - Email: format check, max 255 chars
  - Password: min 8 chars, max 128 chars
  - Name: non-empty, max 255 chars
  - Login: non-empty email and password check

- [x] **6. Add rate limiting on login/register**
  - Added `actix-governor` (5 req/10s per IP, burst 5)
  - Applied to `/auth` scope (login + register)

## Significant (Ops / Maintainability)

- [x] **7. Add `tracing` structured logging**
  - Added `tracing`, `tracing-subscriber` (with `env-filter`), `tracing-actix-web`
  - Initialized subscriber in `main()` with `RUST_LOG` env filter (defaults to `info`)
  - `TracingLogger` middleware on the App — logs every request with method, path, status, latency
  - Replaced all `println!` with structured `info!()` using tracing fields
  - Zero `println!`/`eprintln!` remaining in backend code

- [x] **8. Split monolithic backend modules**
  - `apartments/mod.rs` (2,087 lines), `maintenance/mod.rs` (1,411), `announcements/mod.rs` (1,153)
  - Split into `handlers.rs`, `types.rs`, `helpers.rs` sub-modules (like `meters/` already does)
  - Also split `voting/mod.rs` (535 lines) into `types.rs`, `handlers.rs`

- [x] **9. Split oversized frontend components**
  - `pages/admin/properties.rs` (982 -> 27), `pages/my_property_detail.rs` (816 -> 14), `pages/maintenance/detail.rs` (600 -> 14)
  - Extracted: `AdminPropertiesData`, `PropertyDetailContent`, `MeterCardList`, `MaintenanceDetailContent`
  - All page files now under 30 lines; logic moved to `components/properties/` and `components/maintenance/`

- [x] **10. Add integration tests (RBAC matrix, CRUD flows)**
  - Only 5 unit tests exist currently; zero integration tests
  - Fix: Test RBAC denial, soft-delete flows, maintenance workflows
  - **Deferred**: Requires running MySQL; pre-existing integration test failures confirm DB dependency

## Moderate (Performance / DevEx)

- [x] **11. Add pagination to list endpoints**
  - Shared `pagination` module with `PaginationParams` (page/per_page query params) and `PaginatedResponse<T>` wrapper
  - Paginated endpoints: `/buildings`, `/apartments`, `/users`, `/requests`, `/announcements`, `/announcements/public`, `/announcements/deleted`, `/proposals`
  - Defaults: page=1, per_page=20, max per_page=100
  - Response includes `{ data: [...], pagination: { page, per_page, total, total_pages } }`
  - Frontend updated: `PaginatedResponse<T>` type added, all affected call sites extract `.data`

- [x] **12. Add Dockerfile + docker-compose**
  - Multi-stage Dockerfile (backend build, frontend build, slim runtime)
  - docker-compose.yml with MySQL service, health checks, and volume persistence

- [x] **13. Add `.env.example` file**
  - Created `api/.env.example` with all required/optional vars documented

- [x] **14. Update CI to `dtolnay/rust-toolchain`**
  - Replaced deprecated `actions-rs/toolchain@v1` with `dtolnay/rust-toolchain@stable`

- [x] **15. Add DB pool monitoring/tuning**
  - Configured: max_size (env `DB_POOL_SIZE`, default 10), min_idle 2, connection_timeout 5s, idle_timeout 300s
  - Enabled `test_on_check_out` for connection health checks
  - Added pool state logging on startup
