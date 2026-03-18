CREATE TABLE IF NOT EXISTS processed_match_results (
    match_id UUID PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);