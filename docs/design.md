# House Management System - Design Document

## Project Overview

The House Management System is a comprehensive platform designed to facilitate the management of residential properties for Homeowners Associations (HOAs) and similar entities. The system provides a centralized solution for managing various aspects of residential properties, including user access, property information, common areas, financial management, event scheduling, voting, and maintenance requests.

## Goals

- Provide a unified platform for managing residential properties
- Support multiple user types with different permissions and access levels
- Enable efficient management of properties, common areas, and shared assets
- Facilitate financial management with flexible cost calculation methods
- Support scheduling and tracking of events and maintenance activities
- Implement a voting system for community decision-making
- Handle maintenance requests and complaints efficiently
- Support management of multiple houses within a single system

## User Types and Permissions

### User Types

1. **Administrators**
   - Full access to all system features
   - Can manage users, properties, and system settings

2. **Homeowners**
   - Access to their own property information
   - Can view and participate in community activities
   - Can submit maintenance requests and vote on issues
   - Can access financial information related to their property

3. **Renters**
   - Limited access to property information
   - Can submit maintenance requests
   - Can view community information
   - Cannot vote on community issues

4. **Management Personnel**
   - Can manage properties and common areas
   - Can handle maintenance requests
   - Can manage financial aspects
   - Cannot modify system settings

5. **HOA Members**
   - Can participate in voting
   - Can view community information
   - Can access financial reports
   - Cannot modify property information

### Permission System

The system will implement a role-based access control (RBAC) system where:
- Each user can have one or more roles
- Each role has a set of permissions
- Permissions determine what actions a user can perform
- Permissions are granular and can be combined to create custom roles

## Managed Entities

1. **Houses/Buildings**
   - Basic information (address, construction year, etc.)
   - Common facilities
   - General maintenance history
   - Documents and plans

2. **Common Areas**
   - Description and purpose
   - Usage rules
   - Maintenance schedule
   - Reservation system (if applicable)

3. **Individual Apartments/Units**
   - Owner/tenant information
   - Size and specifications
   - Maintenance history
   - Financial obligations

4. **Shared Assets**
   - Tools, furniture, equipment
   - Availability status
   - Reservation system
   - Maintenance records

## Features

### Financial Management

- **Cost Calculation Methods**
  - Per inhabitant (e.g., garbage removal)
  - Based on apartment area (e.g., repair fund)
  - Equal distribution among all units
  - Custom formulas

- **Financial Tracking**
  - Income and expenses
  - Budget planning
  - Payment tracking
  - Financial reports

### Calendar and Scheduling

- **Event Types**
  - Regular maintenance
  - Planned repairs
  - Community meetings
  - Special events

- **Calendar Features**
  - Event creation and editing
  - Notifications and reminders
  - Recurring events
  - Calendar sharing

### Voting System

- **Voting Types**
  - Simple majority
  - Weighted voting (based on apartment size)
  - Consensus-based decisions

- **Voting Features**
  - Proposal creation
  - Voting period management
  - Result calculation and display
  - Historical voting records

### Maintenance and Complaints

- **Request Types**
  - Urgent repairs
  - Regular maintenance
  - Improvements
  - Complaints

- **Request Management**
  - Submission form
  - Status tracking
  - Assignment to responsible parties
  - Resolution documentation

## Technical Architecture

### Backend (API)

- **Framework**: Actix-web
- **Database**: MySQL with Diesel ORM
- **Authentication**: JWT-based authentication
- **API Design**: RESTful API with JSON responses
- **Migrations**: Diesel migrations for database schema management

### Frontend

- **Framework**: Yew (Rust-based WebAssembly framework)
- **Styling**: Bootstrap CSS
- **State Management**: Yew's built-in state management
- **Routing**: Client-side routing with Yew Router
- **API Communication**: Fetch API with Rust wrappers

### Deployment

- **Backend**: Containerized deployment with Docker
- **Frontend**: Static site hosting with WebAssembly support
- **Database**: Managed MySQL instance or containerized deployment
- **CI/CD**: Automated testing and deployment pipeline

### Multi-Language Support

- **Supported Languages**
  - English (primary/default)
  - Czech
  - Other European languages as needed

- **Translation Features**
  - User interface translation
  - Content translation
  - User-selectable language preference
  - Language detection based on browser settings
  - Support for right-to-left languages

## Future Considerations

- Mobile application for convenient access
- Integration with smart home systems
- Advanced reporting and analytics
- Document management system
- Integration with payment processors

## Current Implementation Status (MVP)

- Backend
  - Actix-web server with versioned API at /api/v1
  - Endpoints:
    - GET /api/v1/health (i18n-aware)
    - GET/POST /api/v1/users (basic user list/create)
    - GET/POST /api/v1/buildings
    - GET/POST /api/v1/apartments
    - GET /api/v1/buildings/{id}/apartments
  - Diesel migrations: users, roles, user_roles, buildings, apartments
  - Structure encourages small modules and can be expanded with middleware (JWT/RBAC)

