use raylib::prelude::*;
use crate::framebuffer::{Framebuffer, rgba_to_u32};
use crate::player::Player;
use crate::game::Maze;

pub struct Intersect{
    pub distance: f32,
    pub impact: char
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    a: f32,
    init_pos: Vector2,
    draw_line: bool
) -> Intersect {
    let mut d = 0.0;
    let max_distance = 1000.0;
    let step_size = 1.0; // Incrementado para mejor rendimiento
    
    // Pre-calcular color para el rayo
    let ray_color = rgba_to_u32(212, 255, 247, 100);
    
    // Pre-calcular cos y sin para evitar recálculos
    let cos_a = a.cos();
    let sin_a = a.sin();
    let init_x = player.pos.x + init_pos.x;
    let init_y = player.pos.y + init_pos.y;

    while d < max_distance {
        let x = init_x + d * cos_a;
        let y = init_y + d * sin_a;

        let maze_x = x as usize / block_size;
        let maze_y = y as usize / block_size;

        if maze_y >= maze.len() || maze_x >= maze[0].len() {
            break;
        }

        if maze[maze_y][maze_x] != ' ' {
            return Intersect {
                distance: d,
                impact: maze[maze_y][maze_x]
            };
        }

        if draw_line {
            framebuffer.set_pixel_fast(x as u32, y as u32, ray_color);
        }
        
        d += step_size;
    }
    
    Intersect { distance: max_distance, impact: ' ' }
}