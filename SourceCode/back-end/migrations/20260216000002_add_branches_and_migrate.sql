-- Create branches table
CREATE TABLE branches (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    address TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Add branch_id to users
ALTER TABLE users ADD COLUMN branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL;

-- Add role and branch_id to invitations
ALTER TABLE invitations ADD COLUMN role user_role NOT NULL DEFAULT 'member';
ALTER TABLE invitations ADD COLUMN branch_id TEXT REFERENCES branches(id) ON DELETE SET NULL;

-- Migrate existing data to new roles
UPDATE users SET role = 'company_manager' WHERE role = 'admin';
UPDATE users SET role = 'staff' WHERE role = 'member';

-- Add index for branch_id
CREATE INDEX idx_users_branch_id ON users(branch_id);
CREATE INDEX idx_branches_company_id ON branches(company_id);
