extends StaticBody3D

@export var player_wrapper: Node3D
var node1: RigidBody3D
var node2: RigidBody3D
var last_position: Vector3

# Called when the node enters the scene tree for the first time.
func _ready():
	node1 = $"../joint_node_1_1"
	node2 = $"../joint_node_2_1"
	last_position = player_wrapper.position

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	var new_position = player_wrapper.position

	# Check if position changed - wake up rigid bodies if it did
	if new_position != last_position:
		if node1 and node1.sleeping:
			node1.sleeping = false
		if node2 and node2.sleeping:
			node2.sleeping = false
		last_position = new_position

	position = new_position
	player_wrapper.get_node("arm1_1").global_position = node1.global_position
	player_wrapper.get_node("arm1_1").global_rotation = node1.global_rotation
	player_wrapper.get_node("arm2_1").global_position = node2.global_position
	player_wrapper.get_node("arm2_1").global_rotation = node2.global_rotation
