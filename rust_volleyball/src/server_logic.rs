use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use log::{debug, error};
use rand::Rng;
use tokio::sync::mpsc::UnboundedSender;
use crate::GameState;
use crate::tcp_server::TcpMessage;
use crate::udp_server::Key::Jump;
use crate::udp_server::{Key, MsgIn, SenderMsg};

pub enum LogicMessage {
    CalculateBoard,
    PlayerMsg(SocketAddr, MsgIn),
    SetChannel(u64, UnboundedSender<TcpMessage>),
    Disconnect(u64, Option<u64>),
}

#[derive(Copy, Clone)]
pub struct GameStateSerialized {
    pub ball_pos: (f32, f32),
    pub ball_radius: f32,
    // pub ball_angle: f32,
    pub player_radius: f32,
    pub player1_pos: (f32, f32),
    pub player2_pos: (f32, f32),
    pub score1: u32,
    pub score2: u32,
    pub game_over: bool,
}

pub fn start(logic_sender: Sender<LogicMessage>, logic_receiver: Receiver<LogicMessage>, udp_sender: Sender<SenderMsg>) {
    let mut player_in_lobby: Option<(u64, u64)> = None;
    let mut boards: HashMap<u64, (u64, u64, GameState)> = HashMap::new();
    let mut player_channels: HashMap<u64, UnboundedSender<TcpMessage>> = HashMap::new();
    let mut rng = rand::rng();

    loop {
        match logic_receiver.recv() {
            Ok(m) => match m {
                LogicMessage::CalculateBoard => {
                    boards.retain(|_board_id, (player1, player2, board)| {
                        if !player_channels.contains_key(player1) && !player_channels.contains_key(player2) {
                            false
                        }
                        else {
                            if board.step() {
                                let (bx, by, br) = board.ball();
                                let (p1x, p1y, p1r, p2x, p2y, _p2r) = board.players();
                                let (score1, score2, game_over) = board.points();
                                let serialized = GameStateSerialized {
                                    ball_pos: (bx, by),
                                    ball_radius: br,
                                    player_radius: p1r,
                                    player1_pos: (p1x, p1y),
                                    player2_pos: (p2x, p2y),
                                    score1,
                                    score2,
                                    game_over
                                };
                                notify(&udp_sender, SenderMsg::GameLogicState(*player1, serialized));
                                notify(&udp_sender, SenderMsg::GameLogicState(*player2, serialized));
                            }
                            true
                        }
                    });
                }
                LogicMessage::SetChannel(player_id, channel) => {
                    player_channels.insert(player_id, channel);
                }
                LogicMessage::PlayerMsg(addr, msg) => match msg {
                    MsgIn::GameRequest(player_id) => {
                        let (new_player_id, board_id) = match player_in_lobby {
                            None => {
                                let game_id: u64 = rng.random();
                                player_in_lobby = Some((player_id, game_id));
                                (player_id, game_id)
                            }
                            Some((waiting_player_id, board_id)) => {
                                let game = GameState::new();
                                boards.insert(board_id, (waiting_player_id, player_id, game));
                                player_in_lobby = None;
                                send_tcp_message(&player_channels, player_id, TcpMessage::SetOpponent(waiting_player_id));
                                send_tcp_message(&player_channels, waiting_player_id, TcpMessage::SetOpponent(player_id));
                                (player_id, board_id)
                            }
                        };
                        notify(&udp_sender, SenderMsg::SetAddress(new_player_id, board_id, addr));
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
                    },
                }
                LogicMessage::Disconnect(player, opponent) => {
                    log::debug!("Player {player}, opponent {opponent:?} disconnects");
                    [Some(player), opponent].iter().flatten().for_each(|&player_id| {
                        send_tcp_message(&player_channels, player_id, TcpMessage::DisconnectPlayer);
                        player_channels.remove(&player_id);
                        notify(&udp_sender, SenderMsg::ForgetAddress(player_id));
                        if let Some((p_id, _b_id)) = player_in_lobby && player_id == p_id {
                            player_in_lobby = None;
                        }
                    });
                }
            }
            Err(e) => error!("Game logic receive error, {e}")
        }
    }
}

fn notify(sender: &Sender<SenderMsg>, msg: SenderMsg) {
    match sender.send(msg) {
        Ok(_) => {}
        Err(e) => log::warn!("Cannot send UPD game state, {e}")
    }
}

fn send_tcp_message(channels: &HashMap<u64, UnboundedSender<TcpMessage>>, player_id: u64, message: TcpMessage) {
    match channels.get(&player_id) {
        None => log::error!("Player {player_id} channel not found"),
        Some(channel) => if let Err(e) = channel.send(message) {
            log::warn!("Cannot send message {message:?}, {e}");
        }
    }
}
