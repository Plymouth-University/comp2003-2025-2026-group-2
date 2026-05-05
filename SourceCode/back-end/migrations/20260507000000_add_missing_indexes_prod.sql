-- Fix missing indexes identified in production pg_stat_user_indexes

-- Password_resets: token lookups not using index
CREATE INDEX idx_password_resets_token_lookup ON password_resets(token) WHERE used_at IS NULL;

-- Users: drop unused idx_users_email (redundant with unique constraint)
DROP INDEX IF EXISTS idx_users_email;

-- Users: add proper index for email lookups
CREATE INDEX idx_users_email_lookup ON users(email);