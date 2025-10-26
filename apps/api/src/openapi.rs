use utoipa::OpenApi;

/// OpenAPI documentation for the Equipment Troubleshooting API
///
/// This provides comprehensive API documentation accessible via Swagger UI.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Equipment Troubleshooting API",
        version = "2.0.0",
        description = "
# 🛠️ Equipment Troubleshooting System API

A comprehensive REST API for managing equipment troubleshooting workflows, issues, and user sessions.

> **Quick Start:** Use the `POST /api/auth/login` endpoint below to get a JWT token, then click the 🔒 **Authorize** button at the top to test authenticated endpoints.
---

## 📚 Overview

This API powers an intelligent troubleshooting system for equipment diagnosis using:
- **Decision Trees**: Guided question-and-answer workflows
- **Node Graphs**: Visual decision flow management with React Flow
- **Session Tracking**: Full audit trail of troubleshooting sessions

## ✨ Key Features

| Feature | Description |
|---------|-------------|
| 🔐 **Authentication** | JWT-based auth with role-based access control (Admin/Viewer/Technician) |
| 🔍 **Troubleshooting** | Guided Q&A flows to diagnose equipment issues |
| 📋 **Issue Management** | Create and manage issue categories with decision trees |
| 📊 **Admin Dashboard** | Real-time session tracking, analytics, and performance metrics |
| 🎨 **Visual Editor** | React Flow-based graph editor for creating diagnostic flows |
| 💾 **Caching** | Intelligent caching layer for optimal performance |
| 🚦 **Rate Limiting** | Built-in protection against abuse (100 req/min per IP) |

---

## 🔐 Authentication

**IMPORTANT:** Most endpoints require authentication. Follow these steps:

### Step 1: Get a JWT Token
```bash
curl -X POST https://your-domain.com/api/auth/login \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"email\": \"admin@example.com\",
    \"password\": \"your-password\"
  }'
```

**Response:**
```json
{
  \"token\": \"eyJhbGciOiJIUzI1NiIs...\",
  \"refresh_token\": \"eyJhbGciOiJIUzI1...\",
  \"user\": {
    \"email\": \"admin@example.com\",
    \"role\": \"Admin\"
  }
}
```

### Step 2: Use the Token
Click the **🔒 Authorize** button at the top of this page and paste your token:
```
Bearer eyJhbGciOiJIUzI1NiIs...
```

Or include it in request headers:
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

---

## 📍 API Endpoints Reference

### 💚 Health & Monitoring
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/health` | Basic health check | ❌ No |
| `GET` | `/api/health` | Database connection health | ❌ No |
| `GET` | `/api/admin/performance` | Performance metrics (DB pool, cache stats) | ✅ Admin |

### 🔐 Authentication
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/auth/login` | Login and get JWT token | ❌ No |
| `POST` | `/api/auth/refresh` | Refresh expired JWT token | ❌ No |
| `GET` | `/api/auth/me` | Get current user info | ✅ Yes |

### ❓ Questions (Legacy - Question/Answer Tree System)
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/questions` | List all questions | ❌ No |
| `GET` | `/api/questions/:id` | Get question by ID | ❌ No |
| `POST` | `/api/questions` | Create question | ✅ Admin |
| `PUT` | `/api/questions/:id` | Update question | ✅ Admin |
| `DELETE` | `/api/questions/:id` | Delete question | ✅ Admin |

### 💬 Answers (Legacy - Question/Answer Tree System)
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/questions/:question_id/answers` | List answers for question | ❌ No |
| `POST` | `/api/questions/:question_id/answers` | Create answer | ✅ Admin |
| `PUT` | `/api/answers/:id` | Update answer | ✅ Admin |
| `DELETE` | `/api/answers/:id` | Delete answer | ✅ Admin |

### 🔍 Troubleshooting (Public User Sessions)
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/api/troubleshoot/start` | Start troubleshooting session | ❌ No |
| `GET` | `/api/troubleshoot/:session_id` | Get session state | ❌ No |
| `POST` | `/api/troubleshoot/:session_id/answer` | Submit answer to current question | ❌ No |
| `GET` | `/api/troubleshoot/:session_id/history` | Get session history | ❌ No |

### 📊 Admin Dashboard
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/admin/sessions` | List all troubleshooting sessions (paginated) | ✅ Admin |
| `GET` | `/api/admin/stats` | Dashboard statistics (sessions, conclusions, etc.) | ✅ Admin |
| `GET` | `/api/admin/audit-logs` | Get audit logs | ✅ Admin |

