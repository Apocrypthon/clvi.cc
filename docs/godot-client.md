# Godot client calls to the clvi.cc API

These notes describe how to call the server endpoints from Godot using an `HTTPRequest` node. The examples assume the backend is reachable at `http://localhost:3001/api` and that you already have an auth token and session identifier for the current player.

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

* `Authorization: Bearer <auth_token>` — required for all endpoints.
* `X-Session-Id: <session_id>` — required to bind requests to a player session.
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

Fetches the player's aggregate action totals. Use this after login, after collect/recycle calls, or when resuming a session.

```gdscript
var headers := [
    "Authorization: Bearer %s" % auth_token,
    "X-Session-Id: %s" % session_id,
    "Accept: application/json"
]
var query := "?player_id=%s" % player_id # optional; defaults to session player
http.request("%s/player/state%s" % [base_url, query], headers, HTTPClient.METHOD_GET)
```

**Sample response (200):**

```json
{
  "player_id": "player_123",
  "collected_total": 12,
  "recycled_total": 7
}
```

**Godot parsing tips:**

```gdscript
var state := parsed
collected_label.text = str(state.collected_total)
recycled_label.text = str(state.recycled_total)
```

If you omit `player_id`, the server will resolve it using the session. Include it when you need to fetch another player's totals.

## POST /action/collect

Collects a resource and returns the action response summary.

```gdscript
var idempotency_key := UUID.v4()
var headers := [
    "Authorization: Bearer %s" % auth_token,
    "X-Session-Id: %s" % session_id,
    "Content-Type: application/json",
    "Accept: application/json",
    "Idempotency-Key: %s" % idempotency_key # reuse on retry
]
var body := {
    "resource": "battery",
    "amount": 1,
    "player_id": "player_123" # optional; defaults to session player
}
http.request("%s/action/collect" % base_url, headers, HTTPClient.METHOD_POST, JSON.stringify(body))
```

**Successful response (200/201):**

```json
{
  "player_id": "player_123",
  "action": "collect",
  "resource": "battery",
  "amount": 1,
  "status": "ok"
}
```

**Error examples:**

* `409 Conflict` with `{ "error": "collect_cooldown_active", "retry_after_ms": 1200 }`.
* `400 Bad Request` with `{ "error": "invalid_node_id" }`.

After a success, refresh UI by calling `/player/state`. On `409`, wait for `retry_after_ms` before retrying.

## POST /action/recycle

Recycles resources and returns the action response summary.

```gdscript
var recycle_key := UUID.v4()
var headers := [
    "Authorization: Bearer %s" % auth_token,
    "X-Session-Id: %s" % session_id,
    "Content-Type: application/json",
    "Accept: application/json",
    "Idempotency-Key: %s" % recycle_key # reuse if you retry
]
var body := {
    "resource": "battery",
    "amount": 2,
    "player_id": "player_123" # optional; defaults to session player
}
http.request("%s/action/recycle" % base_url, headers, HTTPClient.METHOD_POST, JSON.stringify(body))
```

**Successful response (200/201):**

```json
{
  "player_id": "player_123",
  "action": "recycle",
  "resource": "battery",
  "amount": 2,
  "status": "ok"
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
