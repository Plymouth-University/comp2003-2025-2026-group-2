ALTER TABLE passkey_sessions
ADD CONSTRAINT fk_passkey_sessions_user_id
FOREIGN KEY (user_id) REFERENCES users(id)
ON DELETE SET NULL;