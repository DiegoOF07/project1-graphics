mod framebuffer;
mod game;
mod player;
mod cast;
mod events;
mod renderer;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use game::{load_maze, render_maze, GameState};
use player::Player;
use events::process_events;
use renderer::render_world;

fn main() {
    let window_width = 845;
    let window_height = 585;
    let block_size = 65 as usize;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Doom estilo clásico")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.set_target_fps(60); // Aumentado a 60 por el mejor rendimiento
    window.hide_cursor();
    // NO usar disable_cursor() - interfiere con nuestro sistema manual

    let mut game_state = GameState::Menu;

    // Bucle principal
    while !window.window_should_close() && game_state != GameState::Exiting {
        match game_state {
            GameState::Menu => {
                let mut d = window.begin_drawing(&raylib_thread);
                d.clear_background(Color::BLACK);
                d.draw_text(">> DOOM CLÁSICO <<", window_width / 2 - 150, 100, 30, Color::RAYWHITE);
                d.draw_text("Presiona ENTER para jugar", window_width / 2 - 150, 200, 20, Color::LIGHTGRAY);
                d.draw_text("Presiona ESC para salir", window_width / 2 - 150, 230, 20, Color::LIGHTGRAY);
                d.draw_text("Controles:", window_width / 2 - 150, 280, 16, Color::GRAY);
                d.draw_text("WASD: Movimiento | Q/E: Strafe", window_width / 2 - 150, 300, 14, Color::GRAY);
                d.draw_text("Mouse: Mirar | M: Cambiar vista", window_width / 2 - 150, 320, 14, Color::GRAY);

                if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    // Preparar para el juego: capturar cursor
                    game_state = GameState::Playing;
                } else if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    game_state = GameState::Exiting;
                }
            }

            GameState::Playing => {
                let framebuffer_width = 845;
                let framebuffer_height = 585;

                let maze = load_maze("./maze.txt");
                let mut player = Player::new(Vector2::new(1.5 * block_size as f32, 1.5 * block_size as f32));
                let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height, Color::BLACK);

                let mut mode = "3D";
                
                // Centrar el mouse al iniciar el juego
                window.set_mouse_position(Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0));
                let mut last_mouse_x = window.get_mouse_x();

                // Bucle del juego
                while !window.window_should_close() {
                    if window.is_key_pressed(KeyboardKey::KEY_M) {
                        mode = if mode == "2D" { "3D" } else { "2D" };
                    }

                    if window.is_key_pressed(KeyboardKey::KEY_P) {
                        window.show_cursor(); // Mostrar cursor al salir del juego
                        game_state = GameState::Menu;
                        break;
                    }

                    framebuffer.clear();
                    if mode == "2D" {
                        render_maze(&mut framebuffer, &maze, &player, block_size, Vector2::new(0.0, 0.0), true);
                    } else {
                        render_world(&mut framebuffer, &maze, &player, block_size);
                        render_maze(&mut framebuffer, &maze, &player, block_size - 55, Vector2::new((window_width - 130) as f32, 0.0), false);
                        process_events(&mut window, &mut player, &maze, &mut last_mouse_x, block_size);
                    }

                    framebuffer.swap_buffers(&mut window, &raylib_thread, |d| {
                        d.draw_text(&format!("FPS: {}", d.get_fps()), 10, 10, 20, Color::WHITE);
                    });
                }
            }

            GameState::Exiting => break,
        }
    }
}