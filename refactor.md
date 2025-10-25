# Node-Graph Architecture Refactor Plan

**Status**: 30% Complete (Database migration done, backend models created)
**Estimated Remaining Time**: 2-3 hours
**Last Updated**: 2025-10-25

## Table of Contents
1. [Overview](#overview)
2. [What's Been Completed](#whats-been-completed)
3. [Architecture Change](#architecture-change)
4. [Remaining Steps](#remaining-steps)
5. [Testing Strategy](#testing-strategy)
6. [Rollback Procedure](#rollback-procedure)

---

## Overview

### The Problem
The original architecture used a complex questions/answers model where:
- Questions had answers
- Answers pointed to either next questions OR had conclusion text
- The admin UI was confusing with "answers" as separate entities
- Creating/editing trees was overly complex

### The Solution
Refactor to a pure node-graph model where:
- **Nodes** represent both questions AND conclusions
- **Connections** are labeled edges between nodes
- Nodes with no outgoing connections are automatically conclusions
- Much simpler mental model: it's just a flowchart

---

## What's Been Completed

### ✅ 1. Database Migration
**File**: `apps/api/migrations/006_node_graph_refactor.sql`

Created two new tables:
```sql
-- nodes: Replaces questions + conclusion-type answers
CREATE TABLE nodes (
    id UUID PRIMARY KEY,
    category VARCHAR(255) NOT NULL,
    node_type VARCHAR(50) CHECK (node_type IN ('question', 'conclusion')),
    text TEXT NOT NULL,
    semantic_id VARCHAR(255),
    position_x FLOAT,
    position_y FLOAT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

-- connections: Replaces answers that pointed to next questions
CREATE TABLE connections (
    id UUID PRIMARY KEY,
    from_node_id UUID REFERENCES nodes(id) ON DELETE CASCADE,
    to_node_id UUID REFERENCES nodes(id) ON DELETE CASCADE,
    label VARCHAR(255) NOT NULL,
    order_index INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);
```

**Migration Results**:
- 39 nodes created (all questions + conclusion nodes)
- 39 connections created (all answer relationships)
- Old `questions` and `answers` tables still exist (for safety)

### ✅ 2. Rust Models
**File**: `apps/api/src/models.rs` (lines 220-330)

Created new model types:
- `NodeType` enum (Question | Conclusion)
- `Node` struct with position tracking
- `CreateNode`, `UpdateNode` DTOs
- `Connection` struct
- `CreateConnection`, `UpdateConnection` DTOs
- `NodeWithConnections` (node + outgoing edges)
- `ConnectionWithTarget` (connection + target node info)
- `IssueGraph` (complete graph for a category)

All models have `#[ts(export)]` for TypeScript generation.

### ✅ 3. Node API Routes
**File**: `apps/api/src/routes/nodes.rs`

Created CRUD endpoints:
- `GET /api/nodes` - List all nodes (filterable by category/type)
- `GET /api/nodes/:id` - Get specific node
- `POST /api/nodes` - Create new node
- `PUT /api/nodes/:id` - Update node (text, position, etc.)
- `DELETE /api/nodes/:id` - Soft delete node

---

## Architecture Change

### Old Architecture
```
User starts troubleshooting
    ↓
GET /api/troubleshoot/start
    ↓
Returns Question with semantic_id='start'
    ↓
Question has Answers
    ↓
Each Answer points to:
    - next_question_id (another Question)
    - OR conclusion_text (end of path)
```

### New Architecture
```
User starts troubleshooting
    ↓
GET /api/troubleshoot/start
    ↓
Returns Node with semantic_id='start' and node_type='question'
    ↓
Node has Connections
    ↓
Each Connection points to:
    - Another Node (could be question OR conclusion)
    - Connection.label is what user sees ("Yes", "No", etc.)
```

**Key Insight**: Conclusions are just nodes with `node_type='conclusion'` and no outgoing connections.

---

## Remaining Steps

### Step 1: Create Connections API Routes
**Priority**: HIGH
**Estimated Time**: 30 minutes
**File to Create**: `apps/api/src/routes/connections.rs`

Create CRUD endpoints for connections:

```rust
use crate::error::{ApiError, ApiResult};
use crate::models::{Connection, CreateConnection, UpdateConnection};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListConnectionsQuery {
    pub from_node_id: Option<Uuid>,
    pub to_node_id: Option<Uuid>,
}

/// GET /api/connections
/// List connections, optionally filtered by from/to node
pub async fn list_connections(
    State(state): State<AppState>,
    Query(query): Query<ListConnectionsQuery>,
) -> ApiResult<Json<Vec<Connection>>> {
    let mut sql = String::from(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE is_active = true"
    );

    let mut conditions = Vec::new();

    if let Some(from_id) = query.from_node_id {
        conditions.push(format!("from_node_id = '{}'", from_id));
    }

    if let Some(to_id) = query.to_node_id {
        conditions.push(format!("to_node_id = '{}'", to_id));
    }

    if !conditions.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY order_index ASC");

    let connections = sqlx::query_as::<_, Connection>(&sql)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(connections))
}

/// POST /api/connections
/// Create new connection (ADMIN only)
pub async fn create_connection(
    State(state): State<AppState>,
    Json(req): Json<CreateConnection>,
) -> ApiResult<Json<Connection>> {
    // Validate both nodes exist
    let from_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)"
    )
    .bind(req.from_node_id)
    .fetch_one(&state.db)
    .await?;

    let to_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)"
    )
    .bind(req.to_node_id)
    .fetch_one(&state.db)
    .await?;

    if !from_exists || !to_exists {
        return Err(ApiError::validation(vec![(
            "nodes".to_string(),
            "One or both nodes do not exist".to_string(),
        )]));
    }

    // Insert connection
    let connection = sqlx::query_as::<_, Connection>(
        "INSERT INTO connections (from_node_id, to_node_id, label, order_index, is_active)
         VALUES ($1, $2, $3, $4, true)
         RETURNING id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at"
    )
    .bind(req.from_node_id)
    .bind(req.to_node_id)
    .bind(&req.label)
    .bind(req.order_index)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(connection))
}

/// PUT /api/connections/:id
/// Update connection (ADMIN only)
pub async fn update_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConnection>,
) -> ApiResult<Json<Connection>> {
    // Check if connection exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM connections WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if !exists {
        return Err(ApiError::not_found("Connection not found"));
    }

    // If changing to_node_id, validate it exists
    if let Some(to_node_id) = req.to_node_id {
        let node_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1)"
        )
        .bind(to_node_id)
        .fetch_one(&state.db)
        .await?;

        if !node_exists {
            return Err(ApiError::validation(vec![(
                "to_node_id".to_string(),
                "Target node does not exist".to_string(),
            )]));
        }
    }

    // Build dynamic update query
    let mut query = String::from("UPDATE connections SET updated_at = NOW()");
    let mut param_count = 1;

    if req.to_node_id.is_some() {
        param_count += 1;
        query.push_str(&format!(", to_node_id = ${}", param_count));
    }
    if req.label.is_some() {
        param_count += 1;
        query.push_str(&format!(", label = ${}", param_count));
    }
    if req.order_index.is_some() {
        param_count += 1;
        query.push_str(&format!(", order_index = ${}", param_count));
    }
    if req.is_active.is_some() {
        param_count += 1;
        query.push_str(&format!(", is_active = ${}", param_count));
    }

    query.push_str(" WHERE id = $1 RETURNING id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at");

    let mut query_builder = sqlx::query_as::<_, Connection>(&query).bind(id);

    if let Some(to_node_id) = req.to_node_id {
        query_builder = query_builder.bind(to_node_id);
    }
    if let Some(label) = req.label {
        query_builder = query_builder.bind(label);
    }
    if let Some(order_index) = req.order_index {
        query_builder = query_builder.bind(order_index);
    }
    if let Some(is_active) = req.is_active {
        query_builder = query_builder.bind(is_active);
    }

    let connection = query_builder.fetch_one(&state.db).await?;

    Ok(Json(connection))
}

/// DELETE /api/connections/:id
/// Soft delete connection (ADMIN only)
pub async fn delete_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Connection>> {
    let connection = sqlx::query_as::<_, Connection>(
        "UPDATE connections
         SET is_active = false, updated_at = NOW()
         WHERE id = $1
         RETURNING id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    Ok(Json(connection))
}
```

---

### Step 2: Create Graph API Endpoint
**Priority**: HIGH
**Estimated Time**: 45 minutes
**File to Modify**: `apps/api/src/routes/issues.rs`

Add new endpoint to get complete graph for a category:

```rust
/// GET /api/admin/issues/:category/graph
/// Get complete node graph for an issue category
pub async fn get_issue_graph(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> ApiResult<Json<IssueGraph>> {
    // Get all nodes in this category
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE category = $1
         ORDER BY created_at ASC"
    )
    .bind(&category)
    .fetch_all(&state.db)
    .await?;

    if nodes.is_empty() {
        return Err(ApiError::not_found("Issue category not found"));
    }

    // Get all node IDs
    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();

    // Get all connections between these nodes
    let connections = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE from_node_id = ANY($1)
         ORDER BY order_index ASC"
    )
    .bind(&node_ids)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(IssueGraph {
        category,
        nodes,
        connections,
    }))
}
```

**Also add** helper endpoint to get node with its connections:

```rust
/// GET /api/nodes/:id/with-connections
/// Get a node with all its outgoing connections and target node details
pub async fn get_node_with_connections(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<NodeWithConnections>> {
    // Get the node
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Node not found"))?;

    // Get connections with target nodes
    let connections_with_targets = sqlx::query!(
        r#"
        SELECT
            c.id,
            c.label,
            c.order_index,
            n.id as target_id,
            n.category as target_category,
            n.node_type as target_node_type,
            n.text as target_text,
            n.semantic_id as target_semantic_id,
            n.position_x as target_position_x,
            n.position_y as target_position_y,
            n.is_active as target_is_active,
            n.created_at as target_created_at,
            n.updated_at as target_updated_at
        FROM connections c
        JOIN nodes n ON c.to_node_id = n.id
        WHERE c.from_node_id = $1 AND c.is_active = true
        ORDER BY c.order_index ASC
        "#,
        id
    )
    .fetch_all(&state.db)
    .await?;

    let connections = connections_with_targets
        .into_iter()
        .map(|row| {
            ConnectionWithTarget {
                id: row.id,
                label: row.label,
                order_index: row.order_index,
                target_node: Node {
                    id: row.target_id,
                    category: row.target_category,
                    node_type: match row.target_node_type.as_str() {
                        "question" => NodeType::Question,
                        "conclusion" => NodeType::Conclusion,
                        _ => NodeType::Question,
                    },
                    text: row.target_text,
                    semantic_id: row.target_semantic_id,
                    position_x: row.target_position_x,
                    position_y: row.target_position_y,
                    is_active: row.target_is_active,
                    created_at: row.target_created_at,
                    updated_at: row.target_updated_at,
                },
            }
        })
        .collect();

    Ok(Json(NodeWithConnections {
        node,
        connections,
    }))
}
```

---

### Step 3: Update Troubleshoot Logic
**Priority**: HIGH
**Estimated Time**: 1 hour
**File to Modify**: `apps/api/src/routes/troubleshoot.rs`

The troubleshoot flow needs to work with nodes instead of questions.

**Current logic**:
```rust
// POST /api/troubleshoot/start
// Returns Question with semantic_id='start'
// Question has answers
```

**New logic**:
```rust
// POST /api/troubleshoot/start
// Returns Node with semantic_id='start'
// Node has connections to other nodes
```

**Key changes needed**:

1. **Start session** - Find root node instead of root question:
```rust
pub async fn start_session(
    State(state): State<AppState>,
    Json(_req): Json<StartSessionRequest>,
) -> ApiResult<Json<StartSessionResponse>> {
    // Find the root node (semantic_id='start')
    let root_node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE semantic_id = 'start' AND is_active = true"
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Root node not found"))?;

    // Get connections from root node
    let connections = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE from_node_id = $1 AND is_active = true
         ORDER BY order_index ASC"
    )
    .bind(root_node.id)
    .fetch_all(&state.db)
    .await?;

    // For each connection, get the target node
    let mut options = Vec::new();
    for conn in connections {
        let target_node = sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE id = $1"
        )
        .bind(conn.to_node_id)
        .fetch_one(&state.db)
        .await?;

        options.push(NavigationOption {
            connection_id: conn.id,
            label: conn.label,
            target_category: target_node.category,
        });
    }

    // Generate session ID
    let session_id = Uuid::new_v4().to_string();

    Ok(Json(StartSessionResponse {
        session_id,
        node: root_node,
        options,
    }))
}
```

2. **Submit answer** - Navigate via connections instead of answers:
```rust
pub async fn submit_answer(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<SubmitAnswerRequest>,
) -> ApiResult<Json<SubmitAnswerResponse>> {
    // Get the connection
    let connection = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE id = $1 AND is_active = true"
    )
    .bind(req.connection_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Connection not found"))?;

    // Get the target node
    let next_node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1"
    )
    .bind(connection.to_node_id)
    .fetch_one(&state.db)
    .await?;

    // Check if this is a conclusion node
    if matches!(next_node.node_type, NodeType::Conclusion) {
        // Terminal node - end of path
        return Ok(Json(SubmitAnswerResponse {
            session_id,
            node: next_node.clone(),
            options: vec![],
            is_conclusion: true,
            conclusion_text: Some(next_node.text),
        }));
    }

    // Get next connections (if question node)
    let connections = sqlx::query_as::<_, Connection>(
        "SELECT id, from_node_id, to_node_id, label, order_index, is_active, created_at, updated_at
         FROM connections
         WHERE from_node_id = $1 AND is_active = true
         ORDER BY order_index ASC"
    )
    .bind(next_node.id)
    .fetch_all(&state.db)
    .await?;

    let mut options = Vec::new();
    for conn in connections {
        let target_node = sqlx::query_as::<_, Node>(
            "SELECT id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at
             FROM nodes
             WHERE id = $1"
        )
        .bind(conn.to_node_id)
        .fetch_one(&state.db)
        .await?;

        options.push(NavigationOption {
            connection_id: conn.id,
            label: conn.label,
            target_category: target_node.category,
        });
    }

    Ok(Json(SubmitAnswerResponse {
        session_id,
        node: next_node,
        options,
        is_conclusion: false,
        conclusion_text: None,
    }))
}
```

3. **Update response models**:
```rust
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct StartSessionResponse {
    pub session_id: String,
    pub node: Node,
    pub options: Vec<NavigationOption>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct NavigationOption {
    pub connection_id: Uuid,
    pub label: String,
    pub target_category: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct SubmitAnswerRequest {
    pub connection_id: Uuid,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct SubmitAnswerResponse {
    pub session_id: String,
    pub node: Node,
    pub options: Vec<NavigationOption>,
    pub is_conclusion: bool,
    pub conclusion_text: Option<String>,
}
```

---

### Step 4: Register Routes in Main
**Priority**: HIGH
**Estimated Time**: 15 minutes
**File to Modify**: `apps/api/src/main.rs`

Update the router to include new routes:

```rust
mod routes {
    pub mod auth;
    pub mod troubleshoot;
    pub mod admin;
    pub mod questions;
    pub mod answers;
    pub mod issues;
    pub mod nodes;        // NEW
    pub mod connections;  // NEW
}

// In the router setup:
let app = Router::new()
    // ... existing routes ...

    // NEW NODE ROUTES (ADMIN ONLY)
    .route("/api/nodes", get(routes::nodes::list_nodes))
    .route("/api/nodes/:id", get(routes::nodes::get_node))
    .route("/api/nodes/:id/with-connections", get(routes::nodes::get_node_with_connections))
    .route("/api/nodes", post(routes::nodes::create_node))
    .route("/api/nodes/:id", put(routes::nodes::update_node))
    .route("/api/nodes/:id", delete(routes::nodes::delete_node))
    .layer(from_fn_with_state(app_state.clone(), require_admin))

    // NEW CONNECTION ROUTES (ADMIN ONLY)
    .route("/api/connections", get(routes::connections::list_connections))
    .route("/api/connections", post(routes::connections::create_connection))
    .route("/api/connections/:id", put(routes::connections::update_connection))
    .route("/api/connections/:id", delete(routes::connections::delete_connection))
    .layer(from_fn_with_state(app_state.clone(), require_admin))

    // NEW GRAPH ENDPOINT (ADMIN ONLY)
    .route("/api/admin/issues/:category/graph", get(routes::issues::get_issue_graph))
    .layer(from_fn_with_state(app_state.clone(), require_admin))

    // ... rest of routes ...
    .with_state(app_state);
```

---

### Step 5: Generate TypeScript Types
**Priority**: HIGH
**Estimated Time**: 10 minutes

Run the TypeScript generation:

```bash
cd apps/api
cargo test --lib -- --nocapture
```

This will generate TypeScript types from all models with `#[ts(export)]`.

**Verify these files are generated**:
- `apps/web/src/types/Node.ts`
- `apps/web/src/types/NodeType.ts`
- `apps/web/src/types/Connection.ts`
- `apps/web/src/types/CreateNode.ts`
- `apps/web/src/types/UpdateNode.ts`
- `apps/web/src/types/CreateConnection.ts`
- `apps/web/src/types/UpdateConnection.ts`
- `apps/web/src/types/NodeWithConnections.ts`
- `apps/web/src/types/ConnectionWithTarget.ts`
- `apps/web/src/types/IssueGraph.ts`

---

### Step 6: Update Frontend API Client
**Priority**: HIGH
**Estimated Time**: 30 minutes
**File to Modify**: `apps/web/src/lib/api.ts`

Add new API client functions:

```typescript
import type {
  Node,
  CreateNode,
  UpdateNode,
  Connection,
  CreateConnection,
  UpdateConnection,
  IssueGraph,
  NodeWithConnections,
} from '../types';

export const nodesAPI = {
  list: async (category?: string, nodeType?: string): Promise<Node[]> => {
    const params = new URLSearchParams();
    if (category) params.append('category', category);
    if (nodeType) params.append('node_type', nodeType);

    const { data } = await api.get<Node[]>(`/api/nodes?${params.toString()}`);
    return data;
  },

  get: async (id: string): Promise<Node> => {
    const { data } = await api.get<Node>(`/api/nodes/${id}`);
    return data;
  },

  getWithConnections: async (id: string): Promise<NodeWithConnections> => {
    const { data } = await api.get<NodeWithConnections>(`/api/nodes/${id}/with-connections`);
    return data;
  },

  create: async (node: CreateNode): Promise<Node> => {
    const { data } = await api.post<Node>('/api/nodes', node);
    return data;
  },

  update: async (id: string, updates: UpdateNode): Promise<Node> => {
    const { data } = await api.put<Node>(`/api/nodes/${id}`, updates);
    return data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/api/nodes/${id}`);
  },
};

export const connectionsAPI = {
  list: async (fromNodeId?: string, toNodeId?: string): Promise<Connection[]> => {
    const params = new URLSearchParams();
    if (fromNodeId) params.append('from_node_id', fromNodeId);
    if (toNodeId) params.append('to_node_id', toNodeId);

    const { data } = await api.get<Connection[]>(`/api/connections?${params.toString()}`);
    return data;
  },

  create: async (connection: CreateConnection): Promise<Connection> => {
    const { data } = await api.post<Connection>('/api/connections', connection);
    return data;
  },

  update: async (id: string, updates: UpdateConnection): Promise<Connection> => {
    const { data } = await api.put<Connection>(`/api/connections/${id}`, updates);
    return data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/api/connections/${id}`);
  },
};

// Update issuesAPI to add graph endpoint
export const issuesAPI = {
  // ... existing methods ...

  getGraph: async (category: string): Promise<IssueGraph> => {
    const { data } = await api.get<IssueGraph>(`/api/admin/issues/${category}/graph`);
    return data;
  },
};
```

---

### Step 7: Update Troubleshoot Frontend
**Priority**: MEDIUM
**Estimated Time**: 30 minutes
**Files to Modify**:
- `apps/web/src/pages/TroubleshootPage.tsx`
- `apps/web/src/types/troubleshoot.ts`

Update the troubleshoot flow to use new response structure:

**Update types**:
```typescript
// apps/web/src/types/troubleshoot.ts
export interface StartSessionRequest {
  // Empty for now, might add metadata later
}

export interface StartSessionResponse {
  session_id: string;
  node: Node;
  options: NavigationOption[];
}

export interface NavigationOption {
  connection_id: string;
  label: string;
  target_category: string;
}

export interface SubmitAnswerRequest {
  connection_id: string;
}

export interface SubmitAnswerResponse {
  session_id: string;
  node: Node;
  options: NavigationOption[];
  is_conclusion: boolean;
  conclusion_text?: string;
}
```

**Update TroubleshootPage.tsx**:
```typescript
// Change state from Question to Node
const [currentNode, setCurrentNode] = useState<Node | null>(null);
const [options, setOptions] = useState<NavigationOption[]>([]);

// Update start session
const handleStart = async () => {
  try {
    const response = await troubleshootAPI.startSession({});
    setSessionId(response.session_id);
    setCurrentNode(response.node);
    setOptions(response.options);
  } catch (err) {
    // error handling
  }
};

// Update answer submission
const handleAnswer = async (connectionId: string) => {
  try {
    const response = await troubleshootAPI.submitAnswer(sessionId, {
      connection_id: connectionId,
    });

    setCurrentNode(response.node);
    setOptions(response.options);

    if (response.is_conclusion) {
      // Show conclusion
      setConclusionText(response.conclusion_text);
    }
  } catch (err) {
    // error handling
  }
};

// Update rendering
return (
  <div>
    {currentNode && (
      <>
        <h2>{currentNode.text}</h2>
        {currentNode.node_type === 'conclusion' ? (
          <div className="conclusion">
            <p>{currentNode.text}</p>
          </div>
        ) : (
          <div className="options">
            {options.map(opt => (
              <button
                key={opt.connection_id}
                onClick={() => handleAnswer(opt.connection_id)}
              >
                {opt.label}
              </button>
            ))}
          </div>
        )}
      </>
    )}
  </div>
);
```

---

### Step 8: Refactor TreeEditorModal (Most Complex)
**Priority**: MEDIUM
**Estimated Time**: 1 hour
**File to Modify**: `apps/web/src/components/TreeEditorModal.tsx`

This is the biggest UI change. The new UI should be **node-centric** instead of question/answer-centric.

**Key Changes**:

1. **Load graph instead of tree**:
```typescript
const loadGraph = async () => {
  setLoading(true);
  setError(null);
  try {
    const graph = await issuesAPI.getGraph(category);
    setGraphData(graph);
    convertGraphToFlow(graph);
  } catch (err: any) {
    setError(`Failed to load graph: ${err.message}`);
  } finally {
    setLoading(false);
  }
};
```

2. **Convert graph to React Flow format**:
```typescript
const convertGraphToFlow = (graph: IssueGraph) => {
  const flowNodes: Node[] = [];
  const flowEdges: Edge[] = [];

  // Load saved positions
  const layoutKey = `graph_layout_${category}`;
  const savedPositions = localStorage.getItem(layoutKey);
  const nodePositions: Record<string, { x: number; y: number }> = savedPositions
    ? JSON.parse(savedPositions)
    : {};

  // Create React Flow nodes from graph nodes
  graph.nodes.forEach((node, index) => {
    const savedPos = nodePositions[node.id] || node.position_x && node.position_y
      ? { x: node.position_x, y: node.position_y }
      : null;

    const x = savedPos?.x ?? (index % 3) * 350;
    const y = savedPos?.y ?? Math.floor(index / 3) * 200;

    flowNodes.push({
      id: node.id,
      type: node.node_type === 'conclusion' ? 'output' : 'default',
      position: { x, y },
      data: {
        label: (
          <div className="p-2">
            <div className="font-semibold text-sm mb-1">
              {node.node_type === 'conclusion' ? '🎯 ' : '❓ '}
              {node.semantic_id || 'Node'}
            </div>
            <div className="text-xs text-gray-600">
              {node.text.substring(0, 50)}...
            </div>
          </div>
        )
      },
      style: {
        background: node.node_type === 'conclusion' ? '#dcfce7' : '#fff',
        border: '2px solid ' + (node.node_type === 'conclusion' ? '#16a34a' : '#667eea'),
        borderRadius: '8px',
        width: 250,
      },
    });
  });

  // Create React Flow edges from connections
  graph.connections.forEach(connection => {
    flowEdges.push({
      id: connection.id,
      source: connection.from_node_id,
      target: connection.to_node_id,
      label: connection.label,
      type: 'smoothstep',
      animated: true,
      markerEnd: {
        type: MarkerType.ArrowClosed,
      },
    });
  });

  setNodes(flowNodes);
  setEdges(flowEdges);
};
```

3. **Simplified Edit Panel** (no more "answers" section):
```typescript
{selectedNode && graphData && (
  <div className="p-4">
    {(() => {
      const node = graphData.nodes.find(n => n.id === selectedNode);
      if (!node) return null;

      return (
        <>
          <h3 className="font-bold mb-4">
            {node.node_type === 'conclusion' ? '🎯 Conclusion' : '❓ Question'}
          </h3>

          {/* Node Type Badge */}
          <div className="mb-4">
            <span className={`px-2 py-1 rounded text-xs font-medium ${
              node.node_type === 'conclusion'
                ? 'bg-green-100 text-green-800'
                : 'bg-blue-100 text-blue-800'
            }`}>
              {node.node_type}
            </span>
          </div>

          {/* Semantic ID */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Semantic ID
            </label>
            <input
              type="text"
              value={node.semantic_id || ''}
              onChange={(e) => handleUpdateNode(node.id, { semantic_id: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="e.g., brush_worn"
            />
          </div>

          {/* Node Text */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {node.node_type === 'conclusion' ? 'Conclusion Text' : 'Question Text'}
            </label>
            <textarea
              value={node.text}
              onChange={(e) => handleUpdateNode(node.id, { text: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md resize-none h-24"
              placeholder={node.node_type === 'conclusion'
                ? "Enter what to do next..."
                : "Enter the question..."
              }
            />
          </div>

          {/* Outgoing Connections (only for question nodes) */}
          {node.node_type === 'question' && (
            <>
              <div className="mb-4">
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Connections ({outgoingConnections.length})
                </label>
                <div className="space-y-2">
                  {outgoingConnections.map((conn, index) => {
                    const targetNode = graphData.nodes.find(n => n.id === conn.to_node_id);
                    return (
                      <div key={conn.id} className="border rounded-md p-3 bg-gray-50">
                        <div className="flex justify-between items-start mb-2">
                          <span className="text-sm font-medium">Option {index + 1}</span>
                          <button
                            onClick={() => handleDeleteConnection(conn.id)}
                            className="text-red-500 text-xs hover:text-red-700"
                          >
                            Delete
                          </button>
                        </div>

                        {/* Connection Label */}
                        <label className="block text-xs text-gray-600 mb-1">Label:</label>
                        <input
                          type="text"
                          value={conn.label}
                          onChange={(e) => handleUpdateConnection(conn.id, { label: e.target.value })}
                          className="w-full px-2 py-1 border rounded text-sm mb-2"
                          placeholder="e.g., Yes, No, Worn..."
                        />

                        {/* Target Node Selector */}
                        <label className="block text-xs text-gray-600 mb-1">Goes to:</label>
                        <select
                          value={conn.to_node_id}
                          onChange={(e) => handleUpdateConnection(conn.id, { to_node_id: e.target.value })}
                          className="w-full px-2 py-1 border rounded text-sm"
                        >
                          {graphData.nodes
                            .filter(n => n.id !== node.id) // Can't connect to self
                            .map(n => (
                              <option key={n.id} value={n.id}>
                                {n.node_type === 'conclusion' ? '🎯 ' : '❓ '}
                                {n.semantic_id || n.text.substring(0, 30)}...
                              </option>
                            ))
                          }
                        </select>

                        {targetNode && (
                          <div className="mt-2 text-xs text-gray-500">
                            Preview: {targetNode.text.substring(0, 40)}...
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>

                {/* Add Connection Button */}
                <button
                  onClick={() => handleAddConnection(node.id)}
                  className="mt-3 w-full px-3 py-2 rounded-md bg-blue-500 text-white text-sm font-medium hover:bg-blue-600"
                >
                  + Add Connection
                </button>
              </div>
            </>
          )}

          {/* Delete Node Button */}
          <button
            onClick={() => handleDeleteNode(node.id)}
            className="w-full px-3 py-2 rounded-md bg-red-500 text-white font-medium hover:bg-red-600"
          >
            Delete Node
          </button>
        </>
      );
    })()}
  </div>
)}
```

4. **Update CRUD handlers**:
```typescript
const handleUpdateNode = async (nodeId: string, updates: UpdateNode) => {
  if (!graphData) return;

  try {
    await nodesAPI.update(nodeId, updates);
    await loadGraph(); // Reload to get fresh data
    setHasChanges(false);
  } catch (err: any) {
    setError(`Failed to update node: ${err.message}`);
  }
};

const handleAddConnection = async (fromNodeId: string) => {
  if (!graphData) return;

  const label = prompt('Enter connection label (e.g., "Yes", "No"):');
  if (!label) return;

  // For now, connect to first available node (user can change via dropdown)
  const targetNode = graphData.nodes.find(n => n.id !== fromNodeId);
  if (!targetNode) {
    alert('Create another node first!');
    return;
  }

  try {
    await connectionsAPI.create({
      from_node_id: fromNodeId,
      to_node_id: targetNode.id,
      label,
      order_index: outgoingConnections.length,
    });
    await loadGraph();
    setHasChanges(false);
  } catch (err: any) {
    setError(`Failed to add connection: ${err.message}`);
  }
};

const handleUpdateConnection = async (connId: string, updates: UpdateConnection) => {
  try {
    await connectionsAPI.update(connId, updates);
    await loadGraph();
    setHasChanges(false);
  } catch (err: any) {
    setError(`Failed to update connection: ${err.message}`);
  }
};

const handleDeleteConnection = async (connId: string) => {
  if (!confirm('Delete this connection?')) return;

  try {
    await connectionsAPI.delete(connId);
    await loadGraph();
    setHasChanges(false);
  } catch (err: any) {
    setError(`Failed to delete connection: ${err.message}`);
  }
};

const handleDeleteNode = async (nodeId: string) => {
  const node = graphData?.nodes.find(n => n.id === nodeId);
  if (!node) return;

  if (!confirm(`Delete node "${node.text}"? This will also delete all connections.`)) {
    return;
  }

  try {
    await nodesAPI.delete(nodeId);
    await loadGraph();
    setSelectedNode(null);
    setHasChanges(false);
  } catch (err: any) {
    setError(`Failed to delete node: ${err.message}`);
  }
};
```

5. **Add "Create Node" button in header**:
```typescript
const handleCreateNode = async () => {
  const nodeType = confirm('Create a Question node? (Cancel for Conclusion)')
    ? 'question'
    : 'conclusion';

  const text = prompt(`Enter ${nodeType} text:`);
  if (!text) return;

  const semanticId = prompt('Enter semantic ID (optional):');

  try {
    await nodesAPI.create({
      category,
      node_type: nodeType,
      text,
      semantic_id: semanticId || undefined,
      position_x: undefined,
      position_y: undefined,
    });
    await loadGraph();
    setHasChanges(false);
  } catch (err: any) {
    setError(`Failed to create node: ${err.message}`);
  }
};

// In header:
<button
  onClick={handleCreateNode}
  className="px-4 py-2 rounded-md bg-green-500 text-white"
>
  ➕ New Node
</button>
```

---

### Step 9: Update Issue Creation
**Priority**: LOW
**Estimated Time**: 20 minutes
**File to Modify**: `apps/api/src/routes/issues.rs`

Update the create_issue function to create a node instead of a question:

```rust
pub async fn create_issue(
    State(state): State<AppState>,
    Json(req): Json<CreateIssueRequest>,
) -> ApiResult<Json<Issue>> {
    // Validate category is unique
    let existing = sqlx::query!(
        "SELECT COUNT(*) as count FROM nodes WHERE category = $1",
        &req.category
    )
    .fetch_one(&state.db)
    .await?;

    if existing.count.unwrap_or(0) > 0 {
        return Err(ApiError::validation(vec![(
            "category".to_string(),
            "Category already exists".to_string(),
        )]));
    }

    // Create root node (question type)
    let node_id = Uuid::new_v4();
    let semantic_id = format!("{}_start", req.category);

    sqlx::query!(
        "INSERT INTO nodes (id, category, node_type, text, semantic_id, is_active)
         VALUES ($1, $2, 'question', $3, $4, true)",
        node_id,
        &req.category,
        &req.root_question_text,
        &semantic_id
    )
    .execute(&state.db)
    .await?;

    // Fetch the created node
    let node = sqlx::query_as::<_, Node>(
        "SELECT id, category, node_type, text, semantic_id, position_x, position_y, is_active, created_at, updated_at
         FROM nodes
         WHERE id = $1",
    )
    .bind(node_id)
    .fetch_one(&state.db)
    .await?;

    // Automatically link to root node (semantic_id='start')
    let root_node = sqlx::query!(
        "SELECT id FROM nodes WHERE semantic_id = 'start'"
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(root) = root_node {
        // Get connection count for order_index
        let conn_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM connections WHERE from_node_id = $1",
            root.id
        )
        .fetch_one(&state.db)
        .await?;

        // Create connection from root to this new issue's start node
        sqlx::query!(
            "INSERT INTO connections (from_node_id, to_node_id, label, order_index, is_active)
             VALUES ($1, $2, $3, $4, true)",
            root.id,
            node_id,
            &req.name,
            conn_count.count.unwrap_or(0) as i32
        )
        .execute(&state.db)
        .await?;
    }

    Ok(Json(Issue {
        id: node.id.to_string(),
        name: req.name,
        category: req.category,
        root_question_id: node.id.to_string(),
        is_active: true,
        question_count: 1,
        created_at: node.created_at.to_rfc3339(),
        updated_at: node.updated_at.to_rfc3339(),
    }))
}
```

---

## Testing Strategy

### Phase 1: Backend API Testing
Test each new endpoint using curl or Postman:

1. **Test Node CRUD**:
```bash
# List nodes
curl http://localhost:5000/api/nodes

# Get specific node
curl http://localhost:5000/api/nodes/{id}

# Create node
curl -X POST http://localhost:5000/api/nodes \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {token}" \
  -d '{"category":"test","node_type":"question","text":"Test question?"}'

# Update node
curl -X PUT http://localhost:5000/api/nodes/{id} \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {token}" \
  -d '{"text":"Updated text"}'

# Delete node
curl -X DELETE http://localhost:5000/api/nodes/{id} \
  -H "Authorization: Bearer {token}"
```

2. **Test Connection CRUD**:
```bash
# Create connection
curl -X POST http://localhost:5000/api/connections \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {token}" \
  -d '{"from_node_id":"...","to_node_id":"...","label":"Yes","order_index":0}'

# List connections
curl http://localhost:5000/api/connections?from_node_id={id}

# Update connection
curl -X PUT http://localhost:5000/api/connections/{id} \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {token}" \
  -d '{"label":"Updated label"}'
```

3. **Test Graph Endpoint**:
```bash
curl http://localhost:5000/api/admin/issues/brush/graph \
  -H "Authorization: Bearer {token}"
```

4. **Test Troubleshoot Flow**:
```bash
# Start session
curl -X POST http://localhost:5000/api/troubleshoot/start \
  -H "Content-Type: application/json" \
  -d '{}'

# Submit answer
curl -X POST http://localhost:5000/api/troubleshoot/{session_id}/answer \
  -H "Content-Type: application/json" \
  -d '{"connection_id":"..."}'
```

### Phase 2: Frontend Testing

1. **Admin Panel**:
   - Can view issues
   - Can create new issue → verify it creates node and auto-links to root
   - Can toggle issue on/off
   - Can open tree editor

2. **Tree Editor**:
   - Opens and displays graph correctly
   - Can select nodes
   - Can edit node text
   - Can add new nodes (question and conclusion types)
   - Can add connections between nodes
   - Can edit connection labels
   - Can change connection targets via dropdown
   - Can delete connections
   - Can delete nodes
   - Can drag nodes to rearrange
   - Positions persist after save and reload

3. **User Flow**:
   - Can start troubleshooting
   - Can see options (connection labels)
   - Can click through decision tree
   - Reaches conclusion nodes correctly
   - Conclusion text displays properly

### Phase 3: Data Integrity Testing

Verify the migration worked correctly:

```sql
-- Check all questions became nodes
SELECT
  (SELECT COUNT(*) FROM questions) as old_questions,
  (SELECT COUNT(*) FROM nodes WHERE node_type = 'question') as new_question_nodes;

-- Check all answers became connections or conclusion nodes
SELECT
  (SELECT COUNT(*) FROM answers) as old_answers,
  (SELECT COUNT(*) FROM connections) + (SELECT COUNT(*) FROM nodes WHERE node_type = 'conclusion') as new_total;

-- Check orphaned nodes (nodes with no incoming connections except root)
SELECT n.*
FROM nodes n
LEFT JOIN connections c ON n.id = c.to_node_id
WHERE c.id IS NULL
  AND n.semantic_id != 'start'
  AND n.node_type = 'question';

-- Check broken connections (pointing to deleted nodes)
SELECT c.*
FROM connections c
LEFT JOIN nodes n ON c.to_node_id = n.id
WHERE n.id IS NULL;
```

---

## Rollback Procedure

If you need to rollback the migration:

### Option 1: Revert to Old Tables
The old `questions` and `answers` tables still exist. To revert:

1. **Drop new tables**:
```sql
DROP TABLE IF EXISTS connections;
DROP TABLE IF EXISTS nodes;
```

2. **Revert code changes**:
```bash
git checkout apps/api/src/routes/nodes.rs
git checkout apps/api/src/routes/connections.rs
git checkout apps/api/src/models.rs
# ... revert other changed files
```

3. **Restart backend** - it will use old schema

### Option 2: Keep Both Schemas
You can keep both schemas and gradually migrate:
- New admin UI uses node-graph
- User-facing troubleshoot still uses questions/answers
- Sync changes between schemas with triggers or cron jobs

---

## Success Criteria

The refactor is complete when:

- ✅ All backend routes compile and run without errors
- ✅ TypeScript types are generated correctly
- ✅ Frontend compiles without type errors
- ✅ Can create new issue from admin panel
- ✅ New issue appears on troubleshoot start page
- ✅ Can open tree editor and see node graph
- ✅ Can add/edit/delete nodes and connections in tree editor
- ✅ Can complete full troubleshoot flow from start to conclusion
- ✅ Layout positions persist after save/reload
- ✅ All existing data migrated correctly (no broken links)

---

## Notes & Tips

1. **Work incrementally**: Complete and test each step before moving to the next
2. **Keep old tables**: Don't drop `questions` and `answers` until 100% confident
3. **Test thoroughly**: The troubleshoot flow is critical - test with real users
4. **UI/UX polish**: After basic functionality works, improve the tree editor UX
5. **Consider React Flow Pro**: For advanced features like auto-layout, edge editing, etc.

---

## Current File State

**Created**:
- `apps/api/migrations/006_node_graph_refactor.sql` ✅
- `apps/api/src/bin/run_migration.rs` ✅
- `apps/api/src/routes/nodes.rs` ✅
- `apps/api/src/models.rs` (appended node-graph models) ✅

**Need to Create**:
- `apps/api/src/routes/connections.rs`

**Need to Modify**:
- `apps/api/src/routes/issues.rs` (add get_issue_graph, update create_issue)
- `apps/api/src/routes/troubleshoot.rs` (update to use nodes/connections)
- `apps/api/src/main.rs` (register new routes)
- `apps/web/src/lib/api.ts` (add nodesAPI, connectionsAPI)
- `apps/web/src/components/TreeEditorModal.tsx` (major refactor)
- `apps/web/src/pages/TroubleshootPage.tsx` (update to use new response types)
- `apps/web/src/types/troubleshoot.ts` (update types)

---

## Estimated Time Breakdown

| Task | Time | Status |
|------|------|--------|
| Database migration | 30 min | ✅ Done |
| Rust models | 20 min | ✅ Done |
| Node routes | 30 min | ✅ Done |
| Connection routes | 30 min | ⏳ Pending |
| Graph endpoint | 45 min | ⏳ Pending |
| Troubleshoot logic | 1 hour | ⏳ Pending |
| Register routes | 15 min | ⏳ Pending |
| Generate TS types | 10 min | ⏳ Pending |
| Frontend API client | 30 min | ⏳ Pending |
| Troubleshoot frontend | 30 min | ⏳ Pending |
| TreeEditorModal refactor | 1 hour | ⏳ Pending |
| Testing & debugging | 1 hour | ⏳ Pending |
| **Total** | **~6 hours** | **30% done** |

---

**Good luck with the refactor! Take it step by step and test thoroughly.** 🚀
