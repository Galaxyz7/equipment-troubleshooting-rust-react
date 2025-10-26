# API Documentation

## Base URL

```
Development: http://localhost:5000/api
Production: https://your-domain.com/api
```

## Interactive Documentation

The API includes **OpenAPI/Swagger UI** for interactive testing:

```
http://localhost:5000/swagger-ui
```

## Authentication

### JWT Authentication

Protected endpoints require a JWT token in the `Authorization` header:

```http
Authorization: Bearer <your-jwt-token>
```

### Login

**POST** `/api/auth/login`

**Request Body:**
```json
{
  "email": "admin@example.com",
  "password": "your-password"
}
```

**Response** (200 OK):
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "admin@example.com",
    "role": "Admin"
  }
}
```

**Token Expiry:** 24 hours

### Refresh Token

**POST** `/api/auth/refresh`

**Headers:**
```http
Authorization: Bearer <current-token>
```

**Response** (200 OK):
```json
{
  "token": "new-jwt-token..."
}
```

## Rate Limiting

- **Limit:** 100 requests per 60 seconds per IP address
- **Response** (429 Too Many Requests):
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests",
    "data": null
  }
}
```

**Headers:**
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1698765432
Retry-After: 60
```

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "data": {
      "field": ["Field-specific error"]
    }
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `NOT_FOUND` | 404 | Resource not found |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `BAD_REQUEST` | 400 | Invalid request |
| `VALIDATION_ERROR` | 422 | Validation failed |
| `CONFLICT` | 409 | Resource conflict |
| `INTERNAL_ERROR` | 500 | Server error |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |

## Public Endpoints

### Start Troubleshooting Session

**POST** `/api/troubleshoot/start`

**Request Body:**
```json
{
  "category": "hardware" // optional
}
```

**Response** (200 OK):
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "question": {
    "id": "q123",
    "text": "Is the power light on?",
    "answers": [
      { "id": "a1", "text": "Yes", "next_question_id": "q124" },
      { "id": "a2", "text": "No", "conclusion": "Check power connection" }
    ]
  },
  "history": []
}
```

### Submit Answer

**POST** `/api/troubleshoot/:session_id/answer`

**Request Body:**
```json
{
  "answer_id": "a1"
}
```

**Response** (200 OK):
```json
{
  "question": {
    "id": "q124",
    "text": "Do you see error lights?",
    "answers": [...]
  },
  "conclusion": null,
  "history": [
    {
      "question_id": "q123",
      "question_text": "Is the power light on?",
      "answer_id": "a1",
      "answer_text": "Yes"
    }
  ]
}
```

**Response - Conclusion Reached** (200 OK):
```json
{
  "question": null,
  "conclusion": "Replace the power supply unit",
  "history": [...]
}
```

### Get Session

**GET** `/api/troubleshoot/:session_id`

