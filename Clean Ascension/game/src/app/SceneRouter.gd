extends Node

signal scene_changed(scene_path: String)

export(String) var login_scene := "res://game/scenes/ui/Screen_Login.tscn"
export(String) var player_creation_scene := "res://game/scenes/world/PlayerCreation.tscn"
export(String) var world_scene := "res://game/scenes/world/World_Main.tscn"
export(NodePath) var fade_layer_path

var _current_scene: Node

func go_to_login() -> void:
	_transition_to(login_scene)

func go_to_player_creation() -> void:
	_transition_to(player_creation_scene)

func go_to_world() -> void:
	_transition_to(world_scene)

func _transition_to(scene_path: String) -> void:
	if scene_path == "":
		return
	var fade_layer := _get_fade_layer()
	if fade_layer != null and fade_layer.has_method("fade_out"):
		var fade_state = fade_layer.fade_out()
		if fade_state is GDScriptFunctionState:
			yield(fade_state, "completed")
	var load_state = _load_scene_async(scene_path)
	var new_scene = load_state
	if load_state is GDScriptFunctionState:
		new_scene = yield(load_state, "completed")
	if new_scene == null:
		return
	if _current_scene != null and is_instance_valid(_current_scene):
		_current_scene.queue_free()
	_current_scene = new_scene
	get_tree().get_root().add_child(_current_scene)
	if fade_layer != null and fade_layer.has_method("fade_in"):
		var fade_in_state = fade_layer.fade_in()
		if fade_in_state is GDScriptFunctionState:
			yield(fade_in_state, "completed")
	scene_changed.emit(scene_path)

func _load_scene_async(scene_path: String) -> Node:
	var loader := ResourceLoader.load_interactive(scene_path)
	if loader == null:
		return null
	while true:
		var status := loader.poll()
		if status == ERR_FILE_EOF:
			break
		if status != OK:
			return null
		yield(get_tree(), "idle_frame")
	var packed := loader.get_resource()
	if packed == null:
		return null
	return packed.instance()

func _get_fade_layer() -> Node:
	if fade_layer_path == null or str(fade_layer_path) == "":
		return null
	return get_node(fade_layer_path)
