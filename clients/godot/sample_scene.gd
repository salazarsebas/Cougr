extends Node2D

@onready var client = $Client
@onready var status_label = $VBox/Status
@onready var grid = $VBox/Grid

func _ready():
    status_label.text = "Fetching state..."
    for i in range(9):
        var btn = grid.get_child(i)
        btn.pressed.connect(_on_button_pressed.bind(i))
    
    var state_result = await client.read_state()
    if state_result["success"]:
        status_label.text = "State read (base64 length: %d)" % state_result["result_scval"].size()
    else:
        status_label.text = "Error: " + state_result["error"]

func _on_button_pressed(position: int):
    status_label.text = "Submitting move %d..." % position
    var btn = grid.get_child(position)
    btn.disabled = true
    var result = await client.make_move(position)
    if result["success"]:
        status_label.text = "Move sent! Hash: " + result["hash"]
        btn.text = "X"
    else:
        status_label.text = "Failed: " + result["error"]
        btn.disabled = false
