# volleyball_client.gd
extends Node

# Server configuration
#const TCP_HOST := "127.0.0.1"
const TCP_HOST := "20.157.206.105"
const TCP_PORT := 12541
#const UDP_HOST := "127.0.0.1"
const UDP_HOST := "20.157.206.105"
const UDP_PORT := 12542

# Protocol magic bytes
var MAGIC_HEADER := PackedByteArray([58, 41, 58, 80, 58, 68])  # ":):P:D"

# TCP opcodes
var OPCODE_PLAYER_ID_REQUEST := PackedByteArray([13, 22])
var OPCODE_PING := PackedByteArray([96, 22])

# UDP opcodes
var OPCODE_GAME_REQUEST := PackedByteArray([11, 13])
var OPCODE_JUMP := PackedByteArray([97, 33])
var OPCODE_LEFT_PRESSED := PackedByteArray([17, 23])
var OPCODE_LEFT_RELEASED := PackedByteArray([25, 99])
var OPCODE_RIGHT_PRESSED := PackedByteArray([37, 31])
var OPCODE_RIGHT_RELEASED := PackedByteArray([67, 58])

# Node references - assign these in the editor or via code
@export var parent_character_1: Node3D
@export var parent_character_2: Node3D
@export var ball: Node3D

# Connection state
var tcp_stream := StreamPeerTCP.new()
var udp_peer := PacketPeerUDP.new()

var player_id: int = 0
var board_id: int = 0
var opponent_id: int = 0
var is_server_connected := false
var game_started := false

# Ping timer (server timeout is 30 sec, so ping every 20 sec)
var ping_timer := 0.0
const PING_INTERVAL := 20.0

# Game state received from server
var ball_pos := Vector2.ZERO
var player1_pos := Vector2.ZERO
var player2_pos := Vector2.ZERO
var score1 := 0
var score2 := 0
var game_over := false


func _ready() -> void:
	_connect_to_server()


func _connect_to_server() -> void:
	# Connect TCP first to get player ID
	var tcp_error := tcp_stream.connect_to_host(TCP_HOST, TCP_PORT)
	if tcp_error != OK:
		push_error("Failed to initiate TCP connection: %s" % error_string(tcp_error))
		return
	
	# Set up UDP socket (bind to any available port)
	var udp_error := udp_peer.bind(0)
	if udp_error != OK:
		push_error("Failed to bind UDP socket: %s" % error_string(udp_error))
		return
	
	udp_peer.set_dest_address(UDP_HOST, UDP_PORT)
	print("UDP socket bound to port: ", udp_peer.get_local_port())


func _process(delta: float) -> void:
	_poll_tcp()
	_poll_udp()
	
	if is_server_connected:
		_handle_ping(delta)
	
	if game_started:
		_handle_input()


func _poll_tcp() -> void:
	tcp_stream.poll()
	var status := tcp_stream.get_status()
	
	match status:
		StreamPeerTCP.STATUS_CONNECTED:
			if not is_server_connected:
				is_server_connected = true
				print("TCP connected, requesting player ID...")
				_send_player_id_request()
			
			# Read available TCP data
			var available := tcp_stream.get_available_bytes()
			if available > 0:
				var data := tcp_stream.get_data(available)
				if data[0] == OK:
					_handle_tcp_data(data[1])
		
		StreamPeerTCP.STATUS_CONNECTING:
			pass  # Still connecting
		
		StreamPeerTCP.STATUS_NONE, StreamPeerTCP.STATUS_ERROR:
			if is_server_connected:
				is_server_connected = false
				game_started = false
				push_error("TCP connection lost")


func _poll_udp() -> void:
	while udp_peer.get_available_packet_count() > 0:
		var packet := udp_peer.get_packet()
		_handle_udp_data(packet)


func _send_player_id_request() -> void:
	# Build 32-byte packet: magic (6) + opcode (2) + padding (24)
	var packet := PackedByteArray()
	packet.append_array(MAGIC_HEADER)
	packet.append_array(OPCODE_PLAYER_ID_REQUEST)
	packet.resize(32)  # Pad to 32 bytes
	
	var error := tcp_stream.put_data(packet)
	if error != OK:
		push_error("Failed to send player ID request: %s" % error_string(error))


func _handle_tcp_data(data: PackedByteArray) -> void:
	print("TCP received %d bytes: %s" % [data.size(), data])

	# First TCP response should be player ID (8 bytes)
	if player_id == 0 and data.size() >= 8:
		player_id = _bytes_to_int64(data.slice(0, 8))
		print("Received player ID: %d" % player_id)

		# Now send game request via UDP
		_send_game_request()
	# Opponent ID message (8 bytes) - sent when game starts
	elif opponent_id == 0 and data.size() >= 8:
		opponent_id = _bytes_to_int64(data.slice(0, 8))


