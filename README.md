# House Management System

A comprehensive platform for managing residential properties for Homeowners Associations (HOAs) and similar entities.

## Project Overview

The House Management System provides a centralized solution for managing various aspects of residential properties, including:

- User management with different roles and permissions
- Property and common area management
- Financial management with flexible cost calculation
- Event scheduling and calendar management
- Voting system for community decision-making
- Maintenance request and complaint handling

## Recent Changes (Session Log)

The following features were added or refined in the current development session:

- Manager page (Yew) consolidating building & apartment management plus owner assignment.
- Soft-delete support for buildings and apartments (is_deleted flag) with toggle to show deleted items and one-click restore.
- New backend endpoints:
  - `GET /api/v1/buildings/deleted` – list soft-deleted buildings.
  - `POST /api/v1/buildings/{id}/restore` – restore a building.
  - `DELETE /api/v1/buildings/{id}` – soft-delete a building (Admin/Manager).
  - `GET /api/v1/apartments/deleted` – list soft-deleted apartments.
  - `POST /api/v1/apartments/{id}/restore` – restore an apartment.
  - `DELETE /api/v1/apartments/{id}` – soft-delete an apartment (Admin/Manager).
- Owner assignment UI with searchable public user list; click to assign / badge with close button to remove.
- Deletion confirmation modal for buildings/apartments (prevent accidental removal).
- Loading spinners for buildings, apartments, owners, and deleted lists to improve UX feedback.
- Role-based visibility: building and apartment creation restricted to Admin/Manager; other roles see access denied message.
- Immediate in-place refresh after delete/restore (no full page reload required).

### Upcoming (In Progress)
- Maintenance Requests module: request submission (Open/InProgress/Resolved), status updates with audit history, file attachments (images/PDF) stored server-side.
- Voting groundwork: flexible weight strategies (per seat, by apartment size, or custom override table).
- Test harness: backend integration tests for soft-delete/RBAC and new maintenance workflows.

## RBAC Summary
Roles currently recognized: Admin, Manager, Homeowner, Renter, HOA Member.

| Action | Allowed Roles (current enforcement) |
|--------|-------------------------------------|
| Create/Soft-delete/Restore Building | Admin, Manager |
| Create/Soft-delete/Restore Apartment | Admin, Manager |
| Assign/Remove Apartment Owner | Admin, Manager |
| Submit Maintenance Request (planned) | Homeowner, Renter, Admin, Manager |
| Update Maintenance Status (planned) | Admin, Manager |
| Upload Maintenance Attachment (planned) | Request creator, Admin, Manager |

RBAC checks are centralized via `AuthContext.has_any_role` and enforced in handlers. Upcoming tests will assert denial for unauthorized roles.

## Soft Delete & Restoration

Instead of permanently removing records, delete operations set `is_deleted = true`. Active queries filter on `is_deleted = false`. Restoration endpoints flip the flag back to false. This provides:

- Safety against accidental data loss.
- Ability for managers to audit and restore entities quickly.
- Consistency between buildings and apartments life-cycle.

### Client UX

- A “Show Deleted” switch reveals deleted lists rendered separately.
- Restored items disappear from the deleted section and reappear under active lists immediately.
- Spinners indicate ongoing network operations (fetching, delete, restore, owner list refresh).

## Technical Stack

### Backend (API)
- Rust with Actix-web framework
- MySQL database with Diesel ORM
- RESTful API design

### Frontend
- Rust with Yew framework (WebAssembly)
- Bootstrap CSS for styling

## Project Structure

- `/api` - Backend API implementation
- `/frontend` - Frontend implementation
- `/docs` - Project documentation

## Documentation

For detailed information about the project design, architecture, and features, please refer to the [Design Document](docs/design.md).

## Getting Started

### Prerequisites

- Rust toolchain (latest stable version)
- MySQL database
- Node.js and npm (for frontend development)

### Setup

1. Clone the repository
2. Set up the database:
   ```
   cd api
   echo DATABASE_URL=mysql://username:password@localhost/house_management > .env
   diesel setup
   diesel migration run
   ```
3. Run the backend:
   ```
   cd api
   cargo run
   ```
4. Run the frontend (option A):
   ```
   cd frontend
   trunk serve
   ```

5. Or run both backend and frontend together (option B):
   ```
   ./scripts/dev.sh
   ```

6. Run checks/build locally:
   ```
   ./scripts/test.sh
   ```

## Endpoint Quick Reference (Selected)

| Purpose | Method | Path |
|---------|--------|------|
| List buildings | GET | /api/v1/buildings |
| Create building (Admin/Manager) | POST | /api/v1/buildings |
| Soft-delete building | DELETE | /api/v1/buildings/{id} |
| List deleted buildings | GET | /api/v1/buildings/deleted |
| Restore building | POST | /api/v1/buildings/{id}/restore |
| List building apartments | GET | /api/v1/buildings/{id}/apartments |
| Create apartment (Admin/Manager) | POST | /api/v1/apartments |
| Soft-delete apartment | DELETE | /api/v1/apartments/{id} |
| List deleted apartments | GET | /api/v1/apartments/deleted |
| Restore apartment | POST | /api/v1/apartments/{id}/restore |
| List apartment owners | GET | /api/v1/apartments/{id}/owners |
| Add apartment owner (Admin/Manager) | POST | /api/v1/apartments/{id}/owners |
| Remove apartment owner (Admin/Manager) | DELETE | /api/v1/apartments/{id}/owners/{user_id} |

## Maintenance Requests (Planned Implementation)
Will introduce three tables:
- `maintenance_requests`: core request data (apartment_id, created_by, request_type, priority, status, resolution_notes)
- `maintenance_request_attachments`: uploaded files metadata (original_filename, stored_filename, mime_type, size_bytes, is_deleted)
- `maintenance_request_history`: audit trail of status transitions (from_status, to_status, note)

Endpoints (initial set):
- POST /api/v1/requests
- GET /api/v1/requests (list, role-filtered)
- GET /api/v1/requests/{id}
- PUT /api/v1/requests/{id}/status
- POST /api/v1/requests/{id}/attachments (multipart)
- GET /api/v1/requests/{id}/attachments
- GET /api/v1/requests/{id}/attachments/{attachment_id}
- DELETE /api/v1/requests/{id}/attachments/{attachment_id}

Attachment constraints: max 10MB, allowed types: image/*, application/pdf (extensible). Files stored under STORAGE_DIR (default: ./storage) with UUID filenames.

## Voting Weights (Roadmap)
Voting proposals will specify a `weight_strategy`:
- `PerSeat`: each eligible voter counts as weight 1
- `ByApartmentSize`: weight derived from apartment `size_sq_m`
- `Custom`: weights looked up in a proposal-specific override table

Result calculation will aggregate weights of yes/no votes; majority and consensus rules implemented incrementally.

## Feature Progress Map
Implemented: Buildings, Apartments, Owner assignment, Soft delete + restore, Auth/JWT, Basic RBAC.
In Progress (next sprint): Maintenance Requests + Attachments, RBAC tests, Voting strategy scaffolding, Global frontend state & i18n.
Planned: Financials, Events/Calendar, Documents, Messaging (REST + WebSocket), Visitors, Analytics Dashboard.

## Testing Roadmap (Planned)

Updated upcoming additions:
- Backend tests: soft-delete & restore flows (buildings/apartments)
- Backend tests: RBAC guard denial cases
- Backend tests: maintenance request create/list/status transitions + history records
- Backend tests: attachment upload constraints (size/type)
- Frontend integration tests (Yew): Manager page list updates, maintenance list filtering
- RBAC matrix tests for new maintenance endpoints

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.
