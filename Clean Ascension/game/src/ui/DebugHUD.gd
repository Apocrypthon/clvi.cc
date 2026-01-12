extends CanvasLayer

@onready var label: Label = $MarginContainer/Panel/Label

func _ready() -> void:
	_update_metrics()
	var timer := Timer.new()
	timer.wait_time = 0.5
	timer.one_shot = false
	timer.autostart = true
	timer.timeout.connect(_update_metrics)
	add_child(timer)

func _update_metrics() -> void:
	var fps := int(Performance.get_monitor(Performance.TIME_FPS))
	var draw_calls := int(Performance.get_monitor(Performance.RENDER_DRAW_CALLS_IN_FRAME))
	var nodes := get_tree().get_node_count()
	label.text = "FPS: %s\nDraw Calls: %s\nNodes: %s" % [fps, draw_calls, nodes]
