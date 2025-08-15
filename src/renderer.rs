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
        
        let (wall_x, actual_wall_char) = determine_wall_orientation_and_texture(
            maze, 
            hit_x, 
            hit_y, 
            block_size, 
            intersection.impact,
            ray_angle
        );

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
                let wall_progress = (y - wall_start) as f32 / (wall_end - wall_start) as f32;
                let wall_color = texture_manager.get_wall_color(actual_wall_char, wall_x, wall_progress);

                // Efecto linterna
                let dx = (i as f32 / screen_width as f32) - 0.5;
                let flashlight_angle = 0.0; // Centrado
                let flashlight_width = 0.12; // Haz estrecho
                let flashlight_strength = 0.7; // Qué tan brillante es el centro

                let flashlight_factor = if dx.abs() < flashlight_width {
                    // Atenuación suave en los bordes del haz
                    flashlight_strength * (1.0 - dx.abs() / flashlight_width)
                } else {
                    0.0
                };

                let shaded_color = apply_distance_shading(wall_color, corrected_distance, flashlight_factor);
                framebuffer.set_pixel_fast(i as u32, y, shaded_color);
            }
        }

        // === RENDERIZAR SUELO === Esto causa la bajada de FPS tener en cuenta
        for y in wall_end..screen_height {
            let ray_dir_x = ray_angle.cos();
            let ray_dir_y = ray_angle.sin();

            let p = y as f32 - half_screen_height;
            let pos_z = 0.5 * screen_height as f32;
            let row_distance = pos_z / p;

            let floor_x = player.pos.x + row_distance * ray_dir_x;
            let floor_y = player.pos.y + row_distance * ray_dir_y;

            let floor_color = texture_manager.get_floor_color(floor_x, floor_y);
            let shaded_floor_color = apply_distance_shading(floor_color, row_distance, 0.0);
            framebuffer.set_pixel_fast(i as u32, y, shaded_floor_color);
        }
    }
}

// Ejemplo parcial: downscaling horizontal con ray_step
pub fn render_world_with_textures_downscale(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    texture_manager: &TextureManager,
) {
    let screen_width = framebuffer.width as usize;
    let screen_height = framebuffer.height as usize;
    let ray_step: usize = 2; // <-- configurable: 1 = full, 2 = mitad de rayos
    let virtual_rays = (screen_width + ray_step - 1) / ray_step;
    let fov = player.fov;
    let half_screen_height = screen_height as f32 / 2.0;

    // Para cada rayo "virtual"
    for vr in 0..virtual_rays {
        // coordenada x izquierda del bloque en pantalla
        let block_x = vr * ray_step;
        // usamos el píxel central del bloque para el ángulo (reduce jitter)
        let center_x = (block_x as f32 + (ray_step as f32) / 2.0).min(screen_width as f32 - 1.0);
        let ray_ratio = center_x / screen_width as f32;
        let ray_angle = player.a - (fov / 2.0) + (ray_ratio * fov);

        // lanzar rayo (un solo cast por bloque)
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

        // altura de pared
        let wall_height = if corrected_distance > 0.1 {
            (block_size as f32 * screen_height as f32) / corrected_distance
        } else {
            screen_height as f32
        };

        let wall_start = (half_screen_height - wall_height / 2.0).max(0.0) as usize;
        let wall_end = (half_screen_height + wall_height / 2.0).min(screen_height as f32) as usize;

        // hit coords para calcular wall_x (u)
        let hit_x = player.pos.x + intersection.distance * ray_angle.cos();
        let hit_y = player.pos.y + intersection.distance * ray_angle.sin();
        let (wall_x, actual_wall_char) = determine_wall_orientation_and_texture(
            maze, hit_x, hit_y, block_size, intersection.impact, ray_angle
        );

        // RENDER TECHO para todo el bloque (replicar horizontalmente)
        for y in 0..wall_start {
            let ceiling_color = texture_manager.get_ceiling_color(
                center_x, y as f32, screen_width as f32, screen_height as f32
            );
            for dx in 0..ray_step {
                let px = block_x + dx;
                if px < screen_width { framebuffer.set_pixel_fast(px as u32, y as u32, ceiling_color); }
            }
        }

        // RENDER PARED texturada (pixel-por-pixel vertical, pero replicada horizontalmente)
        if wall_start < wall_end {
            for y in wall_start..wall_end {
                let wall_progress = (y - wall_start) as f32 / ((wall_end - wall_start) as f32).max(1.0);
                let wall_color = texture_manager.get_wall_color(actual_wall_char, wall_x, wall_progress);

                // sombreado por distancia (puedes extraer y optimizar esto)
                let dx_center = (center_x / screen_width as f32) - 0.5;
                let flashlight_width = 0.12;
                let flashlight_strength = 0.6;
                let flashlight_factor = if dx_center.abs() < flashlight_width {
                    flashlight_strength * (1.0 - dx_center.abs() / flashlight_width)
                } else { 0.0 };

                let shaded_color = apply_distance_shading(wall_color, corrected_distance, flashlight_factor);

                // replicar la columna en ray_step píxeles horizontales
                for dx in 0..ray_step {
                    let px = block_x + dx;
                    if px < screen_width {
                        framebuffer.set_pixel_fast(px as u32, y as u32, shaded_color);
                    }
                }
            }
        }

        // RENDER SUELO (similar: calculas por rayo virtual y replicar)
        for y in wall_end..screen_height {
            let ray_dir_x = ray_angle.cos();
            let ray_dir_y = ray_angle.sin();
            let p = y as f32 - half_screen_height;
            let pos_z = 0.5 * screen_height as f32;
            let row_distance = pos_z / p;

            let floor_x = player.pos.x + row_distance * ray_dir_x;
            let floor_y = player.pos.y + row_distance * ray_dir_y;

            let floor_color = texture_manager.get_floor_color(floor_x, floor_y);
            let shaded_floor_color = apply_distance_shading(floor_color, row_distance, 0.0);

            for dx in 0..ray_step {
                let px = block_x + dx;
                if px < screen_width {
                    framebuffer.set_pixel_fast(px as u32, y as u32, shaded_floor_color);
                }
            }
        }
    }
}


