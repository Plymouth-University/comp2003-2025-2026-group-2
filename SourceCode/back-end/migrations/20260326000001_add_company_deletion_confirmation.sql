-- Add deletion confirmation to companies table
ALTER TABLE companies ADD COLUMN deletion_requested_at TIMESTAMP;
ALTER TABLE companies ADD COLUMN deletion_token TEXT;
