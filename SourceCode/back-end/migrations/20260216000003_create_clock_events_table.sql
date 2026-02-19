-- Create clock_events table for clock in/out tracking
CREATE TABLE clock_events (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    clock_in TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    clock_out TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'in',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE CASCADE,
    CONSTRAINT valid_status CHECK (status IN ('in', 'out'))
);

-- Index for fast lookups by user and company
CREATE INDEX idx_clock_events_user_id ON clock_events(user_id);
CREATE INDEX idx_clock_events_company_id ON clock_events(company_id);
CREATE INDEX idx_clock_events_user_status ON clock_events(user_id, status);
CREATE INDEX idx_clock_events_created_at ON clock_events(created_at DESC);