/// Determinar la orientación real de la pared basándose en el contexto del mapa
fn determine_wall_orientation_and_texture(
    maze: &Maze,
    hit_x: f32,
    hit_y: f32,
    block_size: usize,
    impact_char: char,
    ray_angle: f32,
) -> (f32, char) {
    // Convertir coordenadas del mundo a coordenadas del mapa
    let map_x = (hit_x / block_size as f32).floor() as i32;
    let map_y = (hit_y / block_size as f32).floor() as i32;
    
    // Si no es un '+', usar el carácter original
    if impact_char != '+' {
        let wall_x = if impact_char == '|' {
            hit_y % block_size as f32 / block_size as f32
        } else {
            hit_x % block_size as f32 / block_size as f32
        };
        return (wall_x, impact_char);
    }
    
    // Para '+', usar método mejorado de detección
    let context = analyze_wall_context(maze, map_x, map_y);
    
    // Calcular distancias a los bordes de la celda
    let cell_x = hit_x % block_size as f32;
    let cell_y = hit_y % block_size as f32;
    let half_block = block_size as f32 / 2.0;
    
    let dist_to_left = cell_x;
    let dist_to_right = block_size as f32 - cell_x;
    let dist_to_top = cell_y;
    let dist_to_bottom = block_size as f32 - cell_y;
    
    // Encontrar el borde más cercano
    let min_dist = dist_to_left.min(dist_to_right).min(dist_to_top).min(dist_to_bottom);
    
    let (wall_x, wall_type) = if min_dist == dist_to_left || min_dist == dist_to_right {
        // Golpeamos una cara vertical
        let wall_x = cell_y / block_size as f32;
        let wall_type = if context.has_vertical_preference() { '|' } else { '+' };
        (wall_x, wall_type)
    } else {
        // Golpeamos una cara horizontal
        let wall_x = cell_x / block_size as f32;
        let wall_type = if context.has_horizontal_preference() { '-' } else { '+' };
        (wall_x, wall_type)
    };
    
    (wall_x, wall_type)
}

/// Estructura para almacenar información del contexto de una pared
struct WallContext {
    has_vertical_walls: bool,
    has_horizontal_walls: bool,
    is_corner: bool,
    is_intersection: bool,
    vertical_connections: usize,
    horizontal_connections: usize,
}

