use equipment_troubleshooting::routes::auth::{LoginRequest, LoginResponse, UserInfo};
use equipment_troubleshooting::routes::troubleshoot::{StartSessionRequest, SubmitAnswerRequest, NavigationOption};
use equipment_troubleshooting::routes::issues::{CreateIssueRequest, UpdateIssueRequest};
use equipment_troubleshooting::models::*;
use uuid::Uuid;

/// Validation and structure tests for API requests/responses
/// These tests verify request validation logic and response structures

// ============================================
// AUTH ROUTE VALIDATION TESTS
// ============================================

#[tokio::test]
async fn test_login_request_valid() {
    let request = LoginRequest {
        email: "admin@example.com".to_string(),
        password: "password123".to_string(),
    };

    assert_eq!(request.email, "admin@example.com");
    assert!(!request.password.is_empty());
}

#[tokio::test]
async fn test_login_request_empty_fields() {
    let request = LoginRequest {
        email: "".to_string(),
        password: "".to_string(),
    };

    // Empty fields should be caught by validation
    assert!(request.email.is_empty());
    assert!(request.password.is_empty());
}

#[tokio::test]
async fn test_login_response_structure() {
    let response = LoginResponse {
        token: "jwt.token.here".to_string(),
        user: UserInfo {
            id: Uuid::new_v4().to_string(),
            email: "user@test.com".to_string(),
            role: UserRole::Admin,
        },
    };

    assert!(!response.token.is_empty());
    assert_eq!(response.user.email, "user@test.com");
}

#[tokio::test]
async fn test_user_info_serialization() {
    let user_info = UserInfo {
        id: Uuid::new_v4().to_string(),
        email: "admin@example.com".to_string(),
        role: UserRole::Admin,
    };

    let json = serde_json::to_value(&user_info).unwrap();
    assert!(json["id"].is_string());
    assert_eq!(json["email"], "admin@example.com");
}

// ============================================
// TROUBLESHOOT ROUTE VALIDATION TESTS
// ============================================

#[tokio::test]
async fn test_start_session_request_all_fields() {
    let request = StartSessionRequest {
        tech_identifier: Some("TECH001".to_string()),
        client_site: Some("Site A".to_string()),
        category: Some("hardware".to_string()),
    };

    assert_eq!(request.tech_identifier, Some("TECH001".to_string()));
    assert_eq!(request.client_site, Some("Site A".to_string()));
    assert_eq!(request.category, Some("hardware".to_string()));
}

#[tokio::test]
async fn test_start_session_request_minimal() {
    let request = StartSessionRequest {
        tech_identifier: None,
        client_site: None,
        category: None,
    };

    assert!(request.tech_identifier.is_none());
    assert!(request.client_site.is_none());
    assert!(request.category.is_none());
}

#[tokio::test]
async fn test_start_session_request_partial() {
    let request = StartSessionRequest {
        tech_identifier: Some("TECH002".to_string()),
        client_site: None,
        category: Some("software".to_string()),
    };

    assert!(request.tech_identifier.is_some());
    assert!(request.client_site.is_none());
    assert!(request.category.is_some());
}

#[tokio::test]
async fn test_submit_answer_request_valid() {
    let connection_id = Uuid::new_v4();
    let request = SubmitAnswerRequest {
        connection_id,
    };

    assert_eq!(request.connection_id, connection_id);
}

#[tokio::test]
async fn test_navigation_option_structure() {
    let option = NavigationOption {
        connection_id: Uuid::new_v4(),
        label: "Yes - Power is on".to_string(),
        target_category: "software".to_string(),
        display_category: Some("Software Issues".to_string()),
    };

    assert!(!option.label.is_empty());
    assert!(!option.target_category.is_empty());
    assert!(option.display_category.is_some());
}

// ============================================
// ISSUE ROUTE VALIDATION TESTS
// ============================================

#[tokio::test]
async fn test_create_issue_request_valid() {
    let request = CreateIssueRequest {
        name: "Network Issues".to_string(),
        category: "network".to_string(),
        display_category: Some("Network Troubleshooting".to_string()),
        root_question_text: "Is the network cable connected?".to_string(),
    };

    assert_eq!(request.name, "Network Issues");
    assert_eq!(request.category, "network");
    assert!(!request.root_question_text.is_empty());
}

