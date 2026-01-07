extends Node

# Clean-Ascension: Multiplayer & Economic Sync
# Interfaces with Nakama server for real-time state and token mining.

signal token_progress_updated(token_id: String, progress: float)
signal player_spawned(spawn_data: Dictionary)

var client: NakamaClient
var session: NakamaSession
var socket: NakamaSocket
var match_id: String = ""

const SERVER_KEY := "defaultkey"
const HOST := "127.0.0.1"
const PORT := 7350

func authenticate_and_connect(config: Dictionary) -> void:
	client = Nakama.create_client(SERVER_KEY, HOST, PORT, "http")

	# MetaMask/Google/Apple Auth Wrapper Placeholder
	var device_id := OS.get_unique_id()
	session = await client.authenticate_device_async(device_id)

	socket = Nakama.create_socket_from(client)
	await socket.connect_async(session)

	_join_remediation_match(config.get("private_code", ""))

func request_spawn() -> void:
	if match_id == "" or socket == null:
		return
	var payload := {
		"op": "PLAYER_SPAWN",
		"data": {
			"timestamp": Time.get_unix_time_from_system()
		}
	}
	socket.send_match_state_async(match_id, 2, JSON.stringify(payload))

func _join_remediation_match(code: String) -> void:
	if code != "":
		var result := await socket.join_match_async(code)
		match_id = result.match_id
	else:
		var result := await socket.create_match_async()
		match_id = result.match_id

	socket.received_match_state.connect(_on_match_state)

func sync_token_progress(token_id: String, amount: float) -> void:
	if match_id == "" or socket == null:
		return
	var payload := {
		"op": "TOKEN_MINING",
		"data": {
			"token_id": token_id,
			"contribution": amount,
			"timestamp": Time.get_unix_time_from_system()
		}
	}
	socket.send_match_state_async(match_id, 1, JSON.stringify(payload))

func _on_match_state(p_state: NakamaRTAPI.MatchData) -> void:
	var content := JSON.parse_string(p_state.data)
	if typeof(content) != TYPE_DICTIONARY:
		return
	match p_state.op_code:
		1: # TOKEN_UPDATE
			_handle_token_sync(content)
		2: # PLAYER_SPAWN
			_handle_remote_player(content)

func _handle_token_sync(data: Dictionary) -> void:
	if not data.has("data"):
		return
	var token_payload := data.data
	if not token_payload.has("token_id"):
		return
	var token_id := str(token_payload.token_id)
	var progress := float(token_payload.get("progress", token_payload.get("contribution", 0.0)))
	token_progress_updated.emit(token_id, progress)

func _handle_remote_player(data: Dictionary) -> void:
	if not data.has("data"):
		return
	player_spawned.emit(data.data)