func _handle_udp_data(data: PackedByteArray) -> void:
	if data.size() < 4:
		return
	
	# Check if this is a "set address" response (board assignment)
	# Format: [12, 64, 13, 56] + player_id (8) + board_id (8)
	if data.size() >= 20 and data[0] == 12 and data[1] == 64 and data[2] == 13 and data[3] == 56:
		player_id = _bytes_to_int64(data.slice(4, 12))
		board_id = _bytes_to_int64(data.slice(12, 20))
		game_started = true
		print("Game assigned - Player ID: %d, Board ID: %d" % [player_id, board_id])
		return
	
	# Game state update (64 bytes)
	# Format: ball_radius(4) + ball_x(4) + ball_y(4) + player_radius(4) + 
	#         p1_x(4) + p1_y(4) + p2_x(4) + p2_y(4) + score1(4) + score2(4) + game_over(1)
	if data.size() >= 41:
		ball_pos.x = _bytes_to_float(data.slice(4, 8))
		ball_pos.y = _bytes_to_float(data.slice(8, 12))
		player1_pos.x = _bytes_to_float(data.slice(16, 20))
		player1_pos.y = _bytes_to_float(data.slice(20, 24))
		player2_pos.x = _bytes_to_float(data.slice(24, 28))
		player2_pos.y = _bytes_to_float(data.slice(28, 32))
		score1 = _bytes_to_int32(data.slice(32, 36))
		score2 = _bytes_to_int32(data.slice(36, 40))
		game_over = data[40] == 1
		
		_update_node_positions()


func _update_node_positions() -> void:
	# Server sends (x, y) in 2D space
	# Map to Godot 3D: server_x -> z, server_y -> y (same as main_scene.gd line 82-84)
	# Server layout: player1 at x=6.0, player2 at x=2.0, net at x=4.0

	if parent_character_1:
		parent_character_1.position.z = player1_pos.x
		parent_character_1.position.y = player1_pos.y

	if parent_character_2:
		parent_character_2.position.z = player2_pos.x
		parent_character_2.position.y = player2_pos.y

	if ball:
		ball.position.z = ball_pos.x
		ball.position.y = ball_pos.y


func _send_game_request() -> void:
	var packet := _build_udp_packet(OPCODE_GAME_REQUEST)
	udp_peer.put_packet(packet)
	print("Game request sent")


func _handle_ping(delta: float) -> void:
	ping_timer += delta
	if ping_timer >= PING_INTERVAL:
		ping_timer = 0.0
		_send_ping()


func _send_ping() -> void:
	# Ping is sent via TCP (server tracks last_ping per TCP connection)
	var packet := PackedByteArray()
	packet.append_array(MAGIC_HEADER)
	packet.append_array(OPCODE_PING)
	packet.resize(32)  # Pad to 32 bytes
	
	var error := tcp_stream.put_data(packet)
	if error != OK:
		push_error("Failed to send ping: %s" % error_string(error))
	else:
		print("Ping sent")


func _handle_input() -> void:
	# Jump
	if Input.is_action_just_pressed("ui_up"):
		_send_udp_input(OPCODE_JUMP)

	# Left movement - swap left/right to fix inverted controls
	if Input.is_action_just_pressed("ui_left"):
		_send_udp_input(OPCODE_RIGHT_PRESSED)
	if Input.is_action_just_released("ui_left"):
		_send_udp_input(OPCODE_RIGHT_RELEASED)

	# Right movement - swap left/right to fix inverted controls
	if Input.is_action_just_pressed("ui_right"):
		_send_udp_input(OPCODE_LEFT_PRESSED)
	if Input.is_action_just_released("ui_right"):
		_send_udp_input(OPCODE_LEFT_RELEASED)


func _send_udp_input(opcode: PackedByteArray) -> void:
	var packet := _build_udp_packet(opcode)
	udp_peer.put_packet(packet)


func _build_udp_packet(opcode: PackedByteArray) -> PackedByteArray:
	# Build 32-byte packet:
	# magic (6) + opcode (2) + player_id (8) + board_id (8) + padding (8)
	var packet := PackedByteArray()
	packet.append_array(MAGIC_HEADER)
	packet.append_array(opcode)
	packet.append_array(_int64_to_bytes(player_id))
	packet.append_array(_int64_to_bytes(board_id))
	packet.resize(32)  # Ensure exactly 32 bytes
	return packet


# Utility functions for byte conversion (little-endian)
func _int64_to_bytes(value: int) -> PackedByteArray:
	var bytes := PackedByteArray()
	bytes.resize(8)
	bytes.encode_s64(0, value)
	return bytes


func _bytes_to_int64(bytes: PackedByteArray) -> int:
	return bytes.decode_s64(0)


func _bytes_to_int32(bytes: PackedByteArray) -> int:
	return bytes.decode_s32(0)


func _bytes_to_float(bytes: PackedByteArray) -> float:
	return bytes.decode_float(0)


func disconnect_from_server() -> void:
	tcp_stream.disconnect_from_host()
	udp_peer.close()
	is_server_connected = false
	game_started = false
	player_id = 0
	board_id = 0
	opponent_id = 0


func _exit_tree() -> void:
	disconnect_from_server()


#extends Node3D
#
#var socket = StreamPeerTCP.new()
#
## Called when the node enters the scene tree for the first time.
#func _ready():
	#var res = socket.connect_to_host("127.0.0.1", 12541)
	#if res == OK:
		#print("tcp socket, start connection")
	#else:
		#print("tcp connect error: ", res)
#
#
## Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(_delta):
	#while socket.get_available_bytes() > 0:
		#print("read data")
