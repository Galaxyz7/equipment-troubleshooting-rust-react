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
#[ts(export, export_to = "../web/src/types/")]
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

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}

// ============================================
// QUESTION MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, FromRow)]
#[ts(export, export_to = "../web/src/types/")]
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
#[ts(export, export_to = "../web/src/types/")]
pub struct CreateQuestion {
    pub semantic_id: String,
    pub text: String,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct UpdateQuestion {
    pub text: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================
// ANSWER MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, FromRow)]
#[ts(export, export_to = "../web/src/types/")]
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
#[ts(export, export_to = "../web/src/types/")]
pub struct CreateAnswer {
    pub question_id: Uuid,
    pub label: String,
    pub next_question_id: Option<Uuid>,
    pub conclusion_text: Option<String>,
    pub order_index: i32,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct UpdateAnswer {
    pub label: Option<String>,
    pub next_question_id: Option<Uuid>,
    pub conclusion_text: Option<String>,
    pub order_index: Option<i32>,
    pub is_active: Option<bool>,
}

// ============================================
// SESSION MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub steps: serde_json::Value,
    pub final_conclusion: Option<String>,
    pub tech_identifier: Option<String>,
    pub client_site: Option<String>,
    pub user_agent: Option<String>,
    pub ip_hash: Option<String>,
    pub abandoned: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSession {
    pub tech_identifier: Option<String>,
    pub client_site: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SessionStep {
    pub question_id: Uuid,
    pub answer_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteSession {
    pub final_conclusion: String,
}

// ============================================
// AUDIT LOG MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuditLog {
    pub user_id: Uuid,
    pub action: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// ============================================
// QUERY RESPONSE MODELS
// ============================================

/// Response for question with its answers
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct QuestionWithAnswers {
    pub id: Uuid,
    pub semantic_id: String,
    pub text: String,
    pub category: Option<String>,
    pub answers: Vec<Answer>,
}

/// Response for decision tree navigation
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../web/src/types/")]
pub struct NavigationResponse {
    pub question: Question,
    pub answers: Vec<Answer>,
    pub session_id: String,
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

#[derive(Debug, Deserialize, TS)]
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

#[derive(Debug, Deserialize, TS)]
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
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../web/src/types/")]
pub struct IssueGraph {
    pub category: String,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}
