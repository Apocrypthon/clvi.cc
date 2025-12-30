# Godot client calls to the clvi.cc API

These notes describe how to call the server endpoints from Godot using an `HTTPRequest` node. The examples assume the backend is reachable at `http://localhost:3001/api` and that you already have a player UUID for the current player.

```gdscript
const base_url := "http://localhost:3001/api"
```

## HTTPRequest setup (Godot 4.x)

```gdscript
@onready var http := HTTPRequest.new()

func _ready() -> void:
    add_child(http)
    http.request_completed.connect(_on_request_completed)

func _on_request_completed(result: int, response_code: int, headers: PackedStringArray, body: PackedByteArray) -> void:
    if response_code == 0:
        push_error("Network error: %s" % result)
        return
    var parsed := JSON.parse_string(body.get_string_from_utf8())
    if typeof(parsed) == TYPE_DICTIONARY:
        # Handle endpoint-specific bodies below
        print(parsed)
    else:
        push_error("Unexpected payload")
```

### Required headers

* `x-player-id: <uuid>` — required for all endpoints to identify the player.
* `Content-Type: application/json` and `Accept: application/json`.
* For idempotent POST retries, include `Idempotency-Key: <uuid>` (see retries section).

### Common status codes

* `200 OK` — successful GET/POST with a JSON body.
* `201 Created` — resource was created (some collect/recycle flows may return this instead of 200).
* `400 Bad Request` — validation error (missing fields, invalid quantities).
* `401 Unauthorized` — missing/invalid auth token or session.
* `404 Not Found` — unknown resource id.
* `409 Conflict` — action blocked (cooldown or duplicate idempotency key).
* `429 Too Many Requests` — rate limit exceeded.
* `500 Internal Server Error` — unexpected failure; safe to retry with the same idempotency key.

## GET /player/state

Fetches the player profile, current inventory, and cooldowns. Use this after login, after collect/recycle calls, or when resuming a session.

```gdscript
var headers := [
    "x-player-id: %s" % player_id,
    "Accept: application/json"
]
http.request("%s/player/state" % base_url, headers, HTTPClient.METHOD_GET)
```

**Sample response (200):**

```json
{
  "player": {
    "id": "player_123",
    "display_name": "Jordan",
    "level": 4,
    "xp": 1230,
    "energy": 82
  },
  "inventory": [
    { "id": "battery", "label": "Battery", "quantity": 3, "rarity": "common" },
    { "id": "plastic", "label": "Plastic", "quantity": 5, "rarity": "common" }
  ],
  "cooldowns": {
    "collect_until": "2024-07-01T12:04:30Z",
    "recycle_until": null
  }
}
```

**Godot parsing tips:**

```gdscript
var state := parsed
name_label.text = state.player.display_name
xp_bar.value = float(state.player.xp)
energy_bar.value = float(state.player.energy)
inventory_list.clear()
for item in state.inventory:
    inventory_list.add_item("%s x%s" % [item.label, item.quantity])
```

Use `cooldowns.collect_until`/`recycle_until` to gray out UI buttons until the timestamp is in the past.

## POST /collect

Collects an item from the world and returns the updated state.

```gdscript
var idempotency_key := UUID.v4()
var headers := [
    "x-player-id: %s" % player_id,
    "Content-Type: application/json",
    "Accept: application/json",
    "Idempotency-Key: %s" % idempotency_key # reuse on retry
]
var body := {
    "node_id": "trash_can_7",
    "item_id": "battery",
    "quantity": 1
}
http.request("%s/collect" % base_url, headers, HTTPClient.METHOD_POST, JSON.stringify(body))
```

**Successful response (200/201):**

```json
{
  "status": "collected",
  "granted": { "id": "battery", "quantity": 1 },
  "state": {
    "inventory": [
      { "id": "battery", "label": "Battery", "quantity": 4 }
    ],
    "cooldowns": { "collect_until": "2024-07-01T12:04:30Z" }
  }
}
```

**Error examples:**

* `409 Conflict` with `{ "error": "collect_cooldown_active", "retry_after_ms": 1200 }`.
* `400 Bad Request` with `{ "error": "invalid_node_id" }`.

After a success, refresh UI with the nested `state` block (same shape as `/player/state`). On `409`, wait for `retry_after_ms` or the cooldown timestamp before retrying.

## POST /recycle

Recycles items and returns rewards plus the updated state.

```gdscript
var recycle_key := UUID.v4()
var headers := [
    "x-player-id: %s" % player_id,
    "Content-Type: application/json",
    "Accept: application/json",
    "Idempotency-Key: %s" % recycle_key # reuse if you retry
]
var body := {
    "item_id": "battery",
    "quantity": 2,
    "location_id": "station_a"
}
http.request("%s/recycle" % base_url, headers, HTTPClient.METHOD_POST, JSON.stringify(body))
```

**Successful response (200/201):**

```json
{
  "status": "recycled",
  "rewards": { "xp": 40, "tokens": 5 },
  "state": {
    "inventory": [
      { "id": "battery", "label": "Battery", "quantity": 2 },
      { "id": "token", "label": "Token", "quantity": 5 }
    ],
    "cooldowns": { "recycle_until": "2024-07-01T12:06:00Z" }
  }
}
```

**Error examples:**

* `400 Bad Request` with `{ "error": "insufficient_quantity" }`.
* `409 Conflict` with `{ "error": "recycle_cooldown_active", "retry_after_ms": 2000 }`.

## Rate limiting, retries, and idempotency

* The server may return `429 Too Many Requests` if a client sends bursts. Back off for at least 1–2 seconds before retrying.
* `collect` and `recycle` are **idempotent** when you reuse the same `Idempotency-Key` header. If the first attempt times out or you get a 500, retry with the same key so the server can safely return the original result without double-processing.
* Prefer resuming by calling `/player/state` after a retryable error to reconcile inventory and cooldowns.

## Updating UI after responses

1. If the response contains a top-level `state`, prefer it over stale local data.
2. Update player stats (`player.level`, `xp`, `energy`) and progress bars.
3. Rebuild inventory UI from `state.inventory` so quantities stay authoritative.
4. Disable collect/recycle buttons until their corresponding cooldown timestamps are in the past.
5. Surface `error` strings from 4xx responses to the user; for `retry_after_ms`, start a timer and retry with the same `Idempotency-Key`.
