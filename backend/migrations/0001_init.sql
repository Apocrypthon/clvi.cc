BEGIN;

CREATE TABLE players (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE litter_types (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    value_per_unit NUMERIC(12, 2) NOT NULL DEFAULT 0 CHECK (value_per_unit >= 0),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX litter_types_name_idx ON litter_types (name);
CREATE INDEX litter_types_metadata_gin_idx ON litter_types USING GIN (metadata);

CREATE TABLE inventories (
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    litter_type_id BIGINT NOT NULL REFERENCES litter_types(id) ON DELETE CASCADE,
    quantity BIGINT NOT NULL DEFAULT 0 CHECK (quantity >= 0),
    PRIMARY KEY (player_id, litter_type_id)
);

CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK (type IN ('collect', 'recycle')),
    delta_litter BIGINT NOT NULL,
    delta_currency NUMERIC(12, 2) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX transactions_player_id_idx ON transactions (player_id);
CREATE INDEX transactions_created_at_idx ON transactions (created_at DESC);

INSERT INTO litter_types (name, value_per_unit, metadata) VALUES
    ('Plastic Bottle', 0.05, '{"material": "plastic", "size": "standard"}'),
    ('Aluminum Can', 0.10, '{"material": "aluminum", "size": "330ml"}'),
    ('Glass Bottle', 0.08, '{"material": "glass", "color": "green"}'),
    ('Paper', 0.02, '{"material": "paper", "category": "mixed"}');

COMMIT;
