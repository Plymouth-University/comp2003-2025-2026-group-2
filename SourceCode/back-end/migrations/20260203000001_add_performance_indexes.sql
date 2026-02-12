-- Add composite indexes for performance optimization on sorted queries

-- Optimize get_security_logs_by_user (WHERE user_id = $1 ORDER BY created_at DESC)
CREATE INDEX idx_security_logs_user_id_created_at ON security_logs(user_id, created_at DESC);

-- Optimize get_recent_security_logs with event filter (WHERE event_type = $1 ORDER BY created_at DESC)
CREATE INDEX idx_security_logs_event_type_created_at ON security_logs(event_type, created_at DESC);

-- Optimize get_passkeys_by_user (WHERE user_id = $1 ORDER BY created_at DESC)
CREATE INDEX idx_passkeys_user_id_created_at ON passkeys(user_id, created_at DESC);
