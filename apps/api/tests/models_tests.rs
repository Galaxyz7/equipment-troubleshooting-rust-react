use equipment_troubleshooting::models::*;
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_user_role_serialization() {
    let admin = UserRole::Admin;
    let viewer = UserRole::Viewer;
    let tech = UserRole::Tech;

    let admin_json = serde_json::to_string(&admin).unwrap();
    let viewer_json = serde_json::to_string(&viewer).unwrap();
    let tech_json = serde_json::to_string(&tech).unwrap();

    assert_eq!(admin_json, "\"Admin\"");
    assert_eq!(viewer_json, "\"Viewer\"");
    assert_eq!(tech_json, "\"Tech\"");
}

#[tokio::test]
async fn test_user_role_deserialization() {
    let admin: UserRole = serde_json::from_str("\"Admin\"").unwrap();
    let viewer: UserRole = serde_json::from_str("\"Viewer\"").unwrap();
    let tech: UserRole = serde_json::from_str("\"Tech\"").unwrap();

    assert!(matches!(admin, UserRole::Admin));
    assert!(matches!(viewer, UserRole::Viewer));
    assert!(matches!(tech, UserRole::Tech));
}

#[tokio::test]
async fn test_node_type_serialization() {
    let question = NodeType::Question;
    let conclusion = NodeType::Conclusion;

    let question_json = serde_json::to_string(&question).unwrap();
    let conclusion_json = serde_json::to_string(&conclusion).unwrap();

    assert_eq!(question_json, "\"Question\"");
    assert_eq!(conclusion_json, "\"Conclusion\"");
}

#[tokio::test]
async fn test_node_type_deserialization() {
    let question: NodeType = serde_json::from_str("\"Question\"").unwrap();
    let conclusion: NodeType = serde_json::from_str("\"Conclusion\"").unwrap();

    assert!(matches!(question, NodeType::Question));
    assert!(matches!(conclusion, NodeType::Conclusion));
}

#[tokio::test]
async fn test_create_node_serialization() {
    let create_node = CreateNode {
        category: "test_category".to_string(),
        node_type: NodeType::Question,
        text: "Test question".to_string(),
        semantic_id: Some("test_id".to_string()),
        display_category: Some("Display Category".to_string()),
        position_x: Some(100.0),
        position_y: Some(200.0),
    };

    let json = serde_json::to_value(&create_node).unwrap();

    assert_eq!(json["category"], "test_category");
    assert_eq!(json["text"], "Test question");
    assert_eq!(json["position_x"], 100.0);
    assert_eq!(json["position_y"], 200.0);
}

#[tokio::test]
async fn test_update_node_partial() {
    let update = UpdateNode {
        text: Some("Updated text".to_string()),
        semantic_id: None,
        node_type: None,
        display_category: None,
        position_x: Some(150.0),
        position_y: None,
        is_active: Some(false),
    };

    let json = serde_json::to_value(&update).unwrap();

    assert_eq!(json["text"], "Updated text");
    assert_eq!(json["position_x"], 150.0);
    assert_eq!(json["is_active"], false);
    assert!(json["semantic_id"].is_null());
}

#[tokio::test]
async fn test_create_connection_validation() {
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();

    let connection = CreateConnection {
        from_node_id: from_id,
        to_node_id: to_id,
        label: "Yes".to_string(),
        order_index: 0,
    };

    assert_eq!(connection.from_node_id, from_id);
    assert_eq!(connection.to_node_id, to_id);
    assert_eq!(connection.label, "Yes");
    assert_eq!(connection.order_index, 0);
}

#[tokio::test]
async fn test_update_connection_partial() {
    let new_target = Uuid::new_v4();

    let update = UpdateConnection {
        to_node_id: Some(new_target),
        label: Some("No".to_string()),
        order_index: Some(1),
        is_active: None,
    };

    assert_eq!(update.to_node_id, Some(new_target));
    assert_eq!(update.label, Some("No".to_string()));
    assert_eq!(update.order_index, Some(1));
    assert!(update.is_active.is_none());
}

#[tokio::test]
async fn test_create_question_validation() {
    let question = CreateQuestion {
        semantic_id: "q1".to_string(),
        text: "What is the issue?".to_string(),
        category: Some("hardware".to_string()),
    };

    assert_eq!(question.semantic_id, "q1");
    assert_eq!(question.text, "What is the issue?");
    assert_eq!(question.category, Some("hardware".to_string()));
}

