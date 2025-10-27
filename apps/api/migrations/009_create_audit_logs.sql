-- Create audit_logs table for tracking all administrative actions
-- This table stores a comprehensive audit trail for compliance and security monitoring

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    details JSONB,
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Composite index for common query patterns (user + time range)
CREATE INDEX idx_audit_logs_user_time ON audit_logs(user_id, created_at DESC);

-- Comments for documentation
COMMENT ON TABLE audit_logs IS 'Audit trail for all administrative actions';
COMMENT ON COLUMN audit_logs.user_id IS 'User who performed the action';
COMMENT ON COLUMN audit_logs.action IS 'Type of action performed (e.g., issue_created, node_deleted)';
COMMENT ON COLUMN audit_logs.resource_type IS 'Type of resource affected (e.g., issue, node, connection)';
COMMENT ON COLUMN audit_logs.resource_id IS 'Identifier of the specific resource affected';
COMMENT ON COLUMN audit_logs.details IS 'Additional context about the action in JSON format';
COMMENT ON COLUMN audit_logs.ip_address IS 'IP address of the request';
COMMENT ON COLUMN audit_logs.created_at IS 'Timestamp when the action occurred';
