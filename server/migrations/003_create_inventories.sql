-- Track per-player holdings of each litter type
CREATE TABLE IF NOT EXISTS inventories (
    id BIGSERIAL PRIMARY KEY,
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    litter_type_id INTEGER NOT NULL REFERENCES litter_types(id),
    quantity INTEGER NOT NULL DEFAULT 0 CHECK (quantity >= 0),
    UNIQUE (player_id, litter_type_id)
);
