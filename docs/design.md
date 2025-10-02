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
