use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::texture::TextureManager;
use std::f32::consts::PI;
use std::time::Instant;

/// Representa un sprite en el mundo
#[derive(Clone)]
pub struct Sprite {
    pub pos: Vector2,
    pub texture_name: String,
    pub scale: f32,
    pub damaging: bool,
    pub animation: Option<AnimatedSprite>, // Opcional para sprites estáticos
}

#[derive(Clone)]
pub struct AnimatedSprite {
    pub frames: Vec<String>,          // nombres de las texturas para cada frame
    pub frame_duration: f32,          // duración de cada frame en segundos
    pub current_frame: usize,         // frame actual
    pub last_update: Instant,         // último momento de actualización
}

impl Sprite {
    pub fn new_static(pos: Vector2, texture_name: String, scale: f32, damaging: bool) -> Self {
        Sprite {
            pos,
            texture_name,
            scale,
            damaging,
            animation: None,
        }
    }

    pub fn new_animated(pos: Vector2, frames: Vec<String>, frame_duration: f32, scale: f32, damaging: bool) -> Self {
        Sprite {
            pos,
            texture_name: frames[0].clone(), // Usar el primer frame como textura inicial
            scale,
            damaging,
            animation: Some(AnimatedSprite {
                frames,
                frame_duration,
                current_frame: 0,
                last_update: Instant::now(),
            }),
        }
    }

    pub fn update(&mut self) {
        if let Some(anim) = &mut self.animation {
            let now = Instant::now();
            let elapsed = now.duration_since(anim.last_update).as_secs_f32();
            
            if elapsed >= anim.frame_duration {
                anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
                self.texture_name = anim.frames[anim.current_frame].clone();
                anim.last_update = now;
            }
        }
    }
}

pub fn render_sprites(
    framebuffer: &mut Framebuffer,
    sprites: &mut Vec<Sprite>,
    player: &Player,
    texture_manager: &TextureManager,
    depth_buffer: &Vec<f32>,
) {
    let screen_width = framebuffer.width as f32;
    let screen_height = framebuffer.height as f32;
    let half_screen_height = screen_height / 2.0;

    for sprite in sprites.iter_mut() {
        sprite.update();
    }

    // 1. Ordenar sprites de más lejos a más cerca
    let mut sorted_sprites = sprites.clone();
    sorted_sprites.sort_by(|a, b| {
        let dist_a = (a.pos.x - player.pos.x).powi(2) + (a.pos.y - player.pos.y).powi(2);
        let dist_b = (b.pos.x - player.pos.x).powi(2) + (b.pos.y - player.pos.y).powi(2);
        dist_b.partial_cmp(&dist_a).unwrap()
    });

    // 2. Dibujar sprites uno por uno
    for sprite in sorted_sprites {
        // Calcular posición relativa al jugador
        let dx = sprite.pos.x - player.pos.x;
        let dy = sprite.pos.y - player.pos.y;

        // Transformar a espacio del jugador
        let sprite_angle = dy.atan2(dx);
        let mut angle_diff = sprite_angle - player.a;

        // Normalizar ángulo
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }

        // Si el sprite está fuera del FOV, skipear
        if angle_diff.abs() > player.fov / 2.0 {
            continue;
        }

        // Calcular distancia perpendicular (corregir fish-eye)
        let distance = (dx * dx + dy * dy).sqrt() * (angle_diff).cos();
        
        // Calcular tamaño en pantalla
        let sprite_height = (screen_height / distance) * sprite.scale;
        let sprite_width = sprite_height; // mantener proporción 1:1

        // Calcular posición en pantalla
        let sprite_screen_x = (screen_width / 2.0 * (1.0 + angle_diff / (player.fov / 2.0))) - sprite_width / 2.0;
        let sprite_screen_y = half_screen_height;

        // Obtener textura
        if let Some(texture) = texture_manager.get_sprite_texture(&sprite.texture_name) {
            // Dibujar sprite
            for x in 0..sprite_width as i32 {
                let screen_x = sprite_screen_x as i32 + x;
                if screen_x < 0 || screen_x >= screen_width as i32 {
                    continue;
                }

                // Verificar depth buffer
                if distance >= depth_buffer[screen_x as usize] {
                    continue;
                }

                let tex_x = x as f32 / sprite_width;
                
                for y in 0..sprite_height as i32 {
                    let screen_y = sprite_screen_y as i32 + y;
                    if screen_y < 0 || screen_y >= screen_height as i32 {
                        continue;
                    }

                    let tex_y = y as f32 / sprite_height;
                    let color = sample_sprite_texture(texture, tex_x, tex_y);
                    
                    // Solo dibujar si el pixel no es transparente
                    if (color >> 24) > 10 {
                        framebuffer.set_pixel_fast(screen_x as u32, screen_y as u32, color);
                    }
                }
            }
        }
    }
}

/// Función auxiliar para muestrear textura de sprite (consistente con TextureManager)
fn sample_sprite_texture(texture: &crate::texture::TextureData, u: f32, v: f32) -> u32 {
    // Asegurar que las coordenadas estén en rango [0,1)
    let u = u.clamp(0.0, 1.0 - f32::EPSILON);
    let v = v.clamp(0.0, 1.0 - f32::EPSILON);

    // Convertir a coordenadas de píxel
    let x = ((u * texture.width as f32) as u32).min(texture.width - 1);
    let y = ((v * texture.height as f32) as u32).min(texture.height - 1);

    // Obtener píxel
    let index = (y * texture.width + x) as usize;
    texture.pixels.get(index).copied().unwrap_or(0xFF00FF00) // Verde magenta si error
}