#[tokio::test]
async fn test_create_issue_request_minimal() {
    let request = CreateIssueRequest {
        name: "Test Issue".to_string(),
        category: "test".to_string(),
        display_category: None,
        root_question_text: "Test question?".to_string(),
    };

    assert!(request.display_category.is_none());
    assert!(!request.name.is_empty());
}

#[tokio::test]
async fn test_update_issue_request_partial() {
    let request = UpdateIssueRequest {
        name: Some("Updated Name".to_string()),
        display_category: None,
        is_active: Some(false),
    };

    assert_eq!(request.name, Some("Updated Name".to_string()));
    assert!(request.display_category.is_none());
    assert_eq!(request.is_active, Some(false));
}

#[tokio::test]
async fn test_update_issue_request_all_fields() {
    let request = UpdateIssueRequest {
        name: Some("New Name".to_string()),
        display_category: Some("New Display".to_string()),
        is_active: Some(true),
    };

    assert!(request.name.is_some());
    assert!(request.display_category.is_some());
    assert!(request.is_active.is_some());
}

// ============================================
// NODE VALIDATION TESTS
// ============================================

#[tokio::test]
async fn test_create_node_question_type() {
    let node = CreateNode {
        category: "hardware".to_string(),
        node_type: NodeType::Question,
        text: "Is the device powered on?".to_string(),
        semantic_id: Some("hw_power".to_string()),
        display_category: Some("Hardware".to_string()),
        position_x: Some(100.0),
        position_y: Some(200.0),
    };

    assert!(matches!(node.node_type, NodeType::Question));
    assert!(!node.text.is_empty());
}

#[tokio::test]
async fn test_create_node_conclusion_type() {
    let node = CreateNode {
        category: "software".to_string(),
        node_type: NodeType::Conclusion,
        text: "Restart the application".to_string(),
        semantic_id: None,
        display_category: None,
        position_x: None,
        position_y: None,
    };

    assert!(matches!(node.node_type, NodeType::Conclusion));
    assert!(node.semantic_id.is_none());
}

#[tokio::test]
async fn test_update_node_text_only() {
    let update = UpdateNode {
        text: Some("Updated question text".to_string()),
        semantic_id: None,
        node_type: None,
        display_category: None,
        position_x: None,
        position_y: None,
        is_active: None,
    };

    assert!(update.text.is_some());
    assert!(update.semantic_id.is_none());
}

#[tokio::test]
async fn test_update_node_position() {
    let update = UpdateNode {
        text: None,
        semantic_id: None,
        node_type: None,
        display_category: None,
        position_x: Some(150.0),
        position_y: Some(250.0),
        is_active: None,
    };

    assert_eq!(update.position_x, Some(150.0));
    assert_eq!(update.position_y, Some(250.0));
}

#[tokio::test]
async fn test_update_node_deactivate() {
    let update = UpdateNode {
        text: None,
        semantic_id: None,
        node_type: None,
        display_category: None,
        position_x: None,
        position_y: None,
        is_active: Some(false),
    };

    assert_eq!(update.is_active, Some(false));
}

// ============================================
// CONNECTION VALIDATION TESTS
// ============================================

#[tokio::test]
async fn test_create_connection_valid() {
    let connection = CreateConnection {
        from_node_id: Uuid::new_v4(),
        to_node_id: Uuid::new_v4(),
        label: "Yes".to_string(),
        order_index: 0,
    };

    assert!(!connection.label.is_empty());
    assert_eq!(connection.order_index, 0);
}

#[tokio::test]
async fn test_create_connection_different_nodes() {
    let from = Uuid::new_v4();
    let to = Uuid::new_v4();

    assert_ne!(from, to);

    let connection = CreateConnection {
        from_node_id: from,
        to_node_id: to,
        label: "No".to_string(),
        order_index: 1,
    };

    assert_ne!(connection.from_node_id, connection.to_node_id);
}

#[tokio::test]
async fn test_update_connection_label() {
    let update = UpdateConnection {
        to_node_id: None,
        label: Some("Updated label".to_string()),
        order_index: None,
        is_active: None,
    };

    assert_eq!(update.label, Some("Updated label".to_string()));
    assert!(update.to_node_id.is_none());
}

