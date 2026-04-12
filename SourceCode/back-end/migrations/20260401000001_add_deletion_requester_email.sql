-- Store the email of the user who requested company deletion
-- so the confirmation notification can be sent to the correct person
ALTER TABLE companies ADD COLUMN deletion_requested_by_email TEXT;
