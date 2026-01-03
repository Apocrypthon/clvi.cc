CREATE EXTENSION IF NOT EXISTS "pgcrypto";

ALTER TABLE players
ADD COLUMN email TEXT UNIQUE,
ADD COLUMN is_email_verified BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN email_verification_token TEXT,
ADD COLUMN email_verification_sent_at TIMESTAMPTZ,
ADD COLUMN password_hash TEXT;