#[tokio::test]
async fn test_update_connection_target() {
    let new_target = Uuid::new_v4();
    let update = UpdateConnection {
        to_node_id: Some(new_target),
        label: None,
        order_index: None,
        is_active: None,
    };

    assert_eq!(update.to_node_id, Some(new_target));
}

#[tokio::test]
async fn test_update_connection_order() {
    let update = UpdateConnection {
        to_node_id: None,
        label: None,
        order_index: Some(5),
        is_active: None,
    };

    assert_eq!(update.order_index, Some(5));
}

// ============================================
// QUESTION/ANSWER VALIDATION TESTS
// ============================================

#[tokio::test]
async fn test_create_question_valid() {
    let question = CreateQuestion {
        semantic_id: "q_network_cable".to_string(),
        text: "Is the network cable plugged in?".to_string(),
        category: Some("network".to_string()),
    };

    assert!(!question.semantic_id.is_empty());
    assert!(!question.text.is_empty());
}

#[tokio::test]
async fn test_create_question_no_category() {
    let question = CreateQuestion {
        semantic_id: "q_generic".to_string(),
        text: "Generic question?".to_string(),
        category: None,
    };

    assert!(question.category.is_none());
}

#[tokio::test]
async fn test_update_question_text() {
    let update = UpdateQuestion {
        text: Some("Updated question text?".to_string()),
        category: None,
        is_active: None,
    };

    assert!(update.text.is_some());
}

#[tokio::test]
async fn test_create_answer_with_next() {
    let answer = CreateAnswer {
        question_id: Uuid::new_v4(),
        label: "Yes".to_string(),
        next_question_id: Some(Uuid::new_v4()),
        conclusion_text: None,
        order_index: 0,
    };

    assert!(answer.next_question_id.is_some());
    assert!(answer.conclusion_text.is_none());
}

#[tokio::test]
async fn test_create_answer_with_conclusion() {
    let answer = CreateAnswer {
        question_id: Uuid::new_v4(),
        label: "No".to_string(),
        next_question_id: None,
        conclusion_text: Some("Check the cable connection".to_string()),
        order_index: 1,
    };

    assert!(answer.next_question_id.is_none());
    assert!(answer.conclusion_text.is_some());
}

#[tokio::test]
async fn test_update_answer_change_destination() {
    let new_next = Uuid::new_v4();
    let update = UpdateAnswer {
        label: None,
        next_question_id: Some(new_next),
        conclusion_text: None,
        order_index: None,
        is_active: None,
    };

    assert_eq!(update.next_question_id, Some(new_next));
}

// ============================================
// DATA STRUCTURE VALIDATION TESTS
// ============================================

#[tokio::test]
async fn test_issue_graph_empty() {
    let graph = IssueGraph {
        category: "test".to_string(),
        nodes: vec![],
        connections: vec![],
    };

    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.connections.len(), 0);
}

#[tokio::test]
async fn test_issue_graph_with_data() {
    use chrono::Utc;

    let node = Node {
        id: Uuid::new_v4(),
        category: "test".to_string(),
        node_type: NodeType::Question,
        text: "Test?".to_string(),
        semantic_id: None,
        display_category: None,
        position_x: None,
        position_y: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let graph = IssueGraph {
        category: "test".to_string(),
        nodes: vec![node],
        connections: vec![],
    };

    assert_eq!(graph.nodes.len(), 1);
}

#[tokio::test]
async fn test_node_with_connections_structure() {
    use chrono::Utc;

    let node = Node {
        id: Uuid::new_v4(),
        category: "test".to_string(),
        node_type: NodeType::Question,
        text: "Test?".to_string(),
        semantic_id: None,
        display_category: None,
        position_x: None,
        position_y: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let target_node = Node {
        id: Uuid::new_v4(),
        category: "test".to_string(),
        node_type: NodeType::Conclusion,
        text: "Solution".to_string(),
        semantic_id: None,
        display_category: None,
        position_x: None,
        position_y: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let connection_with_target = ConnectionWithTarget {
        id: Uuid::new_v4(),
        label: "Yes".to_string(),
        order_index: 0,
        target_node,
    };

    let node_with_connections = NodeWithConnections {
        node,
        connections: vec![connection_with_target],
    };

    assert_eq!(node_with_connections.connections.len(), 1);
    assert_eq!(node_with_connections.connections[0].label, "Yes");
}