### 📋 Issues (Node-Graph System)
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/admin/issues` | List all issue categories | ✅ Admin |
| `POST` | `/api/admin/issues` | Create issue category with root node | ✅ Admin |
| `GET` | `/api/admin/issues/:category/tree` | Get decision tree (legacy format) | ✅ Admin |
| `GET` | `/api/admin/issues/:category/graph` | Get node graph for React Flow editor | ✅ Admin |
| `PUT` | `/api/admin/issues/:category` | Update issue metadata | ✅ Admin |
| `DELETE` | `/api/admin/issues/:category` | Delete entire issue category | ✅ Admin |
| `PATCH` | `/api/admin/issues/:category/toggle` | Toggle issue active/inactive | ✅ Admin |

### 🎯 Nodes (Decision Flow Nodes)
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/nodes` | List nodes (filterable by category/type) | ✅ Admin |
| `GET` | `/api/nodes/:id` | Get node by ID | ✅ Admin |
| `GET` | `/api/nodes/:id/with-connections` | Get node with all connections | ✅ Admin |
| `POST` | `/api/nodes` | Create node (Question or Conclusion) | ✅ Admin |
| `PUT` | `/api/nodes/:id` | Update node | ✅ Admin |
| `DELETE` | `/api/nodes/:id` | Delete node (also deletes connections) | ✅ Admin |

### 🔗 Connections (Decision Flow Edges)
| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/api/connections` | List connections (filterable by from/to node) | ✅ Admin |
| `POST` | `/api/connections` | Create connection between nodes | ✅ Admin |
| `PUT` | `/api/connections/:id` | Update connection | ✅ Admin |
| `DELETE` | `/api/connections/:id` | Delete connection | ✅ Admin |

---

## 🚦 Rate Limiting

API requests are limited to **100 requests per 60 seconds** per IP address.

**Headers returned:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1234567890
```

---

## 🔒 Security

| Feature | Implementation |
|---------|----------------|
| 🔑 Password Hashing | Argon2 (OWASP recommended) |
| 🎫 JWT Tokens | 24-hour expiration (configurable via `.env`) |
| 🔐 HTTPS | TLS 1.2+ enforced in production |
| 🛡️ Security Headers | HSTS, CSP, X-Frame-Options, X-Content-Type-Options |
| 🚦 Rate Limiting | 100 req/min per IP address |
| 💾 Caching | Aggressive caching with automatic invalidation |

---

## 📦 Response Format

All API responses follow a consistent structure:

### ✅ Success Response
```json
{
  \"id\": \"123e4567-e89b-12d3-a456-426614174000\",
  \"category\": \"brush\",
  \"text\": \"Is the brush worn?\",
  \"node_type\": \"Question\"
}
```

### ❌ Error Response
```json
{
  \"error\": \"Validation failed\",
  \"details\": [
    {
      \"field\": \"email\",
      \"message\": \"Invalid email format\"
    }
  ]
}
```

---

## 📊 HTTP Status Codes

| Code | Meaning | When It's Used |
|------|---------|----------------|
| `200 OK` | Success | Request completed successfully |
| `201 Created` | Created | New resource created |
| `400 Bad Request` | Client Error | Invalid request data |
| `401 Unauthorized` | Auth Required | Missing or invalid JWT token |
| `403 Forbidden` | Permission Denied | Valid token but insufficient permissions |
| `404 Not Found` | Not Found | Resource doesn't exist |
| `422 Unprocessable Entity` | Validation Error | Request data failed validation |
| `429 Too Many Requests` | Rate Limited | Exceeded rate limit |
| `500 Internal Server Error` | Server Error | Unexpected server error |

---

## 🚀 Performance & Caching

This API uses intelligent caching for optimal performance:

| Cache | TTL | Purpose |
|-------|-----|---------|
| **Questions Cache** | 5 minutes | Question/answer tree data |
| **Issue Tree Cache** | 10 minutes | Issue decision trees |
| **Issue Graph Cache** | 10 minutes | React Flow graph data |

Cache is automatically invalidated on mutations (create/update/delete).

---

## 🏗️ Architecture

### Decision Flow System (Current)
```
Issue Category → Nodes (Question/Conclusion) → Connections (Edges)
```

Nodes represent decision points or conclusions, and connections represent the flow between them. This powers the React Flow visual editor.

### Legacy Q&A System (Deprecated)
```
Issue → Questions → Answers → Next Question/Conclusion
```

The original question-answer tree system is still available but new issues should use the node-graph system.

---

## 💡 Quick Examples

### Example 1: Start a Troubleshooting Session
```bash
# Start session for \"brush\" issue
curl -X POST https://your-domain.com/api/troubleshoot/start \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"category\": \"brush\",
    \"tech_identifier\": \"TECH-001\",
    \"client_site\": \"Factory A\"
  }'
```

### Example 2: Create a New Issue
```bash
curl -X POST https://your-domain.com/api/admin/issues \\
  -H \"Authorization: Bearer YOUR_TOKEN\" \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"name\": \"Motor Problems\",
    \"category\": \"motor\",
    \"display_category\": \"Electrical\",
    \"root_question_text\": \"Is the motor making noise?\"
  }'
```

### Example 3: Get Dashboard Statistics
```bash
curl -X GET \"https://your-domain.com/api/admin/stats\" \\
  -H \"Authorization: Bearer YOUR_TOKEN\"
```

---

## 🛠️ Development

**Technology Stack:**
- **Backend:** Rust (Axum framework)
- **Database:** PostgreSQL with SQLx
- **Authentication:** JWT with Argon2 password hashing
- **Caching:** In-memory TTL cache
- **API Docs:** utoipa + Swagger UI

**Source Code:** Check the route handlers in `apps/api/src/routes/` for detailed implementation.
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