#[tokio::test]
async fn test_update_question_partial() {
    let update = UpdateQuestion {
        text: Some("Updated question text".to_string()),
        category: None,
        is_active: Some(true),
    };

    assert_eq!(update.text, Some("Updated question text".to_string()));
    assert!(update.category.is_none());
    assert_eq!(update.is_active, Some(true));
}

#[tokio::test]
async fn test_create_answer_with_next_question() {
    let question_id = Uuid::new_v4();
    let next_id = Uuid::new_v4();

    let answer = CreateAnswer {
        question_id,
        label: "Yes".to_string(),
        next_question_id: Some(next_id),
        conclusion_text: None,
        order_index: 0,
    };

    assert_eq!(answer.question_id, question_id);
    assert_eq!(answer.next_question_id, Some(next_id));
    assert!(answer.conclusion_text.is_none());
}

#[tokio::test]
async fn test_create_answer_with_conclusion() {
    let question_id = Uuid::new_v4();

    let answer = CreateAnswer {
        question_id,
        label: "Replace component".to_string(),
        next_question_id: None,
        conclusion_text: Some("Replace the motherboard".to_string()),
        order_index: 1,
    };

    assert!(answer.next_question_id.is_none());
    assert_eq!(
        answer.conclusion_text,
        Some("Replace the motherboard".to_string())
    );
}

#[tokio::test]
async fn test_update_answer_partial() {
    let new_next_id = Uuid::new_v4();

    let update = UpdateAnswer {
        label: Some("Updated label".to_string()),
        next_question_id: Some(new_next_id),
        conclusion_text: None,
        order_index: Some(2),
        is_active: Some(false),
    };

    assert_eq!(update.label, Some("Updated label".to_string()));
    assert_eq!(update.next_question_id, Some(new_next_id));
    assert_eq!(update.order_index, Some(2));
}

#[tokio::test]
async fn test_node_clone() {
    let node = Node {
        id: Uuid::new_v4(),
        category: "test".to_string(),
        node_type: NodeType::Question,
        text: "Test".to_string(),
        semantic_id: Some("test_id".to_string()),
        display_category: Some("Display".to_string()),
        position_x: Some(0.0),
        position_y: Some(0.0),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let cloned = node.clone();
    assert_eq!(node.id, cloned.id);
    assert_eq!(node.category, cloned.category);
    assert_eq!(node.text, cloned.text);
}

#[tokio::test]
async fn test_connection_clone() {
    let connection = Connection {
        id: Uuid::new_v4(),
        from_node_id: Uuid::new_v4(),
        to_node_id: Uuid::new_v4(),
        label: "Yes".to_string(),
        order_index: 0,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let cloned = connection.clone();
    assert_eq!(connection.id, cloned.id);
    assert_eq!(connection.from_node_id, cloned.from_node_id);
    assert_eq!(connection.to_node_id, cloned.to_node_id);
}

#[tokio::test]
async fn test_question_with_answers_structure() {
    let question = QuestionWithAnswers {
        id: Uuid::new_v4(),
        semantic_id: "q1".to_string(),
        text: "Test question".to_string(),
        category: Some("test".to_string()),
        answers: vec![],
    };

    assert_eq!(question.semantic_id, "q1");
    assert_eq!(question.answers.len(), 0);
}

#[tokio::test]
async fn test_issue_graph_structure() {
    let graph = IssueGraph {
        category: "hardware".to_string(),
        nodes: vec![],
        connections: vec![],
    };

    assert_eq!(graph.category, "hardware");
    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.connections.len(), 0);
}

#[tokio::test]
async fn test_node_with_connections_structure() {
    let node = Node {
        id: Uuid::new_v4(),
        category: "test".to_string(),
        node_type: NodeType::Question,
        text: "Test".to_string(),
        semantic_id: None,
        display_category: None,
        position_x: None,
        position_y: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let node_with_connections = NodeWithConnections {
        node: node.clone(),
        connections: vec![],
    };

    assert_eq!(node_with_connections.node.id, node.id);
    assert_eq!(node_with_connections.connections.len(), 0);
}
