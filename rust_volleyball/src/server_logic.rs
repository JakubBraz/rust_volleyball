use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use log::error;
use rand::Rng;
use crate::GameState;
use crate::udp_server::Key::Jump;
use crate::udp_server::{Key, MsgIn, SenderMsg};

pub enum LogicMessage {
    CalculateBoard,
    PlayerMsg(SocketAddr, MsgIn)
}

#[derive(Copy, Clone)]
pub struct GameStateSerialized {
    pub ball_pos: (f32, f32),
    pub ball_radius: f32,
    pub player_radius: f32,
    pub player1_pos: (f32, f32),
    pub player2_pos: (f32, f32),
}

pub fn start(logic_sender: Sender<LogicMessage>, logic_receiver: Receiver<LogicMessage>, udp_sender: Sender<SenderMsg>) {
    let mut player_in_lobby: Option<(u64, u64)> = None;
    let mut boards: HashMap<u64, (u64, u64, GameState)> = HashMap::new();
    let mut update_queue: VecDeque<(Instant, u64)> = VecDeque::new();
    let mut rng = rand::rng();

    loop {
        match logic_receiver.recv() {
            Ok(m) => match m {
                LogicMessage::CalculateBoard => {
                    match update_queue.pop_front() {
                        None => {}
                        Some((last_update, board_id)) => match boards.get_mut(&board_id) {
                            None => log::error!("Board {board_id} not found"),
                            Some((player1, player2, board)) => {
                                // is it ok to calculate Instant twice here?
                                let board_updated = board.step(last_update.elapsed().as_secs_f32());
                                update_queue.push_back((Instant::now(), board_id));

                                if board_updated {
                                    let (bx, by, br) = board.ball();
                                    let (p1x, p1y, p1r, p2x, p2y, _p2r) = board.players();
                                    let serialized = GameStateSerialized {
                                        ball_pos: (bx, by),
                                        ball_radius: br,
                                        player_radius: p1r,
                                        player1_pos: (p1x, p1y),
                                        player2_pos: (p2x, p2y),
                                    };
                                    match udp_sender.send(SenderMsg::GameLogicState(*player1, serialized)) {
                                        Ok(_) => {}
                                        Err(e) => log::error!("Cannot send UPD game state, {e}")
                                    }
                                    match udp_sender.send(SenderMsg::GameLogicState(*player2, serialized)) {
                                        Ok(_) => {}
                                        Err(e) => log::error!("Cannot send UPD game state, {e}")
                                    }
                                }

                                // todo clean after game finishes
                            }
                        }
                    };
                    match logic_sender.send(LogicMessage::CalculateBoard) {
                        Ok(()) => {}
                        Err(e) => log::error!("Cannot send CalculateBoard, {e}")
                    }
                }
                LogicMessage::PlayerMsg(addr, msg) => match msg {
                    MsgIn::GameRequest => {
                        let (new_player_id, board_id) = match player_in_lobby {
                            None => {
                                let game_id: u64 = rng.random();
                                let player_id: u64 = rng.random();
                                player_in_lobby = Some((player_id, game_id));
                                (player_id, game_id)
                            }
                            Some((waiting_player_id, board_id)) => {
                                let player_id: u64 = rng.random();
                                let game = GameState::new();
                                boards.insert(board_id, (waiting_player_id, player_id, game));
                                update_queue.push_back((Instant::now(), board_id));
                                match logic_sender.send(LogicMessage::CalculateBoard) {
                                    Ok(()) => {}
                                    Err(e) => log::error!("Cannot send CalculateBoard, {e}")
                                }
                                (player_id, board_id)
                            }
                        };
                        match udp_sender.send(SenderMsg::SetAddress(new_player_id, board_id, addr)) {
                            Ok(_) => {}
                            Err(e) => log::error!("Cannot send SetAddress, {e}")
                        }
                    }
                    MsgIn::Input(player_id, board_id, key) => match boards.get_mut(&board_id) {
                        None => log::error!("Board id {board_id} not found"),
                        Some((player1, player2, board)) => {
                            if player_id != *player1 && player_id != *player2 {
                                log::error!("Player id {player_id} not found, {} {}", *player1, *player2);
                            } else {
                                let player = player_id == *player1;
                                match key {
                                    Key::Left(true) => board.add_force(false, player),
                                    Key::Left(false) => board.reset_force(false, player),
                                    Key::Right(true) => board.add_force(true, player),
                                    Key::Right(false) => board.reset_force(true, player),
                                    Jump => board.apply_impulse(false, player)
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Game logic receive error, {e}")
        }
    }
}
