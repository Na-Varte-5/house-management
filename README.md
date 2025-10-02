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

## License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.
