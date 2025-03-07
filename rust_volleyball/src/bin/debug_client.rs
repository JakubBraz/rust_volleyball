use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;
use macroquad::prelude::*;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
const RESIZE_FACTOR: f32 = 100.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "VolleyBall".to_string(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();

    let msg = [58, 41, 58, 80, 58, 68, 11, 13];
    let mut packet = [0; 32];
    packet[..8].copy_from_slice(&msg);
    socket.send_to(&packet, ("127.0.0.1", 12542)).unwrap();
    let mut buff = [0; 32];
    println!("waiting 1");
    let len = socket.recv(&mut buff).unwrap();
    println!("received 1: {:?}", &buff[..len]);

    let player_id: [u8; 8] = buff[4..12].try_into().unwrap();
    let board_id: [u8; 8] = buff[12..20].try_into().unwrap();

    let mut ping_time = Instant::now();

    let mut packet = [0; 32];
    packet[..6].copy_from_slice(&[58, 41, 58, 80, 58, 68]);
    packet[8..16].copy_from_slice(&player_id);
    packet[16..24].copy_from_slice(&board_id);
    loop {
        if ping_time.elapsed().as_secs() >= 1 {
            packet[6..8].copy_from_slice(&[96, 22]);
            socket.send_to(&packet, ("127.0.0.1", 12542)).unwrap();
            ping_time = Instant::now();
            println!("ping!");
        }
        // PLAYER INPUT
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Up) {
            packet[6..8].copy_from_slice(&[97, 33]);
            socket.send_to(&packet, ("127.0.0.1", 12542)).unwrap();
        }
        if is_key_pressed(KeyCode::Left) {
            packet[6.. 8].copy_from_slice(&[17, 23]);
            socket.send_to(&packet, ("127.0.0.1", 12542)).unwrap();
        }
        if is_key_pressed(KeyCode::Right) {
            packet[6..8].copy_from_slice(&[37, 31]);
            socket.send_to(&packet, ("127.0.0.1", 12542)).unwrap();
        }
        if is_key_released(KeyCode::Left) {
            packet[6..8].copy_from_slice(&[25, 99]);
            socket.send_to(&packet, ("127.0.0.1", 12542)).unwrap();
        }
        if is_key_released(KeyCode::Right) {
            packet[6..8].copy_from_slice(&[67, 58]);
            socket.send_to(&packet, ("127.0.0.1", 12542)).unwrap();
        }

        // UPDATE STATE
        // println!("waiting ...");
        socket.recv(&mut buff).unwrap();
        // println!("received 2: {:?}", buff);
        let br = f32::from_be_bytes(buff[0..4].try_into().unwrap());
        let bx = f32::from_be_bytes(buff[4..8].try_into().unwrap());
        let by = f32::from_be_bytes(buff[8..12].try_into().unwrap());
        let pr = f32::from_be_bytes(buff[12..16].try_into().unwrap());
        let p1x = f32::from_be_bytes(buff[16..20].try_into().unwrap());
        let p1y = f32::from_be_bytes(buff[20..24].try_into().unwrap());
        let p2x = f32::from_be_bytes(buff[24..28].try_into().unwrap());
        let p2y = f32::from_be_bytes(buff[28..32].try_into().unwrap());

        // DRAW STATE
        clear_background(Color::new(0.2, 0.5, 0.7, 1.0));
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        let (xp1, yp1, rp1) = resize_ball_shape((p1x, p1y, pr));
        let (xp2, yp2, rp2) = resize_ball_shape((p2x, p2y, pr));
        draw_circle(xp1, yp1, rp1, RED);
        draw_circle(xp2, yp2, rp2, GREEN);
        let (x_p, y_p, r_p) = resize_ball_shape((bx, by, br));
        draw_circle(x_p, y_p, r_p, YELLOW);
        // let (x_g, y_g, w_g, h_g) = resize_box_shape(game_state.ground());
        // draw_rectangle(x_g, y_g, w_g, h_g, BROWN);
        // let (xn, yn, wn, hn) = resize_box_shape(game_state.net());
        // draw_rectangle(xn, yn, wn, hn, BROWN);

        next_frame().await
    }
}

fn resize_ball_shape(player: (f32, f32, f32)) -> (f32, f32, f32) {
    let(x, y, r) = player;
    (x * RESIZE_FACTOR, y * -RESIZE_FACTOR + HEIGHT, r * RESIZE_FACTOR)
}

fn resize_box_shape(ground: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let (x, y, w, h) = ground;
    // ((x - w) * RESIZE_FACTOR, (y - h) * RESIZE_FACTOR + HEIGHT, w * RESIZE_FACTOR * 2.0, h * RESIZE_FACTOR * 2.0)
    ((x - w) * RESIZE_FACTOR, (HEIGHT / RESIZE_FACTOR - y - h) * RESIZE_FACTOR, w * RESIZE_FACTOR * 2.0, h * RESIZE_FACTOR * 2.0)
}
