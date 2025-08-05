use crate::framebuffer::{Framebuffer, rgba_to_u32};
use crate::player::Player;
use crate::game::Maze;
use crate::cast::{cast_ray, Intersect};
use crate::texture::TextureManager;

use raylib::prelude::*;

pub fn render_world_with_textures(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    texture_manager: &TextureManager,
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
            false,
        );

        let corrected_distance = intersection.distance * (player.a - ray_angle).cos();

        // Calcular alturas de pared, techo y suelo
        let wall_height = if corrected_distance > 0.1 {
            (block_size as f32 * screen_height as f32) / corrected_distance
        } else {
            screen_height as f32
        };

        let wall_start = (half_screen_height - wall_height / 2.0).max(0.0) as u32;
        let wall_end = (half_screen_height + wall_height / 2.0).min(screen_height as f32) as u32;

        // Calcular coordenada de textura horizontal (qué parte de la pared estamos viendo)
        let hit_x = player.pos.x + intersection.distance * ray_angle.cos();
        let hit_y = player.pos.y + intersection.distance * ray_angle.sin();
        
        // Determinar si golpeamos una pared vertical u horizontal
        let wall_x = if (hit_x % block_size as f32) < (hit_y % block_size as f32) {
            hit_y % block_size as f32 / block_size as f32
        } else {
            hit_x % block_size as f32 / block_size as f32
        };

        // === RENDERIZAR TECHO ===
        for y in 0..wall_start {
            let ceiling_color = texture_manager.get_ceiling_color(
                i as f32, 
                y as f32, 
                screen_width as f32, 
                screen_height as f32
            );
            framebuffer.set_pixel_fast(i as u32, y, ceiling_color);
        }

        // === RENDERIZAR PARED CON TEXTURA ===
        if wall_start < wall_end {
            for y in wall_start..wall_end {
                // Calcular coordenada vertical de textura
                let wall_progress = (y - wall_start) as f32 / (wall_end - wall_start) as f32;
                
                let wall_color = texture_manager.get_wall_color(
                    intersection.impact, 
                    wall_x, 
                    wall_progress
                );
                
                // Aplicar sombreado basado en distancia
                let shaded_color = apply_distance_shading(wall_color, corrected_distance);
                framebuffer.set_pixel_fast(i as u32, y, shaded_color);
            }
        }

        // === RENDERIZAR SUELO ===
        for y in wall_end..screen_height {
            // Calcular la distancia del suelo usando ray casting hacia abajo
            let distance_to_floor = (half_screen_height * corrected_distance) / (y as f32 - half_screen_height);
            
            if distance_to_floor > 0.0 && distance_to_floor < 1000.0 {
                let floor_x = player.pos.x + distance_to_floor * ray_angle.cos();
                let floor_y = player.pos.y + distance_to_floor * ray_angle.sin();
                
                let floor_color = texture_manager.get_floor_color(floor_x, floor_y);
                let shaded_floor_color = apply_distance_shading(floor_color, distance_to_floor);
                framebuffer.set_pixel_fast(i as u32, y, shaded_floor_color);
            }
        }
    }
}

/// Aplicar sombreado basado en distancia
fn apply_distance_shading(color: u32, distance: f32) -> u32 {
    // Extraer componentes RGBA
    let r = (color & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = ((color >> 16) & 0xFF) as u8;
    let a = ((color >> 24) & 0xFF) as u8;

    // Calcular factor de sombreado (más oscuro con distancia)
    let shade_factor = (1.0 - (distance / 500.0).min(0.7)).max(0.3);
    
    let shaded_r = (r as f32 * shade_factor) as u8;
    let shaded_g = (g as f32 * shade_factor) as u8;
    let shaded_b = (b as f32 * shade_factor) as u8;

    rgba_to_u32(shaded_r, shaded_g, shaded_b, a)
}

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