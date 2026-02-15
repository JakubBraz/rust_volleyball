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
            game_state.apply_impulse(true, true);
        }
        if is_key_pressed(KeyCode::Up) {
            game_state.apply_impulse(false, true);
        }
        if is_key_pressed(KeyCode::Left) {
            game_state.add_force(false, true);
        }
        if is_key_pressed(KeyCode::Right) {
            game_state.add_force(true, true);
        }
        if is_key_released(KeyCode::Left) {
            game_state.reset_force(false, true);
        }
        if is_key_released(KeyCode::Right) {
            game_state.reset_force(true, true);
        }
        if is_key_pressed(KeyCode::W) {
            game_state.apply_impulse(false, false);
        }
        if is_key_pressed(KeyCode::A) {
            game_state.add_force(false, false);
        }
        if is_key_pressed(KeyCode::D) {
            game_state.add_force(true, false);
        }
        if is_key_released(KeyCode::A) {
            game_state.reset_force(false, false);
        }
        if is_key_released(KeyCode::D) {
            game_state.reset_force(true, false);
        }

        // UPDATE STATE
        game_state.step();

        // DRAW STATE
        clear_background(Color::new(0.2, 0.5, 0.7, 1.0));
        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        let (xp1, yp1, rp1, xp2, yp2, rp2) = game_state.players();
        let (xp1, yp1, rp1) = resize_ball_shape((xp1, yp1, rp1));
        let (xp2, yp2, rp2) = resize_ball_shape((xp2, yp2, rp2));
        draw_circle(xp1, yp1, rp1, RED);
        draw_circle(xp2, yp2, rp2, GREEN);
        let (gsx, gsy, gsr) = game_state.ball();
        println!("game_state player1 {} {} {}", game_state.players().0, game_state.players().1, game_state.players().2);
        println!("resized {} {} {}", xp1, yp1, rp1);
        let (x_p, y_p, r_p) = resize_ball_shape(game_state.ball());
        draw_circle(x_p, y_p, r_p, YELLOW);
        let (x_g, y_g, w_g, h_g) = resize_box_shape(game_state.ground());
        draw_rectangle(x_g, y_g, w_g, h_g, BROWN);
        let (xn, yn, wn, hn) = resize_box_shape(game_state.net());
        draw_rectangle(xn, yn, wn, hn, BROWN);

        let (p1, p2, game_over) = game_state.points();
        draw_text(&p1.to_string(), WIDTH / 2.0 + 120.0, 60.0, 100.0, BLACK);
        draw_text(&p2.to_string(), WIDTH / 2.0 - 170.0, 60.0, 100.0, BLACK);
        if game_over {
            let winner = if p1 > p2 { "Player 1" } else { "Player 2" };
            draw_text(&format!("{winner} won!"), 120.0, 150.0, 100.0, BLACK);
        }

        // if loop_counter % 5 == 0 {
        //     println!("player1 x: {} y: {} r: {}", xp1, yp1, rp1);
        //     println!("player2 x: {} y: {} r: {}", xp2, yp2, rp2);
        //     println!("ball x: {} y: {} r: {}", x_p, y_p, y_p);
        //     println!("{} {}", screen_width(), screen_height());
        //     println!();
        //     println!("ground, x: {} y: {} width: {} height: {}", x_g, y_g, w_g, h_g);
        // }

        loop_counter += 1;
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
