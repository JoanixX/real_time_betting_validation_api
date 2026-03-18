CREATE TABLE IF NOT EXISTS bets (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    match_id UUID NOT NULL,
    selection TEXT NOT NULL,
    amount BIGINT NOT NULL,
    odds BIGINT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bets_match_status ON bets (match_id, status);