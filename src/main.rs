mod framebuffer;
mod game;
mod player;
mod cast;
mod events;
mod renderer;
mod texture;
mod sprites;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use game::{load_maze_with_sprites, render_maze, GameState};
use player::Player;
use events::process_events;
use renderer::{render_world, render_world_with_textures_downscale};
use texture::TextureManager;
use sprites::render_sprites;

fn main() {
    let window_width = 930;
    let window_height = 630;
    let block_size = 30 as usize;
    let wall_textures = [
        "./textures/wall1.png",
        "./textures/wall2.png",
        "./textures/wall3.png",
    ];
    let floor_textures = [
        "./textures/floor1.png",
        "./textures/floor2.jpg",
        "./textures/floor3.png",
    ];

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster Game")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.set_target_fps(60);
    window.hide_cursor();
    let mut texture_manager = TextureManager::new();

    texture_manager.generate_default_textures();
    texture_manager.load_sprite_texture("key", "./textures/sprites/key.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("spike", "./textures/sprites/spike.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("fire1", "./textures/sprites/fire1.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("fire2", "./textures/sprites/fire2.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("fire3", "./textures/sprites/fire3.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("heal1", "./textures/sprites/heal1.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("heal2", "./textures/sprites/heal2.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("heal3", "./textures/sprites/heal3.png", &mut window, &raylib_thread).ok();
    texture_manager.load_sprite_texture("heal4", "./textures/sprites/heal4.png", &mut window, &raylib_thread).ok();
    
    // Cargar y mantener la textura de fondo del menú como Texture2D de Raylib
    let menu_bg_texture = match Image::load_image("./textures/menu_bg.jpg") {
        Ok(img) => Some(window.load_texture_from_image(&raylib_thread, &img).unwrap()),
        Err(_) => {
            println!("No se pudo cargar la imagen de fondo del menú, usando color sólido");
            None
        }
    };

    let mut game_state = GameState::Menu;
    let level_files = ["./levels/level1.txt", "./levels/level2.txt", "./levels/level3.txt"];
    let level_names = ["Nivel 1", "Nivel 2", "Nivel 3"];
    let mut selected_level = 0;

    while !window.window_should_close() && game_state != GameState::Exiting {
        match game_state {
            GameState::Menu => {
                let mut d = window.begin_drawing(&raylib_thread);
                
                // Renderizar fondo
                if let Some(bg_texture) = &menu_bg_texture {
                    // Calcular escalado para que la imagen cubra toda la pantalla
                    let scale_x = window_width as f32 / bg_texture.width as f32;
                    let scale_y = window_height as f32 / bg_texture.height as f32;
                    let scale = scale_x.max(scale_y); // Usar el mayor para cubrir toda la pantalla
                    
                    // Centrar la imagen
                    let scaled_width = bg_texture.width as f32 * scale;
                    let scaled_height = bg_texture.height as f32 * scale;
                    let offset_x = (window_width as f32 - scaled_width) / 2.0;
                    let offset_y = (window_height as f32 - scaled_height) / 2.0;
                    
                    d.draw_texture_ex(
                        bg_texture,
                        Vector2::new(offset_x, offset_y),
                        0.0,
                        scale,
                        Color::WHITE,
                    );
                } else {
                    // Fondo alternativo si no se puede cargar la imagen
                    d.clear_background(Color::new(20, 20, 40, 255));
                }
                
                // Overlay semi-transparente para mejorar legibilidad
                d.draw_rectangle(0, 0, window_width, window_height, Color::new(0, 0, 0, 100));
                
                // Título con contorno para mejor visibilidad
                let title_text = ">> Horror Game <<";
                let title_x = window_width / 2 - 150;
                let title_y = 100;
                
                // Contorno del título
                for dx in -2..=2 {
                    for dy in -2..=2 {
                        if dx != 0 || dy != 0 {
                            d.draw_text(title_text, title_x + dx, title_y + dy, 30, Color::BLACK);
                        }
                    }
                }
                d.draw_text(title_text, title_x, title_y, 30, Color::RAYWHITE);
                
                // Instrucciones con contorno
                let instruction_text = "Selecciona un nivel:";
                let inst_x = window_width / 2 - 150;
                let inst_y = 180;
                
                // Contorno de instrucciones
                d.draw_text(instruction_text, inst_x + 1, inst_y + 1, 22, Color::BLACK);
                d.draw_text(instruction_text, inst_x, inst_y, 22, Color::LIGHTGRAY);

                // Dibujar opciones de nivel con selección y contorno
                for (i, name) in level_names.iter().enumerate() {
                    let color = if i == selected_level { Color::YELLOW } else { Color::GRAY };
                    let marker = if i == selected_level { ">" } else { " " };
                    let level_text = format!("{} {}", marker, name);
                    let level_x = window_width / 2 - 100;
                    let level_y = 220 + i as i32 * 40;
                    
                    // Contorno para opciones de nivel
                    d.draw_text(&level_text, level_x + 1, level_y + 1, 28, Color::BLACK);
                    d.draw_text(&level_text, level_x, level_y, 28, color);
                }

                // Instrucciones de control con contorno
                let control_texts = [
                    "Usa W/S para cambiar nivel",
                    "ENTER para jugar | ESC para salir"
                ];
                
                for (i, text) in control_texts.iter().enumerate() {
                    let ctrl_x = window_width / 2 - 150;
                    let ctrl_y = 370 + i as i32 * 30;
                    
                    d.draw_text(text, ctrl_x + 1, ctrl_y + 1, 18, Color::BLACK);
                    d.draw_text(text, ctrl_x, ctrl_y, 18, Color::DARKGRAY);
                }

                // Navegación con W/S
                if d.is_key_pressed(KeyboardKey::KEY_W) {
                    if selected_level > 0 {
                        selected_level -= 1;
                    }
                }
                if d.is_key_pressed(KeyboardKey::KEY_S) {
                    if selected_level < level_files.len() - 1 {
                        selected_level += 1;
                    }
                }

                if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    game_state = GameState::Playing;
                } else if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    game_state = GameState::Exiting;
                }
            }

            GameState::Playing => {
                let framebuffer_width = 930;
                let framebuffer_height = 630;

                texture_manager.load_wall_texture('-', wall_textures[selected_level], &mut window, &raylib_thread).ok();
                texture_manager.load_wall_texture('|', wall_textures[selected_level], &mut window, &raylib_thread).ok();
                texture_manager.load_wall_texture('+', wall_textures[selected_level], &mut window, &raylib_thread).ok();
                texture_manager.load_floor_texture(floor_textures[selected_level], &mut window, &raylib_thread).ok();

                let (maze, mut sprites) = load_maze_with_sprites(level_files[selected_level], block_size);
                let mut player = Player::new(Vector2::new(1.5 * block_size as f32, 1.5 * block_size as f32));
                let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height, Color::BLACK);

                let mut mode = "3D";
                let mut use_textures = true;

                window.set_mouse_position(Vector2::new(window_width as f32 / 2.0, window_height as f32 / 2.0));
                let mut last_mouse_x = window.get_mouse_x();

                while !window.window_should_close() {
                    if window.is_key_pressed(KeyboardKey::KEY_M) {
                        mode = if mode == "2D" { "3D" } else { "2D" };
                    }
                    if window.is_key_pressed(KeyboardKey::KEY_T) {
                        use_textures = !use_textures;
                    }
                    if window.is_key_pressed(KeyboardKey::KEY_P) {
                        window.show_cursor();
                        game_state = GameState::Menu;
                        break;
                    }

                    framebuffer.clear();
                    if mode == "2D" {
                        render_maze(&mut framebuffer, &maze, &player, block_size, Vector2::new(0.0, 0.0), true);
                    } else {
                        if use_textures {
                            let mut depth_buffer = render_world_with_textures_downscale(&mut framebuffer, &maze, &player, block_size, &texture_manager);
                            render_sprites(&mut framebuffer, &mut sprites, &player, &texture_manager, &depth_buffer);
                        } else {
                            render_world(&mut framebuffer, &maze, &player, block_size);
                        }
                        render_maze(&mut framebuffer, &maze, &player, block_size - 20, Vector2::new((window_width - 310) as f32, 0.0), false);
                    }
                    process_events(&mut window, &mut player, &maze, &mut last_mouse_x, block_size);

                    framebuffer.swap_buffers(&mut window, &raylib_thread, |d| {
                        d.draw_text(&format!("FPS: {}", d.get_fps()), 10, 10, 20, Color::WHITE);
                        d.draw_text(&format!("Modo: {}", mode), 10, 35, 16, Color::WHITE);
                        d.draw_text(&format!("Texturas: {}", if use_textures { "ON" } else { "OFF" }), 10, 55, 16, Color::WHITE);
                        d.draw_text("M: Cambiar modo | T: Toggle texturas | P: Menú", 10, 75, 14, Color::LIGHTGRAY);
                    });
                }
            }

            GameState::Exiting => break,
        }
    }
}