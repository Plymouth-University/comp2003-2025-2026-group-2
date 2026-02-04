-- Remove redundant indexes that are covered by unique constraints or composite indexes
DROP INDEX IF EXISTS idx_password_resets_token;
DROP INDEX IF EXISTS idx_invitations_token;
DROP INDEX IF EXISTS idx_security_logs_event_type;
