-- Create users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    password_hash TEXT,
    company_id TEXT,
    role user_role NOT NULL DEFAULT 'member',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    oauth_provider TEXT,
    oauth_subject TEXT,
    oauth_picture TEXT,
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE SET NULL,
    CONSTRAINT valid_email CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'),
    CONSTRAINT password_or_oauth CHECK (password_hash IS NOT NULL OR oauth_provider IS NOT NULL),
    CONSTRAINT unique_oauth UNIQUE (oauth_provider, oauth_subject)
);

-- Create indexes for users table
CREATE INDEX idx_users_company_id ON users(company_id);
CREATE INDEX idx_users_email ON users(email);