- Frontend
  - Yew SPA with yew-router
  - Pages: Home (health), Buildings (list/create), Apartments in a building (list/create)
  - Bootstrap included via CDN; components organized to keep files short

- Tooling
  - Dev script to run API and Frontend together (scripts/dev.sh)
  - Test script (scripts/test.sh) runs fmt, clippy, tests, and trunk build
  - GitHub Actions CI builds and tests backend and builds frontend

- Next steps
  - Add authentication (JWT) and RBAC enforcement per design
  - Expand domain models: maintenance requests, financials, events, voting, documents, messaging, visitors
  - Implement frontend auth and i18n switching

- Soft delete implemented for buildings and apartments using `is_deleted` flag with list + restore endpoints.
- Owner assignment join table `apartment_owners` with idempotent add/remove logic.

### Soft Delete Conventions
- Delete endpoints set `is_deleted = true` rather than removing rows.
- Separate endpoints provide listing of deleted entities and restoration (`POST /buildings/{id}/restore`, `POST /apartments/{id}/restore`).
- Queries for active entities must always filter `is_deleted = false`.

## Upcoming Domain Expansions

### Maintenance Requests
Data model (planned):
- `maintenance_requests` (id PK, apartment_id FK -> apartments, created_by FK -> users, request_type ENUM, priority ENUM, title, description TEXT, status ENUM(Open, InProgress, Resolved), resolution_notes TEXT NULL, created_at, updated_at)
- `maintenance_request_attachments` (id PK, request_id FK, original_filename, stored_filename, mime_type, size_bytes INT, is_deleted BOOL DEFAULT false, created_at)
- `maintenance_request_history` (id PK, request_id FK, from_status ENUM NULL, to_status ENUM, note TEXT NULL, changed_by FK -> users, changed_at)

Attachment handling:
- Uploaded via multipart (`POST /api/v1/requests/{id}/attachments`) containing file field `file`.
- Stored on disk under `STORAGE_DIR/requests/{request_id}/<uuid>`.
- Metadata persisted; physical file removal deferred until cleanup job (soft delete metadata first).
- Validation: file size <= 10MB, mime type whitelist (image/*, application/pdf).

Planned endpoints:
- `POST /api/v1/requests` (create; roles: Homeowner, Renter, Admin, Manager)
- `GET /api/v1/requests` (list; admins/managers see all; others see own apartment(s) or own created requests)
- `GET /api/v1/requests/{id}` (detail with attachments + history)
- `PUT /api/v1/requests/{id}/status` (update status; roles: Admin, Manager)
- `POST /api/v1/requests/{id}/attachments` (upload; creator/Admin/Manager)
- `GET /api/v1/requests/{id}/attachments` (list metadata)
- `GET /api/v1/requests/{id}/attachments/{attachment_id}` (download binary)
- `DELETE /api/v1/requests/{id}/attachments/{attachment_id}` (soft delete attachment; creator/Admin/Manager)

Status transitions:
- Each status change inserts row into history with from/to + note/resolution.
- Resolution notes stored when moving to Resolved.

### Voting System (Weight Strategies)
Data model (planned):
- `proposals` (id, title, description, start_time, end_time, created_by, weight_strategy ENUM(PerSeat, ByApartmentSize, Custom), status ENUM(Open, Closed), created_at)
- `votes` (id, proposal_id FK, user_id FK, choice ENUM(Yes, No, Abstain), weight DECIMAL(18,6), cast_at)
- `proposal_custom_weights` (proposal_id FK, apartment_id FK, weight DECIMAL(18,6)) for Custom strategy.

Weight computation:
- PerSeat: each vote weight = 1.
- ByApartmentSize: join apartment to size; weight = size_sq_m (NULL -> 0).
- Custom: lookup weight; missing -> 0.

Result calculation:
- Aggregate weights of Yes vs total eligible weights; pass threshold = >50% Yes (simple majority) initially.
- Future extension: consensus (100% of non-abstain Yes) and other rules.

### RBAC Overview (Expanded)
- Maintenance create: Homeowner, Renter, Admin, Manager.
- Maintenance status update: Admin, Manager.
- Maintenance attachment upload/delete: Request creator, Admin, Manager.
- Voting proposal create: Admin, HOA Member (to confirm), Manager.
- Vote cast: roles defined per proposal configuration (initial default: Admin, HOA Member, Homeowner).

## Testing Strategy (Planned Additions)
- Integration tests using test database (transaction rollback per test) for maintenance and voting flows.
- RBAC matrix tests for denial cases.
- Attachment validation tests (size/mime) with in-memory multipart payloads.
- Voting weight calculation unit tests (PerSeat, ByApartmentSize, Custom mapping).
- Soft delete queries performance: add indexes to `buildings.is_deleted`, `apartments.is_deleted`, `maintenance_request_attachments.is_deleted`.

## Implementation Preferences (Updated)
- Use enums represented as VARCHAR for flexibility (validate at application layer) or MySQL ENUM if kept stable.
- Prefer explicit `updated_at` triggers in application layer rather than DB triggers for portability.
- History table is append-only; no updates allowed (enforced by not exposing update endpoint).
