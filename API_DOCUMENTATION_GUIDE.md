# API Documentation Guide

## Overview

The House Management API now has OpenAPI/Swagger documentation infrastructure in place using `utoipa`. The documentation is automatically generated from Rust code annotations and provides an interactive API explorer.

## Access Documentation

- **Swagger UI**: http://localhost:8080/swagger-ui/
- **OpenAPI JSON**: http://localhost:8080/api-docs/openapi.json

## Current Status

### ✅ Fully Documented (73 endpoints)

**Authentication** (2 endpoints)
- `POST /api/v1/register` - User registration
- `POST /api/v1/login` - User login with JWT

**Buildings** (5 endpoints)
- `GET /api/v1/buildings` - List all active buildings
- `POST /api/v1/buildings` - Create new building (Admin/Manager)
- `DELETE /api/v1/buildings/{id}` - Soft-delete building (Admin/Manager)
- `GET /api/v1/buildings/deleted` - List deleted buildings (Admin/Manager)
- `POST /api/v1/buildings/{id}/restore` - Restore deleted building (Admin/Manager)

**Apartments** (10 endpoints)
- `GET /api/v1/apartments` - List all apartments
- `GET /api/v1/buildings/{id}/apartments` - List apartments for a building
- `POST /api/v1/apartments` - Create apartment (Admin/Manager)
- `DELETE /api/v1/apartments/{id}` - Soft-delete apartment (Admin/Manager)
- `GET /api/v1/apartments/deleted` - List deleted apartments (Admin/Manager)
- `POST /api/v1/apartments/{id}/restore` - Restore apartment (Admin/Manager)
- `GET /api/v1/apartments/{id}/owners` - List apartment owners
- `POST /api/v1/apartments/{id}/owners` - Assign owner (Admin/Manager)
- `DELETE /api/v1/apartments/{id}/owners/{user_id}` - Remove owner (Admin/Manager)

**Voting System** (5 endpoints)
- `GET /api/v1/proposals` - List all proposals
- `GET /api/v1/proposals/{id}` - Get proposal details with vote statistics
- `POST /api/v1/proposals` - Create proposal (Admin/Manager)
- `POST /api/v1/proposals/{id}/vote` - Cast or update vote
- `POST /api/v1/proposals/{id}/tally` - Tally results (Admin/Manager)

**Maintenance Requests** (15 endpoints)
- `GET /api/v1/requests` - List maintenance requests
- `GET /api/v1/requests/{id}` - Get request details
- `POST /api/v1/requests` - Create request
- `PUT /api/v1/requests/{id}` - Update request (Admin/Manager)
- `PUT /api/v1/requests/{id}/status` - Update status (Admin/Manager)
- `GET /api/v1/requests/{id}/history` - View status history
- `PUT /api/v1/requests/{id}/assign` - Assign request (Admin/Manager)
- `DELETE /api/v1/requests/{id}/assign` - Unassign request (Admin/Manager)
- `POST /api/v1/requests/{id}/attachments` - Upload attachment
- `GET /api/v1/requests/{id}/attachments` - List attachments
- `GET /api/v1/requests/{id}/attachments/deleted` - List deleted attachments
- `GET /api/v1/requests/{id}/attachments/{att_id}` - Get attachment metadata
- `GET /api/v1/requests/{id}/attachments/{att_id}/download` - Download attachment
- `DELETE /api/v1/requests/{id}/attachments/{att_id}` - Soft-delete attachment
- `POST /api/v1/requests/{id}/attachments/{att_id}/restore` - Restore attachment

