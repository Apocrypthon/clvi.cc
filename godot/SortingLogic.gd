extends Node

# Clean-Ascension: Skill-Based Mechanics
# Handles categorization logic, timing windows, and token progress.

signal token_completed(token_id: String)
signal sorting_error(penalty_type: String)

enum TrashType { PAPER, PLASTIC, METAL, ORGANIC, HAZARDOUS }

var accuracy: float = 100.0
var collection_rate: float = 0.0
var combo_multiplier: int = 1
var total_sorts: int = 0
var successful_sorts: int = 0
var _session_start_ms: int = 0

@export var window_time: float = 2.0 # Seconds to sort before expiration

func _ready() -> void:
	_session_start_ms = Time.get_ticks_msec()

func attempt_sort(item_data: Dictionary, bin_type: TrashType) -> void:
	if item_data.is_empty():
		return
	if not item_data.has("correct_type"):
		return
	if not item_data.has("token_id"):
		return

	total_sorts += 1

	if item_data.correct_type == bin_type:
		_process_success(item_data)
	else:
		_process_failure()

	_update_metrics()

func _process_success(item: Dictionary) -> void:
	successful_sorts += 1
	combo_multiplier += 1

	# Guardian Token Progress Logic
	var progress_gain := 0.1 * combo_multiplier
	if get_parent().has_method("sync_token_progress"):
		get_parent().sync_token_progress(item.token_id, progress_gain)
	elif get_parent().has_node("NakamaClient"):
		var nakama := get_parent().get_node("NakamaClient")
		if nakama.has_method("sync_token_progress"):
			nakama.sync_token_progress(item.token_id, progress_gain)

func _process_failure() -> void:
	combo_multiplier = 1
	sorting_error.emit("MISCLASSIFICATION")

func _update_metrics() -> void:
	if total_sorts <= 0:
		accuracy = 100.0
		collection_rate = 0.0
		return
	accuracy = (float(successful_sorts) / total_sorts) * 100.0
	var elapsed_minutes := max((Time.get_ticks_msec() - _session_start_ms) / 60000.0, 0.0001)
	collection_rate = successful_sorts / elapsed_minutes
