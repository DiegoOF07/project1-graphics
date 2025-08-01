mod framebuffer;
mod maze;
mod player;
mod cast;
mod events;
mod renderer;

use std::time::Duration;
use std::thread;
use raylib::prelude::*;
use framebuffer::Framebuffer;
use maze::{load_maze, render_maze};
use player::Player;
use events::process_events;
use renderer::render_world;

fn main(){
    let window_width = 845;
    let window_height = 585;
    let framebuffer_width = 845;
    let framebuffer_height = 585;
    let block_size = 65 as usize;
    let maze = load_maze("./maze.txt");
    let mut player = Player::new(Vector2::new(1.5 * block_size as f32,1.5 * block_size as f32));


    let(mut window, raylib_thread) = raylib::init()
    .size(window_width, window_height)
    .title("Prueba juego")
    .log_level(TraceLogLevel::LOG_WARNING)
    .build();
    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height, Color::WHITE);

    framebuffer.set_background_color(Color::BLACK);
    let mut mode = "3D";
    while !window.window_should_close() {
        framebuffer.clear();

        if window.is_key_pressed(KeyboardKey::KEY_M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }

        if mode == "2D" {
            render_maze(&mut framebuffer, &maze, &player, block_size, Vector2::new(0.0,0.0), true);
        } else {
            render_world(&mut framebuffer, &maze, &player, block_size);
            //Funciona como mini mapa
            render_maze(&mut framebuffer, &maze, &player, block_size-55, Vector2::new((window_width-130) as f32, 0.0),false);
            

        }

        process_events(&window, &mut player, &maze, block_size);
        framebuffer.swap_buffers(&mut window, &raylib_thread);
        thread::sleep(Duration::from_millis(16));
    }


}

