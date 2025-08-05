use crate::framebuffer::{Framebuffer, rgba_to_u32};
use crate::player::Player;
use crate::game::Maze;
use crate::cast::{cast_ray, Intersect};

use raylib::prelude::*;

pub fn render_world(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    let screen_width = framebuffer.width;
    let screen_height = framebuffer.height;
    let num_rays = screen_width;
    let fov = player.fov;
    let half_screen_height = screen_height as f32 / 2.0;

    // Pre-calcular colores para diferentes distancias/tipos de pared
    let wall_colors = [
        rgba_to_u32(200, 200, 200, 255), // Cerca
        rgba_to_u32(150, 150, 150, 255), // Medio
        rgba_to_u32(100, 100, 100, 255), // Lejos
        rgba_to_u32(50, 50, 50, 255),    // Muy lejos
    ];

    for i in 0..num_rays {
        let ray_ratio = i as f32 / num_rays as f32;
        let ray_angle = player.a - (fov / 2.0) + (ray_ratio * fov);

        let intersection = cast_ray(
            framebuffer,
            maze,
            player,
            block_size,
            ray_angle,
            Vector2::new(0.0, 0.0),
            false,
        );

        let corrected_distance = intersection.distance * (player.a - ray_angle).cos();

        let wall_height = if corrected_distance > 0.1 {
            (block_size as f32 * screen_height as f32) / corrected_distance
        } else {
            screen_height as f32
        };

        let start_y = (half_screen_height - wall_height / 2.0).max(0.0) as u32;
        let end_y = (half_screen_height + wall_height / 2.0).min(screen_height as f32) as u32;

        // Seleccionar color basado en distancia para efecto de profundidad
        let color_index = if corrected_distance < 100.0 { 0 }
                         else if corrected_distance < 200.0 { 1 }
                         else if corrected_distance < 400.0 { 2 }
                         else { 3 };
        
        let wall_color = wall_colors[color_index];

        // Usar draw_vertical_line optimizada para mejor rendimiento
        if start_y < end_y {
            framebuffer.draw_vertical_line(i as u32, start_y, end_y - 1, wall_color);
        }
    }
}