-- Add data export tracking and soft delete to companies table
ALTER TABLE companies ADD COLUMN data_exported_at TIMESTAMPTZ;
ALTER TABLE companies ADD COLUMN deleted_at TIMESTAMPTZ;
