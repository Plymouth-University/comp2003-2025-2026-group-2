-- Create passkey_sessions table
CREATE TABLE passkey_sessions (
    id TEXT PRIMARY KEY,
    session_type TEXT NOT NULL,
    user_id TEXT,
    challenge TEXT NOT NULL,
    meta TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL
);
