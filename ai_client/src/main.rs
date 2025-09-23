use std::fs;
use std::net::UdpSocket;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn};
use std::time::Duration;
use neural_network_lib::neural_network::NeuralNetwork;
use rand::random;
use rand::seq::IndexedRandom;
use crate::Msg::{Action, GameState, Ping, PlayerBoardId};

const SERVER_ADDR: &str = "127.0.0.1:12542";

enum Msg {
    PlayerBoardId(([u8; 8], [u8; 8])),
    GameState([f32; 12]),
    Action(u32),
    Ping,
}

fn main() {
    let mut msg = [58, 41, 58, 80, 58, 68, 11, 13, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let (tx, rx): (Sender<Msg>, Receiver<Msg>) = channel();

    let socket_clone = socket.try_clone().unwrap();
    let tx_clone = tx.clone();
    spawn(|| udp_receiver(socket_clone, tx_clone));
    let tx_clone = tx.clone();
    spawn(|| counter(tx_clone));
    let tx_clone = tx.clone();
    spawn(|| ping(tx_clone));

    let mut player_id = [0; 8];
    let mut board_id = [0; 8];
    let mut last_state = [0.0; 12];

    let network_file = "C:\\Users\\jakubbraz\\me\\programming\\rust\\rust_volleyball\\ai_train\\test_network";
    let network = fs::read_to_string(&network_file).unwrap();
    let network = NeuralNetwork::deserialize(&network);

    // AI PLAYER IS ALWAYS MUST JOIN AS PLAYER_2
    // IT IS TRAINED TO PLAY AS THE ONE ON THE LEFT

    println!("Sending game_request: {:?}", msg);
    socket.send_to(&msg, SERVER_ADDR).unwrap();

    loop {
        match rx.recv().unwrap() {
            PlayerBoardId((new_player_id, new_board_id)) => {
                // todo: unused, remove
                player_id = new_player_id;
                board_id = new_board_id;
                msg[8..16].copy_from_slice(&new_player_id);
                msg[16..24].copy_from_slice(&new_board_id);
            }
            GameState(state) => {
                last_state = state;
            }
            Action(action_id) => {
                // match action_id {
                //     0 => msg[6..8].copy_from_slice(&[17, 23]), // press left
                //     1 => msg[6..8].copy_from_slice(&[25, 29]), // release left
                //     2 => msg[6..8].copy_from_slice(&[37, 31]), // press right
                //     3 => msg[6..8].copy_from_slice(&[67, 58]), // release right
                //     4 => msg[6..8].copy_from_slice(&[97, 33]), // jump
                //     _ => unreachable!()
                // };
                // println!("Sending data: {:?}", msg);
                // socket.send_to(&msg, SERVER_ADDR).unwrap();
                let output = network.process(&last_state);
                println!("network output: {:?}", output);
                let mut max = -1000.0;
                let mut max_i = 99;
                for (i, v) in output.iter().enumerate() {
                    if *v > max {
                        max = *v;
                        max_i = i;
                    }
                }
                let action = match max_i {
                    1 => {
                        println!("press left");
                        Some([17, 23]) // press left
                    },
                    2 => {
                        println!("release left");
                        Some([25, 29]) // release left
                    },
                    3 => {
                        println!("press right");
                        Some([37, 31]) // press right
                    },
                    4 => {
                        println!("release right");
                        Some([67, 58]) // release right
                    },
                    5 => {
                        println!("jump");
                        Some([97, 33]) // jump
                    },
                    0 => {
                        None // no action
                    },
                    _ => unreachable!("unexpected i {}", max_i)
                };
                match action {
                    None => {},
                    Some(x) => {
                        msg[6..8].copy_from_slice(&x);
                        println!("Sending data: {:?}", msg);
                        socket.send_to(&msg, SERVER_ADDR).unwrap();
                    }
                };
            }
            Ping => {
                msg[6..8].copy_from_slice(&[96, 22]);
                println!("Sending ping: {:?}", msg);
                socket.send_to(&msg, SERVER_ADDR).unwrap();
            }
        }
    }
}

fn udp_receiver(socket: UdpSocket, tx: Sender<Msg>) {
    let mut buf = [0; 512];
    loop {
        let len = socket.recv(&mut buf).unwrap();
        // println!("received {:?}", &buf[..len]);
        if &buf[..4] == [12, 64, 13, 56] {
            println!("init msg received");
            let p_id: [u8; 8] = buf[4..12].try_into().unwrap();
            let b_id: [u8; 8] = buf[12..20].try_into().unwrap();
            tx.send(PlayerBoardId((p_id, b_id))).unwrap();
        }
        else {
            let mut state: [f32; 12] = [0.0; 12];
            state[0] = f32::from_le_bytes(buf[4..8].try_into().unwrap()) / 10.0;
            state[1] = f32::from_le_bytes(buf[8..12].try_into().unwrap()) / 10.0;
            state[2] = f32::from_le_bytes(buf[57..61].try_into().unwrap()) / 10.0;
            state[3] = f32::from_le_bytes(buf[61..65].try_into().unwrap()) / 10.0;
            state[4] = f32::from_le_bytes(buf[16..20].try_into().unwrap()) / 10.0;
            state[5] = f32::from_le_bytes(buf[20..24].try_into().unwrap()) / 10.0;
            state[6] = f32::from_le_bytes(buf[41..45].try_into().unwrap()) / 10.0;
            state[7] = f32::from_le_bytes(buf[45..49].try_into().unwrap()) / 10.0;
            state[8] = f32::from_le_bytes(buf[24..28].try_into().unwrap()) / 10.0;
            state[9] = f32::from_le_bytes(buf[28..32].try_into().unwrap()) / 10.0;
            state[10] = f32::from_le_bytes(buf[49..53].try_into().unwrap()) / 10.0;
            state[11] = f32::from_le_bytes(buf[53..57].try_into().unwrap()) / 10.0;
            tx.send(GameState(state)).unwrap();
        }
    }
}

fn counter(tx: Sender<Msg>) {
    let actions = vec![0, 1, 2, 3, 4];
    let mut rng = rand::rng();

    loop {
        sleep(Duration::from_millis(200));
        let action = *actions.choose(&mut rng).unwrap();
        tx.send(Action(action)).unwrap()
        // network.process()
    }
}

fn ping(tx: Sender<Msg>) {
    loop {
        sleep(Duration::from_secs(2));
        tx.send(Ping).unwrap()
    }
}
