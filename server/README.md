# Server schema

This directory contains SQL migrations for the core game data. Apply the files in numerical order to create tables and seed starter data.

## Tables

### players
- `id` BIGSERIAL primary key.
- `username` text, required and unique to identify each player.
- `display_name` optional friendly label.
- `created_at` timestamp with time zone, defaults to `NOW()`.

Integrity: usernames are unique and cannot be null.

### litter_types
- `id` SERIAL primary key.
- `name` text, required and unique.
- `description` optional text describing the item.
- `points_per_unit` integer, defaults to `0`, must be non-negative.
- `created_at` timestamp with time zone, defaults to `NOW()`.

Integrity: unique names prevent duplicate entries, and the `points_per_unit >= 0` check protects scoring logic. Migration `002_create_litter_types.sql` seeds common types like plastic bottles and aluminum cans for initial gameplay.

### inventories
- `id` BIGSERIAL primary key.
- `player_id` references `players(id)` and cascades on delete.
- `litter_type_id` references `litter_types(id)`.
- `quantity` integer, defaults to `0` with a non-negative check.
- `(player_id, litter_type_id)` is unique to ensure one row per player/type pair.

Integrity: foreign keys tie rows to valid players and litter types, the unique key prevents duplicates, and the `quantity >= 0` check stops negative inventory values.

### transactions
- `id` BIGSERIAL primary key.
- `player_id` references `players(id)` and cascades on delete.
- `litter_type_id` references `litter_types(id)`.
- `inventory_id` optional reference to the related inventory snapshot.
- `transaction_type` text constrained to `collect` or `recycle`.
- `quantity` integer with `quantity > 0` check.
- `points_earned` integer with default `0`.
- `inventory_after` optional integer snapshot with `inventory_after >= 0` when present.
- `created_at` timestamp with time zone, defaults to `NOW()`.

Integrity: foreign keys connect transactions to players and litter types, the type check limits actions to collect/recycle, positive quantities prevent zero/negative events, and the `inventory_after` constraint ensures snapshot values are present when referencing inventory rows.
