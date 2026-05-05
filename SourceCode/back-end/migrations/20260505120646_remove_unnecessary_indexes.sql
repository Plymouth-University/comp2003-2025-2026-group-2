-- Drop unused indexes identified via pg_stat_user_indexes analysis

-- Composite indexes never used (added but never queried)
DROP INDEX IF EXISTS idx_security_logs_user_id_created_at;
DROP INDEX IF EXISTS idx_security_logs_event_type_created_at;
DROP INDEX IF EXISTS idx_passkeys_user_id_created_at;

-- Redundant single-column indexes (superseded by composite indexes)
DROP INDEX IF EXISTS idx_passkeys_user_id;
DROP INDEX IF EXISTS idx_security_logs_user_id;
DROP INDEX IF EXISTS idx_security_logs_event_type;

-- Indexes on columns never queried in WHERE clauses
DROP INDEX IF EXISTS idx_users_branch_id;
DROP INDEX IF EXISTS idx_branch_deletion_tokens_user_id;
DROP INDEX IF EXISTS idx_branch_deletion_tokens_branch_id;
DROP INDEX IF EXISTS idx_branch_deletion_tokens_token;
DROP INDEX IF EXISTS idx_security_logs_company_id_created_at;
DROP INDEX IF EXISTS idx_security_logs_target_email_created_at;
DROP INDEX IF EXISTS idx_security_logs_target_email_trgm;
DROP INDEX IF EXISTS idx_security_logs_failed_logins;
DROP INDEX IF EXISTS idx_password_resets_user_id;
DROP INDEX IF EXISTS idx_password_resets_active;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_invitations_company_id;
DROP INDEX IF EXISTS idx_invitations_email;
DROP INDEX IF EXISTS idx_invitations_active;

-- Additional unused indexes found via pg_stat_user_indexes (0 scans)
DROP INDEX IF EXISTS idx_clock_events_created_at;