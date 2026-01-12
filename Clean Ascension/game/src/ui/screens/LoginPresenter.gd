extends Control

@onready var email_field: LineEdit = $Panel/VBoxContainer/Email
@onready var password_field: LineEdit = $Panel/VBoxContainer/Password
@onready var login_button: Button = $Panel/VBoxContainer/BtnLogin
@onready var status_label: Label = $Panel/VBoxContainer/Status

var _request_in_flight := false

func _ready() -> void:
	login_button.pressed.connect(_on_login_pressed)
	status_label.text = ""

func _on_login_pressed() -> void:
	if _request_in_flight:
		return
	_request_in_flight = true
	login_button.disabled = true
	status_label.text = "Signing in..."
	yield(_login_async(), "completed")

func _login_async() -> void:
	var services := _get_services()
	if services == null:
		_status_error("Services unavailable")
		return
	var request := services.http.post("/auth/login", {
		"email": email_field.text.strip_edges(),
		"password": password_field.text
	})
	var response := request
	if request is GDScriptFunctionState:
		response = yield(request, "completed")
	if response == null:
		_status_error("No response")
		return
	if response.has("error"):
		_status_error(str(response.error))
		return
	var token := response.get("token", "")
	if token == "":
		_status_error("Missing session token")
		return
	var app_state := _get_app_state()
	if app_state != null:
		app_state.session_token = token
	var auth_request := services.net.authenticate(token)
	if auth_request is GDScriptFunctionState:
		yield(auth_request, "completed")
	status_label.text = "Authenticated"
	_reset_request_state()

func _status_error(message: String) -> void:
	status_label.text = message
	_reset_request_state()

func _reset_request_state() -> void:
	_request_in_flight = false
	login_button.disabled = false

func _get_services() -> Node:
	if has_node("/root/Services"):
		return get_node("/root/Services")
	return null

func _get_app_state() -> Node:
	if has_node("/root/AppState"):
		return get_node("/root/AppState")
	return null
