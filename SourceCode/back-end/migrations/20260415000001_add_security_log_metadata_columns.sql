ALTER TABLE security_logs
ADD COLUMN actor_role TEXT,
ADD COLUMN company_id TEXT,
ADD COLUMN target_user_id TEXT,
ADD COLUMN target_email TEXT,
ADD COLUMN request_path TEXT,
ADD COLUMN request_method TEXT;

CREATE INDEX idx_security_logs_company_id_created_at
ON security_logs(company_id, created_at DESC);

CREATE INDEX idx_security_logs_target_email_created_at
ON security_logs(target_email, created_at DESC);

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX idx_security_logs_target_email_trgm
ON security_logs USING GIN (target_email gin_trgm_ops);
