use std::fs::File;
use std::io::{BufRead, BufReader};
use raylib::prelude::*;

use crate::framebuffer::{Framebuffer, rgba_to_u32};
use crate::player::Player;
use crate::cast::cast_ray;
use crate::sprites::Sprite; 

pub type Maze = Vec<Vec<char>>;

#[derive(PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    Exiting,
}

pub fn load_maze_with_sprites(filename: &str, block_size: usize) -> (Maze, Vec<Sprite>) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut maze: Maze = Vec::new();
    let mut sprites: Vec<Sprite> = Vec::new();

    for (row_idx, line) in reader.lines().enumerate() {
        let mut row: Vec<char> = Vec::new();
        for (col_idx, ch) in line.unwrap().chars().enumerate() {
            match ch {
                'O' => {
                    let world_x = col_idx as f32 + 0.5;
                    let world_y = row_idx as f32 + 0.5;
                    sprites.push(Sprite {
                        pos: Vector2::new(world_x * block_size as f32,
                                          world_y * block_size as f32),
                        texture_name: "key".to_string(),
                        scale: 8.0,
                        damaging: false,
                    });
                    row.push(' ');
                }
                'A' => {
                    let world_x = col_idx as f32 + 0.5;
                    let world_y = row_idx as f32 + 0.5;
                    sprites.push(Sprite {
                        pos: Vector2::new(world_x * block_size as f32,
                                          world_y * block_size as f32),
                        texture_name: "spike".to_string(),
                        scale: 12.0,
                        damaging: true,
                    });
                    row.push(' ');
                }
                _ => {
                    row.push(ch);
                }
            }
        }
        maze.push(row);
    }

    (maze, sprites)
}


pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    pos: Vector2,
    with_rays: bool,
){
    // Pre-calcular colores como u32 para mejor rendimiento
    let black_color = rgba_to_u32(0, 0, 0, 255);
    let red_color = rgba_to_u32(255, 0, 0, 255);

    for (row_index, row) in maze.iter().enumerate(){
        for (col_index, &cell) in row.iter().enumerate(){
            let xo = (col_index * block_size) + pos.x as usize;
            let yo = (row_index * block_size) + pos.y as usize;
            draw_cell_optimized(framebuffer, xo, yo, block_size, cell, black_color, red_color);
        }
    }

    if with_rays {
        let num_rays = 25;
        for i in 0..num_rays {
            let current_ray = i as f32 / num_rays as f32;
            let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
            cast_ray(framebuffer, maze, player, block_size, a, pos, true);
        }
    }

    let scale = block_size as f32 / 30.0;
    let scaled_pos = Vector2::new(player.pos.x * scale, player.pos.y * scale);
    let triangle_size = block_size as f32 * 0.5;

    let fake_player = Player {
        pos: scaled_pos,
        a: player.a,
        fov: player.fov,
    };

    fake_player.draw_player_triangle(framebuffer, pos, triangle_size, Color::WHITESMOKE);
}

// Función optimizada para dibujar celdas
fn draw_cell_optimized(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
    black_color: u32,
    red_color: u32
){
    let color = match cell {
        ' ' => black_color,
        _ => red_color,
    };

    // Optimización: dibujar rectángulo sólido usando loops optimizados
    let end_x = xo + block_size;
    let end_y = yo + block_size;
    
    for y in yo..end_y {
        if y < framebuffer.height as usize {
            for x in xo..end_x {
                if x < framebuffer.width as usize {
                    framebuffer.set_pixel_fast(x as u32, y as u32, color);
                }
            }
        }
    }
}