use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, Sender};
use crate::server_logic::{GameStateSerialized, LogicMessage};
use crate::server_logic::LogicMessage::PlayerMsg;

pub fn start(socket: UdpSocket, logic_sender: Sender<LogicMessage>) {
    let mut buf = [0; 32];
    loop {
        log::debug!("Waiting for data...");
        match socket.recv_from(&mut buf) {
            Ok((len, sender_addr)) => {
                log::debug!("{} bytes received from {}, received: {:?}", len, sender_addr, &buf[..len]);
                match parse_packet(&buf[..len]) {
                    Ok(r) => match logic_sender.send(PlayerMsg(sender_addr, r)) {
                        Ok(()) => {}
                        Err(e) => log::error!("Cannot send player message, {e}")
                    },
                    Err(e) => log::warn!("parse error: {e:?}")
                };
            }
            Err(e) => log::error!("Error receiving data, kind: {}, error: {e}", {e.kind()})
        }
    }
}

pub enum SenderMsg {
    SetAddress(u64, u64, SocketAddr),
    GameLogicState(u64, GameStateSerialized)
}

pub fn start_sender(socket: UdpSocket, receiver: Receiver<SenderMsg>) {
    let mut addresses: HashMap<u64, SocketAddr> = HashMap::new();
    loop {
        match receiver.recv() {
            Ok(msg) => match msg {
                SenderMsg::SetAddress(player_id, board_id, addr) => {
                    addresses.insert(player_id, addr);
                    match socket.send_to(&parse_ids_to_packet(player_id, board_id), addr) {
                        Ok(_) => log::debug!("Set address packet sent"),
                        Err(e) => log::error!("Cannot send UDP set address, {e}")
                    }
                },
                SenderMsg::GameLogicState(id, state) => match addresses.get(&id) {
                    None => log::error!("Socket address not found for id {id}"),
                    Some(addr) => match socket.send_to(&parse_to_packet(&state), addr) {
                        Ok(_len) => {},
                        Err(e) => log::error!("Cannot send bytes, {e}")
                    }
                }
            }
            Err(e) => log::error!("Cannot receive udp message, {e}")
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Key {
    Left(bool),
    Right(bool),
    Jump
}

#[derive(Debug, PartialEq)]
pub enum MsgIn {
    GameRequest,
    Input(u64, u64, Key),
    Ping(u64, u64),
}

#[derive(Debug, PartialEq)]
struct ParseError;

fn parse_packet(data: &[u8]) -> Result<MsgIn, ParseError> {
    // byte protocol invented by me based on random opcodes
    if data.len() != 32 || data[..6] != [58, 41, 58, 80, 58, 68] {
        Err(ParseError)
    }
    else {
        let player_id = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let board_id = u64::from_le_bytes(data[16..24].try_into().unwrap());
        match data[6..8] {
            [11, 13] => Ok(MsgIn::GameRequest),
            [17, 23] => Ok(MsgIn::Input(player_id, board_id, Key::Left(true))),
            [25, 99] => Ok(MsgIn::Input(player_id, board_id, Key::Left(false))),
            [37, 31] => Ok(MsgIn::Input(player_id, board_id, Key::Right(true))),
            [67, 58] => Ok(MsgIn::Input(player_id, board_id, Key::Right(false))),
            [97, 33] => Ok(MsgIn::Input(player_id, board_id, Key::Jump)),
            [96, 22] => Ok(MsgIn::Ping(player_id, board_id)),
            _ => Err(ParseError)
        }
    }
}

fn parse_ids_to_packet(client_id: u64, board_id: u64) -> [u8; 32]{
    let mut result = [0; 32];
    result[..4].copy_from_slice(&[12, 64, 13, 56]);
    result[4..12].copy_from_slice(&client_id.to_le_bytes());
    result[12..20].copy_from_slice(&board_id.to_le_bytes());
    result
}

fn parse_to_packet(state: &GameStateSerialized) -> [u8; 32] {
    let ball_r = state.ball_radius.to_le_bytes();
    let ball_x = state.ball_pos.0.to_le_bytes();
    let ball_y = state.ball_pos.1.to_le_bytes();
    let player_r = state.player_radius.to_le_bytes();
    let player1_x = state.player1_pos.0.to_le_bytes();
    let player1_y = state.player1_pos.1.to_le_bytes();
    let player2_x = state.player2_pos.0.to_le_bytes();
    let player2_y = state.player2_pos.1.to_le_bytes();
    [ball_r, ball_x, ball_y, player_r, player1_x, player1_y, player2_x, player2_y].concat().try_into().unwrap()
}

mod test {
    use crate::udp_server::Key::{Jump, Left, Right};
    use crate::udp_server::MsgIn::{GameRequest, Input, Ping};
    use crate::udp_server::{parse_packet, ParseError};

    #[test]
    fn test_parse_packet() {
        let one = 1u64.to_le_bytes();
        assert_eq!(parse_packet(&[13, 14]), Err(ParseError));
        assert_eq!(parse_packet(&[13, 14, 31, 43, 53]), Err(ParseError));
        assert_eq!(parse_packet(&[]), Err(ParseError));
        assert_eq!(parse_packet(&[b":):P:D".as_slice(), &[11, 13]].concat()), Err(ParseError));
        assert_eq!(parse_packet(&[b":):P:D".as_slice(), &[11, 13], &[0; 8], &[0; 16]].concat()), Ok(GameRequest));
        assert_eq!(parse_packet(&[b":):P:D".as_slice(), &[17, 23], &one, &one, &[0; 8]].concat()), Ok(Input(1, 1, Left(true))));
        assert_eq!(parse_packet(&[b":):P:D".as_slice(), &[25, 99], &99u64.to_le_bytes(), &one, &[0; 8]].concat()), Ok(Input(99, 1, Left(false))));
        assert_eq!(parse_packet(&[b":):P:D".as_slice(), &[37, 31], &99u64.to_le_bytes(), &one, &[0; 8]].concat()), Ok(Input(99, 1, Right(true))));
        assert_eq!(parse_packet(&[58, 41, 58, 80, 58, 68, 67, 58, 2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), Ok(Input(258, 1, Right(false))));
        assert_eq!(parse_packet(&[58, 41, 58, 80, 58, 68, 97, 33, 7, 0, 1, 0, 0, 0, 0, 0, 163, 49, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), Ok(Input(65543, 78243, Jump)));
        assert_eq!(parse_packet(&[b":):P:D".as_slice(), &[96, 22], &[197], &[0; 7], &one, &[0; 8]].concat()), Ok(Ping(197, 1)));
    }
}