**Response** (200 OK):
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "current_question_id": "q124",
  "history": [...],
  "created_at": "2024-01-01T12:00:00Z"
}
```

## Admin Endpoints

All admin endpoints require authentication with `role: "Admin"`.

### Issues (Categories)

#### List All Issues

**GET** `/api/admin/issues`

**Response** (200 OK):
```json
[
  {
    "id": "1",
    "name": "Hardware Issues",
    "category": "hardware",
    "display_category": "Hardware",
    "root_question_id": "q1",
    "is_active": true,
    "question_count": 15,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Get Issue by Category

**GET** `/api/admin/issues/:category`

**Response** (200 OK):
```json
{
  "id": "1",
  "name": "Hardware Issues",
  "category": "hardware",
  "display_category": "Hardware",
  "root_question_id": "q1",
  "is_active": true,
  "question_count": 15,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Create Issue

**POST** `/api/admin/issues`

**Request Body:**
```json
{
  "name": "Electrical Issues",
  "category": "electrical",
  "display_category": "Electrical",
  "root_question_text": "Is there power to the device?"
}
```

**Response** (201 Created):
```json
{
  "id": "2",
  "name": "Electrical Issues",
  "category": "electrical",
  "display_category": "Electrical",
  "root_question_id": "q50",
  "is_active": true,
  "question_count": 1,
  "created_at": "2024-01-02T00:00:00Z",
  "updated_at": "2024-01-02T00:00:00Z"
}
```

#### Update Issue

**PUT** `/api/admin/issues/:category`

**Request Body:**
```json
{
  "name": "Electrical Problems",
  "display_category": "Electrical Systems"
}
```

**Response** (200 OK):
```json
{
  "id": "2",
  "name": "Electrical Problems",
  ...
}
```

#### Toggle Issue Active Status

**PATCH** `/api/admin/issues/:category/toggle`

**Request Body:**
```json
{
  "is_active": false
}
```

**Response** (200 OK):
```json
{
  "id": "2",
  "is_active": false,
  ...
}
```

#### Delete Issue

**DELETE** `/api/admin/issues/:category`

**Response** (204 No Content)

**Note:** Deletes the issue and all associated nodes/connections.

#### Get Issue Graph

**GET** `/api/admin/issues/:category/graph`

Returns the full decision tree graph for React Flow visualization.

**Response** (200 OK):
```json
{
  "nodes": [
    {
      "id": "n1",
      "category": "hardware",
      "node_type": "Question",
      "text": "Is the power on?",
      "semantic_id": "power_check",
      "is_active": true,
      "position_x": 100.0,
      "position_y": 50.0
    }
  ],
  "connections": [
    {
      "id": "c1",
      "from_node_id": "n1",
      "to_node_id": "n2",
      "label": "Yes",
      "order_index": 0
    }
  ]
}
```

#### Get Issue Tree

**GET** `/api/admin/issues/:category/tree`

Returns the decision tree in hierarchical format.

**Response** (200 OK):
```json
{
  "id": "n1",
  "text": "Is the power on?",
  "node_type": "Question",
  "answers": [
    {
      "text": "Yes",
      "next_node": {
        "id": "n2",
        "text": "Check error lights",
        "node_type": "Question",
        "answers": [...]
      }
    },
    {
      "text": "No",
      "conclusion": "Check power connection"
    }
  ]
}
```

### Nodes

#### Create Node

**POST** `/api/admin/nodes`

**Request Body:**
```json
{
  "category": "hardware",
  "node_type": "Question",
  "text": "Does the fan spin?",
  "semantic_id": "fan_check",
  "display_category": "Hardware",
  "position_x": 200.0,
  "position_y": 150.0
}
```

**Response** (201 Created):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "category": "hardware",
  "node_type": "Question",
  "text": "Does the fan spin?",
  "semantic_id": "fan_check",
  "display_category": "Hardware",
  "is_active": true,
  "position_x": 200.0,
  "position_y": 150.0,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Get Node

**GET** `/api/admin/nodes/:id`

#### Update Node

**PUT** `/api/admin/nodes/:id`

**Request Body:**
```json
{
  "text": "Updated question text",
  "position_x": 250.0,
  "position_y": 175.0
}
```

#### Delete Node

**DELETE** `/api/admin/nodes/:id`

**Response** (204 No Content)

**Note:** Also deletes all connections to/from this node.

### Connections

#### Create Connection

**POST** `/api/admin/connections`

**Request Body:**
```json
{
  "from_node_id": "n1",
  "to_node_id": "n2",
  "label": "Yes",
  "order_index": 0
}
```

**Response** (201 Created):
```json
{
  "id": "c1",
  "from_node_id": "n1",
  "to_node_id": "n2",
  "label": "Yes",
  "order_index": 0,
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Update Connection

**PUT** `/api/admin/connections/:id`

**Request Body:**
```json
{
  "label": "Updated answer text",
  "order_index": 1
}
```

#### Delete Connection

**DELETE** `/api/admin/connections/:id`

**Response** (204 No Content)

### Analytics

#### Get Dashboard Stats

**GET** `/api/admin/stats`

**Response** (200 OK):
```json
{
  "total_sessions": 1250,
  "completed_sessions": 980,
  "abandoned_sessions": 270,
  "active_sessions": 15,
  "avg_steps_to_completion": 4.2,
  "most_common_conclusions": [
    {
      "conclusion": "Replace power supply",
      "count": 145
    }
  ],
  "sessions_by_category": [
    {
      "category": "hardware",
      "count": 450
    }
  ]
}
```

#### Get Performance Metrics

**GET** `/api/admin/performance`

**Response** (200 OK):
```json
{
  "cache": {
    "questions_cache": {
      "entries": 8,
      "hit_rate": 0.85,
      "max_size": 10
    },
    "issue_tree_cache": {
      "entries": 12,
      "hit_rate": 0.78,
      "max_size": 50
    },
    "issue_graph_cache": {
      "entries": 5,
      "hit_rate": 0.92,
      "max_size": 50
    }
  },
  "database": {
    "connections": 8,
    "idle_connections": 5,
    "max_connections": 20
  }
}
```

## Request Examples

### cURL

```bash
# Login
curl -X POST http://localhost:5000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"password"}'

# Get issues (with auth)
curl -X GET http://localhost:5000/api/admin/issues \
  -H "Authorization: Bearer YOUR_TOKEN"

# Start session
curl -X POST http://localhost:5000/api/troubleshoot/start \
  -H "Content-Type: application/json" \
  -d '{"category":"hardware"}'
```

### JavaScript (Axios)

```javascript
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:5000/api',
});

// Login
const { data } = await api.post('/auth/login', {
  email: 'admin@example.com',
  password: 'password',
});

// Save token
localStorage.setItem('token', data.token);

// Authenticated request
api.defaults.headers.common['Authorization'] = `Bearer ${data.token}`;
const issues = await api.get('/admin/issues');
```

## Websockets

Currently not implemented. All communication is via REST API.

## Changelog

### v2.0.0
- Added node-graph architecture
- Added caching layer
- Added performance metrics endpoint
- Added security headers
- Added rate limiting
- Added OpenAPI documentation

### v1.0.0
- Initial release
- Question/answer model
- Basic CRUD operations

---

**Last Updated**: October 2025
**API Version**: 2.0.0
**OpenAPI Spec**: `/swagger-ui`
