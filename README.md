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

## Recent Changes (January 2026)

### Renter Invitation System ✅ (January 28, 2026)
Email-based invitation workflow for apartment owners to invite renters:
- **Invite by email**: Owners can invite renters by email address
- **Smart assignment**: If email exists in system, user is immediately assigned as renter; otherwise invitation is created
- **Token-based acceptance**: Pending invitations can be accepted when user registers or logs in
- **Invitation management**: List, cancel pending invitations; automatic expiration after 7 days
- **RBAC enforcement**: Apartment owners, Admin, and Manager can manage renters

### Owner-Based Renter Management ✅ (January 28, 2026)
Expanded permissions for apartment owners to manage their own renters:
- **Owner permissions**: Apartment owners can now add, update, and remove renters (previously Admin/Manager only)
- **New endpoints**: `GET/POST /apartments/{id}/invite`, `GET/DELETE /apartments/{id}/invitations/{id}`
- **Apartment details**: New `GET /apartments/{id}` and `GET /apartments/{id}/permissions` endpoints
- **Property history**: Audit trail for all renter-related changes

### Maintenance Request Escalation ✅ (January 28, 2026)
Apartment owners can escalate maintenance requests to building managers:
- **Escalation endpoint**: `POST /requests/{id}/escalate` to reassign to building manager
- **Assignment history**: All assignment changes (assign, unassign, escalate) now logged in history
- **Attachment permissions**: Apartment owners can now manage attachments on their requests

### Maintenance Request System - Complete Implementation ✅
Full-featured maintenance request tracking with enriched data and comprehensive audit history:
- **Enriched API responses**: All endpoints return apartment numbers, building addresses, and user names (not just IDs)
- **Comprehensive audit history**: Status, priority, and assignment changes all logged with user names and timestamps
- **User-friendly display**: Formatted dates ("Jan 14, 2026 at 10:30"), proper dropdown defaults, names instead of IDs
- **File attachments**: Upload images/PDFs (max 10MB) with metadata tracking
- **RBAC enforcement**: Admin/Manager can update all fields; users can only view their own requests

### Code Quality Improvements (January 28, 2026)
- Fixed all Clippy warnings across the codebase
- Added type aliases for complex tuple types (RenterRow, PropertyHistoryRow, InvitationRow, etc.)
- Refactored collapsible if statements for cleaner code
- Removed unnecessary `.clone()` calls on Copy types

### Previous Features (November 2025)
- Manager page (Yew) consolidating building & apartment management plus owner assignment.
- Soft-delete support for buildings and apartments (is_deleted flag) with toggle to show deleted items and one-click restore.
- Owner assignment UI with searchable public user list; click to assign / badge with close button to remove.
- Deletion confirmation modal for buildings/apartments (prevent accidental removal).
- Loading spinners for buildings, apartments, owners, and deleted lists to improve UX feedback.
- Role-based visibility: building and apartment creation restricted to Admin/Manager; other roles see access denied message.
- Immediate in-place refresh after delete/restore (no full page reload required).

### Upcoming (In Progress)
- Water meter analytics and visualization (consumption charts, period comparisons, PDF reports)
- Email notifications for maintenance requests
- Test harness: backend integration tests for RBAC and maintenance workflows

## RBAC Summary
Roles currently recognized: Admin, Manager, Homeowner, Renter, HOA Member.

