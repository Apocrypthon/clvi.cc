CREATE TABLE IF NOT EXISTS guardian_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    current_progress DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    is_completed BOOLEAN NOT NULL DEFAULT false,
    last_contributor_id UUID REFERENCES players(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS trash_items (
    id BIGSERIAL PRIMARY KEY,
    category TEXT NOT NULL CHECK (category IN ('paper', 'plastic', 'metal', 'organic', 'hazardous')),
    base_value DOUBLE PRECISION NOT NULL,
    required_accuracy DOUBLE PRECISION NOT NULL
);
