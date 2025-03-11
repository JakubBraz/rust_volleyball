extends Node3D

var players_fixed_axis
var ball_fixed_axis
var socket: PacketPeerUDP
var msg = [58, 41, 58, 80, 58, 68, 11, 13, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
var player_id = PackedByteArray()
var board_id = PackedByteArray()
var ping_time = 0

# send ping every 2 seconds
const PING_FREQ = 2.0


func _ready() -> void:
	socket = PacketPeerUDP.new()
	socket.bind(12001)
	socket.set_dest_address("127.0.0.1", 12542)
	socket.put_packet(msg)
	print("packet sent")
	
	players_fixed_axis = $player1.position[0]
	ball_fixed_axis = $ball.position[0]


func _process(delta: float) -> void:
	#print("delta ", delta, ' ', 1/delta)
	ping_time += delta
	#print("ping time", ping_time)
	if ping_time >= PING_FREQ:
		ping_time = 0
		msg[6] = 96
		msg[7] = 22
		socket.put_packet(msg)
		#print("ping sent")
		
	#print('message count: ', socket.get_available_packet_count())
	while socket.get_available_packet_count() > 0:
		#print(socket.get_available_packet_count())
		var packet = socket.get_packet()
		if player_id.is_empty() && board_id.is_empty():
			print("first packet", packet)
			player_id = packet.slice(4, 12)
			board_id = packet.slice(12, 20)
			print(player_id)
			print(board_id)
			for i in range(0, 8):
				msg[i + 8] = player_id[i]
				msg[i + 16] = board_id[i]
		else:
			#print(packet)
			var ball_r = packet.decode_float(0)
			var ball_x = packet.decode_float(4)
			var ball_y = packet.decode_float(8)
			var player_r = packet.decode_float(12)
			var p1_x = packet.decode_float(16)
			var p1_y = packet.decode_float(20)
			var p2_x = packet.decode_float(24)
			var p2_y = packet.decode_float(28)
			#print("%f %f %f" % [ball_r, ball_x, ball_y])
			$ball.position = Vector3(ball_fixed_axis, ball_y, ball_x)
			$player1.position = Vector3(players_fixed_axis, p1_y, p1_x)
			$player2.position = Vector3(players_fixed_axis, p2_y, p2_x)
	
	#[11, 13] => Ok(MsgIn::GameRequest),
	#[17, 23] => Ok(MsgIn::Input(player_id, board_id, Key::Left(true))),
	#[25, 99] => Ok(MsgIn::Input(player_id, board_id, Key::Left(false))),
	#[37, 31] => Ok(MsgIn::Input(player_id, board_id, Key::Right(true))),
	#[67, 58] => Ok(MsgIn::Input(player_id, board_id, Key::Right(false))),
	#[97, 33] => Ok(MsgIn::Input(player_id, board_id, Key::Jump)),
	#[96, 22] => Ok(MsgIn::Ping(player_id, board_id)),
	
	if Input.is_action_just_pressed("ui_left"):
		msg[6] = 17
		msg[7] = 23
		socket.put_packet(msg)
	elif Input.is_action_just_released("ui_left"):
		msg[6] = 25
		msg[7] = 99
		socket.put_packet(msg)
	elif Input.is_action_just_pressed("ui_right"):
		msg[6] = 37
		msg[7] = 31
		socket.put_packet(msg)
	elif Input.is_action_just_released("ui_right"):
		msg[6] = 67
		msg[7] = 58
		socket.put_packet(msg)
	elif Input.is_action_just_pressed("ui_up"):
		msg[6] = 97
		msg[7] = 33
		socket.put_packet(msg)
		
	if Input.is_action_just_pressed("my_key_1"):
		print("set camera 1")
		$camera1.current = true
	elif Input.is_action_just_pressed("my_key_2"):
		print("set camera 2")
		$camera2.current = true
	elif Input.is_action_just_pressed("my_key_3"):
		print("set camera 3")
		$camera3.current = true
	elif Input.is_action_just_pressed("my_key_4"):
		print("set camera 4")
		$camera4.current = true


func _input(_event: InputEvent) -> void:
	pass
	#print("ket event: ", event)
