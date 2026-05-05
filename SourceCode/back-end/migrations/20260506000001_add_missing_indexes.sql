-- Add indexes to fix sequential scans identified via pg_stat_user_tables

-- passkey_sessions: Optimize WHERE id = $1 AND expires_at > NOW()
CREATE INDEX IF NOT EXISTS idx_passkey_sessions_id_expires ON passkey_sessions(id, expires_at);

-- passkeys: Optimize get_passkeys_by_user (WHERE user_id = $1 ORDER BY created_at DESC)
CREATE INDEX IF NOT EXISTS idx_passkeys_user_created ON passkeys(user_id, created_at DESC);

-- security_logs: Optimize user_id lookups with ordering
CREATE INDEX IF NOT EXISTS idx_security_logs_user_created ON security_logs(user_id, created_at DESC);

-- security_logs: Optimize event_type filtering with ordering
CREATE INDEX IF NOT EXISTS idx_security_logs_event_created ON security_logs(event_type, created_at DESC);

-- security_logs: Optimize company_id filtering with ordering
CREATE INDEX IF NOT EXISTS idx_security_logs_company_created ON security_logs(company_id, created_at DESC);

-- invitations: Optimize get_pending_invitations_by_company_id
CREATE INDEX IF NOT EXISTS idx_invitations_company_pending ON invitations(company_id, expires_at)
WHERE accepted_at IS NULL AND cancelled_at IS NULL;