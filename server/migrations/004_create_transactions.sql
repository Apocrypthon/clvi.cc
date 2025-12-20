-- Record collection and recycling events
CREATE TABLE IF NOT EXISTS transactions (
    id BIGSERIAL PRIMARY KEY,
    player_id BIGINT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    litter_type_id INTEGER NOT NULL REFERENCES litter_types(id),
    inventory_id BIGINT REFERENCES inventories(id) ON DELETE SET NULL,
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('collect', 'recycle')),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    points_earned INTEGER NOT NULL DEFAULT 0,
    inventory_after INTEGER CHECK (inventory_after >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enforce that inventory snapshots cannot exist without quantity data
ALTER TABLE transactions
    ADD CONSTRAINT transactions_inventory_after_requires_inventory
    CHECK ((inventory_after IS NULL AND inventory_id IS NULL) OR (inventory_after IS NOT NULL));
