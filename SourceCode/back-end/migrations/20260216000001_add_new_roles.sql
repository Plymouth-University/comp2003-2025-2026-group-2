-- Add new values to user_role enum
-- no-transaction
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'company_manager';
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'branch_manager';
ALTER TYPE user_role ADD VALUE IF NOT EXISTS 'staff';