| Action | Allowed Roles (current enforcement) |
|--------|-------------------------------------|
| Create/Soft-delete/Restore Building | Admin, Manager |
| Create/Soft-delete/Restore Apartment | Admin, Manager |
| Assign/Remove Apartment Owner | Admin, Manager |
| **Manage Apartment Renters** | Admin, Manager, **Apartment Owner** |
| **Invite Renter by Email** | Admin, Manager, **Apartment Owner** |
| Submit Maintenance Request | Homeowner, Renter, Admin, Manager |
| View Maintenance Request | Request creator, assigned user, **apartment owner**, Admin, Manager |
| Update Maintenance Status/Priority/Assignment | Admin, Manager |
| **Escalate Maintenance Request** | Admin, Manager, **Apartment Owner** |
| Upload Maintenance Attachment | Request creator, **apartment owner**, Admin, Manager |
| View Maintenance History | Request creator, **apartment owner**, Admin, Manager |

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
| **Get apartment details** | GET | /api/v1/apartments/{id} |
| **Get apartment permissions** | GET | /api/v1/apartments/{id}/permissions |
| **Invite renter by email** | POST | /api/v1/apartments/{id}/invite |
| **List pending invitations** | GET | /api/v1/apartments/{id}/invitations |
| **Cancel invitation** | DELETE | /api/v1/apartments/{id}/invitations/{invitation_id} |
| **Get invitation by token** | GET | /api/v1/invitations/{token} |
| **Accept invitation** | POST | /api/v1/invitations/{token}/accept |
| **List my invitations** | GET | /api/v1/invitations/my |

## Maintenance Requests (✅ Implemented)

Three tables in production:
- `maintenance_requests`: core request data (apartment_id, created_by, request_type, priority, status, resolution_notes, assigned_to)
- `maintenance_request_attachments`: uploaded files metadata (original_filename, stored_filename, mime_type, size_bytes, is_deleted)
- `maintenance_request_history`: comprehensive audit trail (status changes, priority changes, assignment changes with user names)

### Available Endpoints

| Method | Path | Description | Response Type |
|--------|------|-------------|---------------|
| GET | /api/v1/requests | List requests with apartment/building context | MaintenanceRequestEnriched[] |
| POST | /api/v1/requests | Create new request | { id: number } |
| GET | /api/v1/requests/{id} | Get request details with user names | MaintenanceRequestDetail |
| PUT | /api/v1/requests/{id} | Update status/priority/assignment | MaintenanceRequestDetail |
| GET | /api/v1/requests/{id}/history | Get audit trail with user names | MaintenanceRequestHistoryEnriched[] |
| POST | /api/v1/requests/{id}/attachments | Upload file (multipart) | Attachment metadata |
| GET | /api/v1/requests/{id}/attachments | List attachments | Attachment[] |
| GET | /api/v1/requests/{id}/attachments/{attachment_id} | Download file | Binary stream |
| DELETE | /api/v1/requests/{id}/attachments/{attachment_id} | Soft-delete attachment | 200 OK |
| **POST** | **/api/v1/requests/{id}/escalate** | **Escalate to building manager** | **200 OK** |

### Key Features
- **Enriched responses**: All endpoints return human-readable data (apartment numbers, building addresses, user names)
- **Comprehensive history**: Status, priority, and assignment changes logged with descriptive notes
- **Attachment constraints**: max 10MB, allowed types: `image/*`, `application/pdf`
- **Storage**: Files stored under `STORAGE_DIR` (default: `./storage`) with UUID filenames
- **RBAC**: Role-based filtering ensures users only see authorized requests

## Voting Weights (Roadmap)
Voting proposals will specify a `weight_strategy`:
- `PerSeat`: each eligible voter counts as weight 1
- `ByApartmentSize`: weight derived from apartment `size_sq_m`
- `Custom`: weights looked up in a proposal-specific override table

Result calculation will aggregate weights of yes/no votes; majority and consensus rules implemented incrementally.

## Feature Progress Map

**✅ Fully Implemented:**
- Buildings and Apartments CRUD with soft-delete/restore
- Owner assignment (many-to-many)
- User management with RBAC (5 roles)
- JWT authentication with secure token handling
- **Maintenance Requests**: Complete system with enriched data, comprehensive audit history, file attachments, and escalation
- **Voting System**: Proposal creation, weighted voting methods, result tallying
- **Water Meter System**: Reading tracking, webhook integration, CSV export, calibration monitoring
- **Renter Management**: Owner-based renter management with email invitations
- Announcements with pinning and comments

**🚧 In Progress:**
- Water meter analytics and visualization
- Email notifications for maintenance requests
- Backend integration tests for RBAC enforcement

**📋 Planned:**
- Financials module (invoicing, payments, cost tracking)
- Events/Calendar system
- Document management
- Messaging system (REST + WebSocket)
- Visitor management
- Analytics Dashboard with reporting

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