impl WallContext {
    fn has_vertical_preference(&self) -> bool {
        // Preferir vertical si hay más conexiones verticales o es claramente vertical
        self.vertical_connections > self.horizontal_connections || 
        (self.has_vertical_walls && !self.has_horizontal_walls)
    }
    
    fn has_horizontal_preference(&self) -> bool {
        // Preferir horizontal si hay más conexiones horizontales o es claramente horizontal
        self.horizontal_connections > self.vertical_connections || 
        (self.has_horizontal_walls && !self.has_vertical_walls)
    }
}

/// Analizar el contexto de una pared '+' para determinar su orientación
fn analyze_wall_context(maze: &Maze, x: i32, y: i32) -> WallContext {
    let mut has_vertical_walls = false;
    let mut has_horizontal_walls = false;
    let mut vertical_connections = 0;
    let mut horizontal_connections = 0;
    
    // Verificar conexiones específicas
    let left = get_maze_cell(maze, x - 1, y);
    let right = get_maze_cell(maze, x + 1, y);
    let up = get_maze_cell(maze, x, y - 1);
    let down = get_maze_cell(maze, x, y + 1);
    
    // Contar conexiones horizontales (izquierda-derecha)
    if let Some(cell) = left {
        if matches!(cell, '|' | '-' | '+') {
            horizontal_connections += 1;
            if matches!(cell, '-' | '+') {
                has_horizontal_walls = true;
            }
        }
    }
    
    if let Some(cell) = right {
        if matches!(cell, '|' | '-' | '+') {
            horizontal_connections += 1;
            if matches!(cell, '-' | '+') {
                has_horizontal_walls = true;
            }
        }
    }
    
    // Contar conexiones verticales (arriba-abajo)
    if let Some(cell) = up {
        if matches!(cell, '|' | '-' | '+') {
            vertical_connections += 1;
            if matches!(cell, '|' | '+') {
                has_vertical_walls = true;
            }
        }
    }
    
    if let Some(cell) = down {
        if matches!(cell, '|' | '-' | '+') {
            vertical_connections += 1;
            if matches!(cell, '|' | '+') {
                has_vertical_walls = true;
            }
        }
    }
    
    // Determinar tipo de conexión
    let total_connections = vertical_connections + horizontal_connections;
    let is_corner = total_connections <= 2;
    let is_intersection = total_connections >= 3;
    
    WallContext {
        has_vertical_walls,
        has_horizontal_walls,
        is_corner,
        is_intersection,
        vertical_connections,
        horizontal_connections,
    }
}

/// Obtener celda del mapa de manera segura
fn get_maze_cell(maze: &Maze, x: i32, y: i32) -> Option<char> {
    if x >= 0 && y >= 0 && y < maze.len() as i32 && x < maze[0].len() as i32 {
        Some(maze[y as usize][x as usize])
    } else {
        None
    }
}

/// Contar conexiones de pared para una celda
fn count_wall_connections(maze: &Maze, x: i32, y: i32) -> usize {
    let neighbors = [
        (x - 1, y),     // Izquierda
        (x + 1, y),     // Derecha
        (x, y - 1),     // Arriba
        (x, y + 1),     // Abajo
    ];
    
    neighbors.iter()
        .filter(|(nx, ny)| {
            if let Some(cell) = get_maze_cell(maze, *nx, *ny) {
                matches!(cell, '|' | '-' | '+')
            } else {
                false
            }
        })
        .count()
}

/// Aplicar sombreado basado en distancia
fn apply_distance_shading(color: u32, distance: f32, flashlight_factor: f32) -> u32 {
    let r = (color & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = ((color >> 16) & 0xFF) as u8;
    let a = ((color >> 24) & 0xFF) as u8;

    // Filtro oscuro global
    let global_darkness = 0.1; // Ajusta este valor para más/menos oscuridad

    // Sombreado por distancia
    let shade_factor = (1.0 - (distance / 500.0).min(0.7)).max(0.3) * global_darkness;

    // Efecto linterna (flashlight)
    let final_factor = (shade_factor + flashlight_factor).min(1.0);

    let shaded_r = (r as f32 * final_factor) as u8;
    let shaded_g = (g as f32 * final_factor) as u8;
    let shaded_b = (b as f32 * final_factor) as u8;

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