-- Define litter types available to players
CREATE TABLE IF NOT EXISTS litter_types (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    points_per_unit INTEGER NOT NULL DEFAULT 0 CHECK (points_per_unit >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed initial litter types to enable gameplay
INSERT INTO litter_types (name, description, points_per_unit) VALUES
    ('Plastic Bottle', 'Standard recyclable plastic beverage bottles.', 5),
    ('Aluminum Can', 'Lightweight aluminum cans from drinks.', 7),
    ('Glass Bottle', 'Clear or colored glass bottles.', 6),
    ('Cardboard', 'Flattened cardboard or paperboard items.', 4),
    ('Plastic Bag', 'Single-use plastic bags gathered from public spaces.', 3)
ON CONFLICT (name) DO NOTHING;
