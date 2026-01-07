extends Node

# Clean-Ascension: Central Orchestrator
# Manages scene transitions, camera switching, and HUD initialization.

@onready var nakama_client: Node = $NakamaClient
@onready var sorting_logic: Node = $SortingLogic
@onready var camera_2d: Camera2D = $Cameras/Camera2D
@onready var camera_3d: Camera3D = $Cameras/Camera3D
@onready var ui_canvas: CanvasLayer = $UI/CanvasLayer

enum CameraMode { MODE_2D, MODE_25D, MODE_3D }
var current_mode: CameraMode = CameraMode.MODE_25D
var _player_spawned := false

func _ready() -> void:
	_setup_ui_signals()
	_initialize_game_state()

func _setup_ui_signals() -> void:
	if ui_canvas.has_signal("start_game_pressed"):
		ui_canvas.start_game_pressed.connect(_on_start_game)
	if ui_canvas.has_signal("camera_toggle_pressed"):
		ui_canvas.camera_toggle_pressed.connect(_switch_camera_mode)

func _initialize_game_state() -> void:
	_switch_camera_mode(current_mode)
	if ui_canvas.has_method("fade_title_screen"):
		ui_canvas.fade_title_screen(true)

func _on_start_game(server_config: Dictionary) -> void:
	await nakama_client.authenticate_and_connect(server_config)
	if ui_canvas.has_method("fade_title_screen"):
		ui_canvas.fade_title_screen(false)
	_spawn_player()

func _spawn_player() -> void:
	if _player_spawned:
		return
	_player_spawned = true
	if ui_canvas.has_method("show_hud"):
		ui_canvas.show_hud(true)
	if nakama_client.has_method("request_spawn"):
		nakama_client.request_spawn()

func _switch_camera_mode(mode_index: int) -> void:
	current_mode = mode_index as CameraMode
	match current_mode:
		CameraMode.MODE_2D:
			camera_2d.make_current()
			camera_2d.position_smoothing_enabled = false
			camera_2d.enabled = true
			camera_3d.current = false
		CameraMode.MODE_25D:
			camera_2d.make_current()
			camera_2d.position_smoothing_enabled = true
			camera_2d.enabled = true
			camera_3d.current = false
		CameraMode.MODE_3D:
			camera_3d.make_current()
			camera_2d.enabled = false

func _process(_delta: float) -> void:
	if not ui_canvas.has_method("update_metrics"):
		return
	ui_canvas.update_metrics(
		sorting_logic.accuracy,
		sorting_logic.collection_rate,
		sorting_logic.combo_multiplier
	)
