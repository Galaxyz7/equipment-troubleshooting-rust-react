/// Audit logging utilities for tracking admin actions
///
/// This module provides functionality to log all administrative actions
/// for security monitoring, compliance, and forensic analysis.
use sqlx::PgPool;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Audit event types for different admin actions
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AuditAction {
    // Issue management
    IssueCreated,
    IssueUpdated,
    IssueToggled,
    IssueDeleted,
    IssueExported,
    IssuesImported,

    // Node/Connection management
    NodeCreated,
    NodeUpdated,
    NodeDeleted,
    ConnectionCreated,
    ConnectionUpdated,
    ConnectionDeleted,

    // Category management
    CategoryRenamed,
    CategoryDeleted,

    // Session management
    SessionsDeleted,

    // Authentication
    AdminLogin,
    AdminLogout,
}

impl AuditAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::IssueCreated => "issue_created",
            Self::IssueUpdated => "issue_updated",
            Self::IssueToggled => "issue_toggled",
            Self::IssueDeleted => "issue_deleted",
            Self::IssueExported => "issue_exported",
            Self::IssuesImported => "issues_imported",
            Self::NodeCreated => "node_created",
            Self::NodeUpdated => "node_updated",
            Self::NodeDeleted => "node_deleted",
            Self::ConnectionCreated => "connection_created",
            Self::ConnectionUpdated => "connection_updated",
            Self::ConnectionDeleted => "connection_deleted",
            Self::CategoryRenamed => "category_renamed",
            Self::CategoryDeleted => "category_deleted",
            Self::SessionsDeleted => "sessions_deleted",
            Self::AdminLogin => "admin_login",
            Self::AdminLogout => "admin_logout",
        }
    }
}

/// Log an audit event to the database
///
/// # Arguments
/// * `db` - Database connection pool
/// * `user_id` - UUID of the user performing the action
/// * `action` - Type of action being performed
/// * `resource_type` - Type of resource being acted upon (e.g., "issue", "node")
/// * `resource_id` - ID of the specific resource (optional)
/// * `details` - Additional JSON details about the action (optional)
/// * `ip_address` - IP address of the request (optional)
///
/// # Example
/// ```
/// use uuid::Uuid;
/// use serde_json::json;
///
/// audit::log_event(
///     &db,
///     user_id,
///     AuditAction::IssueCreated,
///     "issue",
///     Some("printer-issues"),
///     Some(json!({ "name": "Printer Issues", "active": true })),
///     Some("192.168.1.100")
/// ).await?;
/// ```
pub async fn log_event(
    db: &PgPool,
    user_id: Uuid,
    action: AuditAction,
    resource_type: &str,
    resource_id: Option<&str>,
    details: Option<JsonValue>,
    ip_address: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user_id,
        action.as_str(),
        resource_type,
        resource_id,
        details,
        ip_address,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Extract IP address from HTTP headers
///
/// Attempts to get the real client IP from various proxy headers,
/// falling back to the direct connection IP.
pub fn extract_ip_address(headers: &axum::http::HeaderMap) -> Option<String> {
    // Try X-Forwarded-For first (most common proxy header)
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded_for.to_str() {
            // X-Forwarded-For can contain multiple IPs, take the first one
            return Some(value.split(',').next()?.trim().to_string());
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return Some(value.to_string());
        }
    }

    // If behind a proxy but no headers, we can't determine the real IP
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_audit_action_as_str() {
        assert_eq!(AuditAction::IssueCreated.as_str(), "issue_created");
        assert_eq!(AuditAction::NodeDeleted.as_str(), "node_deleted");
        assert_eq!(AuditAction::AdminLogin.as_str(), "admin_login");
    }

    #[test]
    fn test_extract_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.100, 10.0.0.1".parse().unwrap());

        let ip = extract_ip_address(&headers);
        assert_eq!(ip, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_extract_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "192.168.1.100".parse().unwrap());

        let ip = extract_ip_address(&headers);
        assert_eq!(ip, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn test_extract_ip_no_headers() {
        let headers = HeaderMap::new();
        let ip = extract_ip_address(&headers);
        assert_eq!(ip, None);
    }
}
