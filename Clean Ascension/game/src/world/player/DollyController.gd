extends Node

export(NodePath) var path_follow_path
export(float) var duration := 4.0
export(float) var delay := 0.0

var _tween: Tween

func _ready() -> void:
	var path_follow := _get_path_follow()
	if path_follow == null:
		return
	path_follow.unit_offset = 0.0
	_tween = Tween.new()
	add_child(_tween)
	_tween.interpolate_property(
		path_follow,
		"unit_offset",
		0.0,
		1.0,
		duration,
		Tween.TRANS_SINE,
		Tween.EASE_IN_OUT,
		delay
	)
	_tween.start()

func _get_path_follow() -> PathFollow:
	if path_follow_path == null or str(path_follow_path) == "":
		return null
	return get_node(path_follow_path) as PathFollow
