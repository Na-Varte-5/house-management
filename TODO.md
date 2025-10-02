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
- [ ] Maintenance Requests: models, endpoints, status workflow
- [ ] Financials: bills, payments, reports
- [ ] Events/Calendar: models, CRUD, filters
- [ ] Voting: proposals, votes, results
- [ ] Documents: upload/retrieve (multipart), storage
- [ ] Messaging: WebSocket endpoint and/or REST
- [ ] Visitors/Access logs
- [ ] Analytics endpoints
- [ ] Unit/integration tests for API modules

## Database (MySQL + Diesel)
- [x] Users, roles, user_roles migrations
- [x] Buildings, apartments migrations
- [x] Apartment owners join table (apartment_owners)
- [ ] Maintenance-related tables
- [ ] Financial tables: bills, payments, accounts
- [ ] Events tables
- [ ] Voting tables (proposals, votes)
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

## Documentation
- [x] docs/design.md: initial design
- [ ] Update design doc with implemented MVP endpoints and data model details
- [ ] .github/copilot-instructions.md: align with current repo layout and chosen libs
- [ ] README: add scripts usage and local dev notes

## Notes
- Keep modules/components small. Split handlers by domain and avoid monolithic files.
- Prefer async DB operations via thread pool (r2d2 is already set up).
- Next priority suggested: Frontend auth (login), global state, and i18n; then maintenance module.
