-- Create branch_deletion_tokens table
CREATE TABLE branch_deletion_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    branch_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE CASCADE,
    CONSTRAINT valid_deletion_expiry CHECK (expires_at > created_at)
);

-- Create indexes for branch_deletion_tokens table
CREATE INDEX idx_branch_deletion_tokens_user_id ON branch_deletion_tokens(user_id);
CREATE INDEX idx_branch_deletion_tokens_branch_id ON branch_deletion_tokens(branch_id);
CREATE INDEX idx_branch_deletion_tokens_token ON branch_deletion_tokens(token);
CREATE INDEX idx_branch_deletion_tokens_active 
ON branch_deletion_tokens(token, expires_at) 
WHERE used_at IS NULL;
