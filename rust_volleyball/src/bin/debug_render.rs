use rust_volleyball::fn1;
use macroquad::prelude::*;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
const RESIZE_FACTOR: f32 = 200.0;

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
    let mut game_state = rust_volleyball::GameState::new();
    let mut loop_counter: u64 = 0;

    loop {
        // PLAYER INPUT
        if is_key_pressed(KeyCode::R) {
            game_state = rust_volleyball::GameState::new();
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::Space) {
            game_state.apply_impulse(true);
        }
        if is_key_pressed(KeyCode::Up) {
            game_state.apply_impulse(false);
        }
        if is_key_pressed(KeyCode::Left) {
            game_state.add_force(false);
        }
        if is_key_pressed(KeyCode::Right) {
            game_state.add_force(true);
        }
        if is_key_released(KeyCode::Left) || is_key_released(KeyCode::Right) {
            game_state.reset_force()
        }

        // UPDATE STATE
        game_state.step();

        // DRAW STATE
        clear_background(Color::new(0.2, 0.5, 0.7, 1.0));
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        let (x_p, y_p, r_p) = resize_player(game_state.player1());
        draw_circle(x_p, y_p, r_p, YELLOW);
        let(x_g, y_g, w_g, h_g) = resize_ground(game_state.ground());
        draw_rectangle(x_g, y_g, w_g, h_g, BROWN);

        if loop_counter % 5 == 0 {
            println!("x: {}", x_p);
            println!("y: {}", y_p);
            println!("r: {}", r_p);
            println!("{} {}", screen_width(), screen_height());
            println!();
            println!("ground, x: {} y: {} width: {} height: {}", x_g, y_g, w_g, h_g);
        }
        draw_text(&format!("Fn result = {}", fn1(10,20)), 20.0, 20.0, 30.0, DARKGRAY);

        loop_counter += 1;
        next_frame().await
    }
}

fn resize_player(player: (f32, f32, f32)) -> (f32, f32, f32) {
    let(x, y, r) = player;
    (x * RESIZE_FACTOR, y * -RESIZE_FACTOR + HEIGHT, r * RESIZE_FACTOR)
}

fn resize_ground(ground: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let (x, y, w, h) = ground;
    ((x - w) * RESIZE_FACTOR, (y - h) * RESIZE_FACTOR + HEIGHT, w * RESIZE_FACTOR * 2.0, h * RESIZE_FACTOR * 2.0)
}
