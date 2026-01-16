use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::server_logic::LogicMessage;
use crate::server_logic::LogicMessage::PlayerMsg;
use crate::udp_server;
use crate::udp_server::{MsgIn, PacketMsg};

#[derive(Copy, Clone, Debug)]
pub enum TcpMessage {
    DisconnectPlayer,
    SetOpponent(u64),
}

pub fn start(sender: Sender<LogicMessage>) {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap()
        .block_on(async { run(sender).await });
    log::error!("TCP server stopped");
}

async fn run(sender: Sender<LogicMessage>) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:12541").await.expect("Cannot bind");
    // todo is unwrap safe on Arc<Mutex<u64>> in this case?
    let counter: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    // let mut game_logic_timer = tokio::time::interval(Duration::from_secs_f32(1.0 / 60.0));
    // let mut game_logic_timer = tokio::time::interval(Duration::from_secs_f32(1.0 / 70.0));
    // let mut game_logic_timer = tokio::time::interval(Duration::from_secs_f32(1.0 / 30.0));
    let mut game_logic_timer = tokio::time::interval(Duration::from_secs_f32(1.0 / 100.0));
    let sender_clone = sender.clone();

    loop {
        tokio::select! {
            _ = game_logic_timer.tick() => {
                if let Err(e) = sender_clone.send(LogicMessage::CalculateBoard) {
                    log::error!("Cannot send GameLogic tick");
                }
            }
            incoming = listener.accept() => match incoming {
                Ok((stream, addr)) => {
                    let logic_sender = sender.clone();
                    let counter_clone = Arc::clone(&counter);
                    tokio::spawn(async move {
                        {
                            let mut c = counter_clone.lock().unwrap();
                            *c += 1;
                            log::debug!("TCP connection, counter: {c}");
                        }
                        handle_connection(stream, addr, logic_sender).await;
                        {
                            let mut c = counter_clone.lock().unwrap();
                            *c -= 1;
                            log::debug!("TCP disconnection, counter: {c:?}");
                        }
                    });
                }
                Err(e) => {
                    log::error!("Could not accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(mut stream: tokio::net::TcpStream, addr: SocketAddr, logic_sender: Sender<LogicMessage>) {
    let mut ping_timer = tokio::time::interval(Duration::from_secs(10));
    // let mut timer2 = tokio::time::interval(Duration::from_secs(5));
    // loop {
    //     tokio::select! {
    //         _ = timer1.tick() => {
    //             log::debug!("timer 1");
    //         }
    //         _ = timer2.tick() => {
    //             log::debug!("timer 2");
    //         }
    //     }
    // }

    let player_id: u64 = rand::rng().random();
    let mut opponent_id: Option<u64> = None;
    let mut last_ping = Instant::now();
    let mut buffer = vec![0; 1024];
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    logic_sender.send(LogicMessage::SetChannel(player_id, sender)).unwrap();
    log::info!("TCP connection accepted: {:?}, player_id: {player_id}", addr);
    loop {
        tokio::select! {
            _ = ping_timer.tick() => {
                if last_ping.elapsed() > Duration::from_secs(30) {
                    log::debug!("No ping, disconnect, {player_id}");
                    if let Err(e) = logic_sender.send(LogicMessage::Disconnect(player_id, opponent_id)) {
                        log::error!("Cannot send LogicMessage");
                    }
                }
            }
            Some(ch_recv) = receiver.recv() => {
                match ch_recv {
                    TcpMessage::DisconnectPlayer => {
                        log::debug!("Disconnecting player {player_id} after Server message");
                        break;
                    }
                    TcpMessage::SetOpponent(opponent) => opponent_id = Some(opponent),
                }
            }
            res = stream.read(&mut buffer) => {
                match res {
                    Ok(0) => {
                        log::debug!("Connection closed, {player_id}");
                        if let Err(e) = logic_sender.send(LogicMessage::Disconnect(player_id, opponent_id)) {
                            log::error!("Cannot send LogicMessage");
                        }
                        break;
                    }
                    Ok(len) => {
                        log::debug!("Read {} bytes from {} client: {:?}", len, player_id, &buffer[..len]);
                        match udp_server::parse_packet(&buffer[..len]) {
                            Ok(m) => match m {
                                PacketMsg::PlayerIdRequest => {
                                    if let Err(e) = stream.write_all(&player_id.to_le_bytes()).await {
                                        log::error!("Cannot send TCP, {player_id}, error: {e}");
                                        if let Err(e) = logic_sender.send(LogicMessage::Disconnect(player_id, opponent_id)) {
                                            log::error!("Cannot send LogicMessage");
                                        }
                                        break;
                                    }
                                }
                                // todo remove player_id and board_id from ping, it is recognize by the connection itself
                                PacketMsg::Ping(_player_id, _board_id) => _ = last_ping = Instant::now(),
                                m => log::debug!("Unexpected TCP message: {m:?}")
                            }
                            Err(e) => {
                                log::error!("Error parsing packet: {e:?}");
                            }
                        };
                    }
                    Err(e) => {
                        log::warn!("Error reading from stream, {player_id}, error: {}", e);
                        if let Err(e) = logic_sender.send(LogicMessage::Disconnect(player_id, opponent_id)) {
                            log::error!("Cannot send LogicMessage");
                        }
                        break;
                    }
                }
            }
        }
    }
    log::info!("TCP task finished");
}
