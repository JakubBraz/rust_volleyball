extends Node3D

@export var godot_body: Node3D

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	position = godot_body.position
	position.y -= 0.7
	#global_transform = godot_body.global_transform
