use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;
use uuid::Uuid;

// ============================================
// USER MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export, export_to = "../../web/src/types/")]
pub enum UserRole {
    Admin,
    Viewer,
    Tech,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// QUESTION MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, FromRow)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct Question {
    pub id: Uuid,
    pub semantic_id: String,
    pub text: String,
    pub category: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CreateQuestion {
    pub semantic_id: String,
    pub text: String,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct UpdateQuestion {
    pub text: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================
// ANSWER MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, FromRow)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct Answer {
    pub id: Uuid,
    pub question_id: Uuid,
    pub label: String,
    pub next_question_id: Option<Uuid>,
    pub conclusion_text: Option<String>,
    pub order_index: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CreateAnswer {
    pub question_id: Uuid,
    pub label: String,
    pub next_question_id: Option<Uuid>,
    pub conclusion_text: Option<String>,
    pub order_index: i32,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct UpdateAnswer {
    pub label: Option<String>,
    pub next_question_id: Option<Uuid>,
    pub conclusion_text: Option<String>,
    pub order_index: Option<i32>,
    pub is_active: Option<bool>,
}

// ============================================
// QUERY RESPONSE MODELS
// ============================================

/// Response for question with its answers
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct QuestionWithAnswers {
    pub id: Uuid,
    pub semantic_id: String,
    pub text: String,
    pub category: Option<String>,
    pub answers: Vec<Answer>,
}

// ============================================
// NODE-GRAPH MODELS (New Architecture)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[ts(export, export_to = "../../web/src/types/")]
pub enum NodeType {
    Question,
    Conclusion,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct Node {
    pub id: Uuid,
    pub category: String,
    pub node_type: NodeType,
    pub text: String,
    pub semantic_id: Option<String>,
    pub display_category: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CreateNode {
    pub category: String,
    pub node_type: NodeType,
    pub text: String,
    pub semantic_id: Option<String>,
    pub display_category: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct UpdateNode {
    #[ts(optional)]
    pub text: Option<String>,
    #[ts(optional)]
    pub semantic_id: Option<String>,
    #[ts(optional)]
    pub node_type: Option<NodeType>,
    #[ts(optional)]
    pub display_category: Option<String>,
    #[ts(optional)]
    pub position_x: Option<f64>,
    #[ts(optional)]
    pub position_y: Option<f64>,
    #[ts(optional)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct Connection {
    pub id: Uuid,
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub label: String,
    pub order_index: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct CreateConnection {
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub label: String,
    pub order_index: i32,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct UpdateConnection {
    #[ts(optional)]
    pub to_node_id: Option<Uuid>,
    #[ts(optional)]
    pub label: Option<String>,
    #[ts(optional)]
    pub order_index: Option<i32>,
    #[ts(optional)]
    pub is_active: Option<bool>,
}

/// Node with its outgoing connections
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct NodeWithConnections {
    pub node: Node,
    pub connections: Vec<ConnectionWithTarget>,
}

/// Connection with target node information
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct ConnectionWithTarget {
    pub id: Uuid,
    pub label: String,
    pub order_index: i32,
    pub target_node: Node,
}

/// Complete graph for an issue category
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct IssueGraph {
    pub category: String,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}
