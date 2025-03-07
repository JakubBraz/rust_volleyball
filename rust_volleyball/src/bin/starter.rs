use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread::spawn;
use rust_volleyball::{server_logic, tcp_server, udp_server};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("debug"))
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();
    log::info!("Main start");

    let udp_socket = UdpSocket::bind("0.0.0.0:12542").unwrap();
    let socket_sender = udp_socket.try_clone().unwrap();

    let (logic_sender, logic_receiver) = channel();
    let udp_logic_sender = logic_sender.clone();

    let (udp_sender_ch, udp_receiver_ch) = channel();

    let udp_sender = spawn(move || udp_server::start_sender(socket_sender, udp_receiver_ch));
    let udp_server = spawn(move || udp_server::start(udp_socket, udp_logic_sender));
    let tcp_server = spawn(move || tcp_server::start());
    let server_logic = spawn(move || server_logic::start(logic_sender, logic_receiver, udp_sender_ch));

    udp_server.join().unwrap();
    tcp_server.join().unwrap();
    udp_sender.join().unwrap();
    server_logic.join().unwrap();
}
