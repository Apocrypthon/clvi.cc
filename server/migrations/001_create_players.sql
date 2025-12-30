CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create players table with unique identifier and metadata
CREATE TABLE IF NOT EXISTS players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    display_name TEXT,
    wallet_address TEXT,
    guardian_tokens_completed INTEGER NOT NULL DEFAULT 0,
    skill_rating DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    last_login TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