**Announcements** (16 endpoints)
- `GET /api/v1/announcements/public` - List public announcements (no auth)
- `GET /api/v1/announcements` - List announcements (authenticated)
- `GET /api/v1/announcements/deleted` - List deleted announcements (Admin/Manager)
- `GET /api/v1/announcements/{id}` - Get announcement details
- `POST /api/v1/announcements` - Create announcement (Admin/Manager)
- `PUT /api/v1/announcements/{id}` - Update announcement
- `DELETE /api/v1/announcements/{id}` - Soft-delete announcement (Admin/Manager)
- `POST /api/v1/announcements/{id}/restore` - Restore announcement (Admin/Manager)
- `DELETE /api/v1/announcements/{id}/purge` - Permanently delete (Admin/Manager)
- `POST /api/v1/announcements/{id}/pin` - Toggle pin status (Admin/Manager)
- `POST /api/v1/announcements/{id}/publish` - Publish immediately
- `GET /api/v1/announcements/{id}/comments` - List comments
- `POST /api/v1/announcements/{id}/comments` - Create comment
- `DELETE /api/v1/announcements/comments/{comment_id}` - Soft-delete comment
- `POST /api/v1/announcements/comments/{comment_id}/restore` - Restore comment (Admin/Manager)
- `DELETE /api/v1/announcements/comments/{comment_id}/purge` - Permanently delete comment (Admin/Manager)

**Users** (5 endpoints)
- `GET /api/v1/users` - List all users
- `GET /api/v1/users/public` - List users (public info only) (Admin/Manager)
- `GET /api/v1/users/with_roles` - List users with roles (Admin)
- `POST /api/v1/users` - Create user (Admin)
- `POST /api/v1/users/{id}/roles` - Set user roles (Admin)

**Meters** (15 endpoints)
- `GET /api/v1/apartments/{apartment_id}/meters` - List meters for apartment
- `POST /api/v1/meters` - Register new meter (Admin/Manager)
- `GET /api/v1/meters/calibration-due` - List meters needing calibration (Admin/Manager)
- `GET /api/v1/meters/{id}` - Get meter details
- `PUT /api/v1/meters/{id}` - Update meter details (Admin/Manager)
- `DELETE /api/v1/meters/{id}` - Deactivate meter (Admin/Manager)
- `GET /api/v1/meters/{id}/readings/export` - Export readings as CSV
- `GET /api/v1/meters/{id}/readings` - Get historical readings (paginated)
- `POST /api/v1/meters/{id}/readings` - Manual reading entry (Admin/Manager)
- `POST /api/v1/meters/{id}/calibrate` - Record meter calibration (Admin/Manager)
- `POST /api/v1/webhooks/meter-reading` - Webhook for single reading (API key auth)
- `POST /api/v1/webhooks/meter-reading-batch` - Webhook for batch readings (API key auth)
- `GET /api/v1/admin/api-keys` - List webhook API keys (Admin)
- `POST /api/v1/admin/api-keys` - Create API key (Admin)
- `DELETE /api/v1/admin/api-keys/{id}` - Revoke API key (Admin)

## How to Add Documentation

### 1. Add `utoipa` import to the module

```rust
use utoipa;
```

### 2. Add `#[utoipa::path]` macro to each handler

```rust
/// Brief description
///
/// Longer description explaining what the endpoint does.
#[utoipa::path(
    get,                                    // HTTP method
    path = "/api/v1/resource",              // Full path
    params(                                  // Optional: path/query parameters
        ("id" = u64, Path, description = "Resource ID"),
        ("filter" = Option<String>, Query, description = "Filter results")
    ),
    request_body = CreateResourceRequest,   // Optional: request body type
    responses(
        (status = 200, description = "Success", body = Resource),
        (status = 404, description = "Not found"),
        (status = 403, description = "Forbidden - requires specific role")
    ),
    tag = "ResourceTag",                    // Group endpoints by tag
    security(("bearer_auth" = []))          // Optional: requires authentication
)]
pub async fn handler_function(...) -> Result<impl Responder, AppError> {
    // Implementation
}
```

### 3. Register the path in `src/openapi.rs`

