# Clvi.cc

## Database migrations

The backend SQL schema lives in `backend/migrations`. Run the migrations in order (e.g. `0001_init.sql`) to create the tables and constraints for players, litter types, inventories, and transactions.

### Default litter type seed data

`backend/migrations/0001_init.sql` seeds the following default litter types:

| Name            | Value per unit | Metadata                                           |
|-----------------|----------------|----------------------------------------------------|
| Plastic Bottle  | 0.05           | `{"material": "plastic", "size": "standard"}`      |
| Aluminum Can    | 0.10           | `{"material": "aluminum", "size": "330ml"}`        |
| Glass Bottle    | 0.08           | `{"material": "glass", "color": "green"}`          |
| Paper           | 0.02           | `{"material": "paper", "category": "mixed"}`       |
