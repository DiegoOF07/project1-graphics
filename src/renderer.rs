use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::maze::Maze;
use crate::cast::cast_ray;

use raylib::prelude::*;
use std::f32::consts::PI;

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
            false, // no dibujar línea en el minimapa
        );

        let corrected_distance = intersection.distance * (player.a - ray_angle).cos();

        let wall_height = if corrected_distance > 0.1 {
            (block_size as f32 * screen_height as f32) / corrected_distance
        } else {
            screen_height as f32
        };

        let start_y = (half_screen_height - wall_height / 2.0).max(0.0);
        let end_y = (half_screen_height + wall_height / 2.0).min(screen_height as f32);

        framebuffer.set_current_color(Color::RAYWHITE); // puedes cambiar color según distancia

        for y in start_y as u32..end_y as u32 {
            framebuffer.set_pixel(i as u32, y);
        }
    }
}