Add the function to the `paths()` section:

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::module::handler_function,
        // ... other paths
    ),
)]
pub struct ApiDoc;
```

### 4. Add request/response types to schemas

If you created new request/response types, add them to `components/schemas`:

```rust
components(
    schemas(
        crate::models::YourNewType,
        // ... other schemas
    )
)
```

## Example: Complete Endpoint Documentation

Here's a complete example for an apartments endpoint:

```rust
/// List apartments for a building
///
/// Returns all apartments in a specific building that haven't been soft-deleted.
/// Accessible to all authenticated users.
#[utoipa::path(
    get,
    path = "/api/v1/buildings/{building_id}/apartments",
    params(
        ("building_id" = u64, Path, description = "Building ID")
    ),
    responses(
        (status = 200, description = "List of apartments", body = Vec<Apartment>),
        (status = 404, description = "Building not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_apartments_for_building(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>
) -> Result<impl Responder, AppError> {
    // Implementation
}
```

Then add to openapi.rs:

```rust
paths(
    crate::apartments::list_apartments_for_building,
)
```

## Templates for Common Patterns

### List Endpoint (GET collection)

```rust
#[utoipa::path(
    get,
    path = "/api/v1/resources",
    responses(
        (status = 200, description = "List of resources", body = Vec<Resource>)
    ),
    tag = "Resources"
)]
```

### Create Endpoint (POST)

```rust
#[utoipa::path(
    post,
    path = "/api/v1/resources",
    request_body = CreateResourceRequest,
    responses(
        (status = 201, description = "Resource created successfully"),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Resources",
    security(("bearer_auth" = []))
)]
```

### Get Single Resource (GET with ID)

```rust
#[utoipa::path(
    get,
    path = "/api/v1/resources/{id}",
    params(
        ("id" = u64, Path, description = "Resource ID")
    ),
    responses(
        (status = 200, description = "Resource details", body = Resource),
        (status = 404, description = "Resource not found")
    ),
    tag = "Resources",
    security(("bearer_auth" = []))
)]
```

### Update Endpoint (PUT)

```rust
#[utoipa::path(
    put,
    path = "/api/v1/resources/{id}",
    params(
        ("id" = u64, Path, description = "Resource ID")
    ),
    request_body = UpdateResourceRequest,
    responses(
        (status = 200, description = "Resource updated successfully", body = Resource),
        (status = 404, description = "Resource not found"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Resources",
    security(("bearer_auth" = []))
)]
```

### Delete Endpoint (DELETE)

```rust
#[utoipa::path(
    delete,
    path = "/api/v1/resources/{id}",
    params(
        ("id" = u64, Path, description = "Resource ID")
    ),
    responses(
        (status = 204, description = "Resource deleted successfully"),
        (status = 404, description = "Resource not found"),
        (status = 403, description = "Forbidden - requires Admin or Manager")
    ),
    tag = "Resources",
    security(("bearer_auth" = []))
)]
```

## Tips

1. **Consistent Descriptions**: Use present tense ("Returns...", "Creates...", "Updates...")
2. **Security**: Add `security(("bearer_auth" = []))` for authenticated endpoints
3. **RBAC**: Mention required roles in descriptions (e.g., "Requires Admin or Manager role")
4. **Status Codes**: Include all possible response codes (200, 201, 400, 403, 404, 500)
5. **Examples**: Add `#[schema(example = "...")]` to model fields for better documentation
6. **Tags**: Group related endpoints with the same tag for organized UI

## Testing Documentation

1. Start the API server:
   ```bash
   cd api && cargo run
   ```

2. Open Swagger UI: http://localhost:8080/swagger-ui/

3. Click "Authorize" and enter a JWT token from `/api/v1/login`

4. Try out endpoints directly from the browser!

## All Endpoints Documented! 🎉

All 58 API endpoints have been fully documented with OpenAPI/Swagger annotations. You can now:

1. View interactive documentation at http://localhost:8080/swagger-ui/
2. Try out endpoints directly from the browser
3. Click "Authorize" to add your JWT token and test authenticated endpoints
4. Download the OpenAPI JSON spec from http://localhost:8080/api-docs/openapi.json

The documentation includes:
- Complete request/response schemas
- All path and query parameters
- Security requirements (JWT authentication)
- RBAC role requirements in descriptions
- Proper HTTP status codes for all scenarios
