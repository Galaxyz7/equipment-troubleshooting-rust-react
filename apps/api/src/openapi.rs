use utoipa::OpenApi;

/// OpenAPI documentation for the Equipment Troubleshooting API
///
/// This provides comprehensive API documentation accessible via Swagger UI.
/// Note: This is a basic specification. For detailed endpoint documentation,
/// refer to the route handler source code.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Equipment Troubleshooting API",
        version = "2.0.0",
        description = "
# Equipment Troubleshooting System API

A comprehensive REST API for managing equipment troubleshooting workflows, issues, and user sessions.

## Features

- **Authentication**: JWT-based authentication with role-based access control (Admin/Viewer/Technician)
- **Troubleshooting**: Guided question-and-answer flows to diagnose equipment issues
- **Issue Management**: Create and manage issue categories with decision trees
- **Admin Dashboard**: Session tracking, analytics, and audit logs
- **Node Graphs**: Alternative troubleshooting via node-based decision graphs

## Authentication

Most endpoints require authentication. Use the `/api/auth/login` endpoint to obtain a JWT token:

```json
POST /api/auth/login
{
  \"email\": \"admin@example.com\",
  \"password\": \"your-password\"
}
```

Include the token in subsequent requests:
```
Authorization: Bearer <your-jwt-token>
```

## API Endpoints

### Health
- `GET /health` - Basic health check
- `GET /api/health` - Database health check

### Authentication
- `POST /api/auth/login` - Login and get JWT token
- `POST /api/auth/refresh` - Refresh expired token
- `GET /api/auth/me` - Get current user info (requires auth)

### Questions (Public Read, Admin Write)
- `GET /api/questions` - List all questions
- `GET /api/questions/:id` - Get question by ID
- `POST /api/questions` - Create question (admin only)
- `PUT /api/questions/:id` - Update question (admin only)
- `DELETE /api/questions/:id` - Delete question (admin only)

### Answers (Public Read, Admin Write)
- `GET /api/questions/:question_id/answers` - List answers for a question
- `POST /api/questions/:question_id/answers` - Create answer (admin only)
- `PUT /api/answers/:id` - Update answer (admin only)
- `DELETE /api/answers/:id` - Delete answer (admin only)

### Troubleshooting (Public Access)
- `POST /api/troubleshoot/start` - Start a troubleshooting session
- `GET /api/troubleshoot/:session_id` - Get session state
- `POST /api/troubleshoot/:session_id/answer` - Submit answer to current question
- `GET /api/troubleshoot/:session_id/history` - Get session history

### Admin (Admin Only)
- `GET /api/admin/sessions` - List all troubleshooting sessions
- `GET /api/admin/stats` - Get dashboard statistics
- `GET /api/admin/audit-logs` - Get audit logs

### Issues (Admin Only)
- `GET /api/admin/issues` - List all issue categories
- `POST /api/admin/issues` - Create issue category
- `GET /api/admin/issues/:category/tree` - Get decision tree for issue
- `GET /api/admin/issues/:category/graph` - Get node graph for issue
- `PUT /api/admin/issues/:category` - Update issue
- `DELETE /api/admin/issues/:category` - Delete issue
- `PATCH /api/admin/issues/:category/toggle` - Toggle issue active status

### Nodes & Connections (Admin Only)
- `GET /api/nodes` - List all nodes
- `GET /api/nodes/:id` - Get node by ID
- `GET /api/nodes/:id/with-connections` - Get node with connections
- `POST /api/nodes` - Create node
- `PUT /api/nodes/:id` - Update node
- `DELETE /api/nodes/:id` - Delete node
- `GET /api/connections` - List all connections
- `POST /api/connections` - Create connection
- `PUT /api/connections/:id` - Update connection
- `DELETE /api/connections/:id` - Delete connection

## Rate Limiting

API requests are limited to **100 requests per minute** per IP address.

## Security

- All passwords are hashed using Argon2
- JWT tokens expire after 24 hours (configurable)
- HTTPS enforced in production
- Security headers enabled (HSTS, CSP, X-Frame-Options, etc.)
- Rate limiting prevents abuse

## Response Format

All responses follow a consistent format:

**Success Response:**
```json
{
  \"data\": { ... }
}
```

**Error Response:**
```json
{
  \"error\": \"Error message\",
  \"details\": \"Additional error details\"
}
```

## Status Codes

- `200 OK` - Request successful
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid request data
- `401 Unauthorized` - Authentication required
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error
        ",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:5000", description = "Local development server"),
        (url = "http://localhost:3000", description = "Frontend development proxy"),
        (url = "https://api.example.com", description = "Production server")
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Authentication", description = "User authentication and authorization"),
        (name = "Questions", description = "Question management"),
        (name = "Answers", description = "Answer management"),
        (name = "Troubleshooting", description = "Troubleshooting session management"),
        (name = "Admin", description = "Administrative endpoints (Admin role required)"),
        (name = "Issues", description = "Issue category management"),
        (name = "Nodes", description = "Node-graph based troubleshooting"),
        (name = "Connections", description = "Connection management for node graphs"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Add security schemes to OpenAPI documentation
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("Enter your JWT token in the format: Bearer <token>"))
                        .build(),
                ),
            );
        }
    }
}
