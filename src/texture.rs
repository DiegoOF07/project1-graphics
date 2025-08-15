use raylib::prelude::*;
use crate::framebuffer::{Framebuffer, rgba_to_u32};
use std::collections::HashMap;
use image;

pub struct TextureManager {
    // Almacenar texturas como datos de píxeles para acceso rápido
    wall_textures: HashMap<char, TextureData>,
    floor_texture: Option<TextureData>,
    ceiling_texture: Option<TextureData>,
}

#[derive(Clone)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>, // RGBA como u32 para acceso rápido
}

impl TextureManager {
    pub fn new() -> Self {
        TextureManager {
            wall_textures: HashMap::new(),
            floor_texture: None,
            ceiling_texture: None,
        }
    }

    /// Cargar textura de pared para un carácter específico del maze
    pub fn load_wall_texture(&mut self, wall_char: char, filename: &str, _rl: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), String> {
        match self.load_texture_data(filename) {
            Ok(texture_data) => {
                self.wall_textures.insert(wall_char, texture_data);
                Ok(())
            }
            Err(e) => Err(format!("Error cargando textura de pared '{}': {}", filename, e))
        }
    }

    /// Cargar textura de suelo
    pub fn load_floor_texture(&mut self, filename: &str, _rl: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), String> {
        match self.load_texture_data(filename) {
            Ok(texture_data) => {
                self.floor_texture = Some(texture_data);
                Ok(())
            }
            Err(e) => Err(format!("Error cargando textura de suelo '{}': {}", filename, e))
        }
    }

    /// Cargar textura de techo/cielo
    pub fn load_ceiling_texture(&mut self, filename: &str, _rl: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), String> {
        match self.load_texture_data(filename) {
            Ok(texture_data) => {
                self.ceiling_texture = Some(texture_data);
                Ok(())
            }
            Err(e) => Err(format!("Error cargando textura de techo '{}': {}", filename, e))
        }
    }

    /// Función interna para cargar datos de textura desde archivo
    fn load_texture_data(&self, filename: &str) -> Result<TextureData, String> {
        use std::path::Path;
        
        // Cargar imagen usando la crate 'image'
        let img = image::open(Path::new(filename))
            .map_err(|e| format!("Error abriendo imagen {}: {}", filename, e))?;
        
        // Convertir a RGBA8
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        
        // Verificar dimensiones válidas
        if width == 0 || height == 0 {
            return Err(format!("Imagen inválida: {} (dimensiones: {}x{})", filename, width, height));
        }
        
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        // Extraer píxeles
        for y in 0..height {
            for x in 0..width {
                let pixel = rgba_img.get_pixel(x, y);
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = pixel[3];
                pixels.push(rgba_to_u32(r, g, b, a));
            }
        }

        println!("Textura cargada exitosamente: {} ({}x{})", filename, width, height);
        Ok(TextureData { width, height, pixels })
    }

    /// Obtener color de textura de pared en coordenadas específicas
    pub fn get_wall_color(&self, wall_char: char, texture_x: f32, texture_y: f32) -> u32 {
        if let Some(texture) = self.wall_textures.get(&wall_char) {
            self.sample_texture(texture, texture_x, texture_y)
        } else {
            // Color por defecto si no hay textura
            match wall_char {
                '+' => rgba_to_u32(139, 69, 19, 255),   // Marrón (madera)
                '#' => rgba_to_u32(128, 128, 128, 255), // Gris (piedra)
                '=' => rgba_to_u32(160, 82, 45, 255),   // Marrón claro (ladrillo)
                '-' => rgba_to_u32(180, 82, 45, 255),   // Marrón claro (pared horizontal)
                '|' => rgba_to_u32(160, 92, 55, 255),   // Marrón medio (pared vertical)
                _ => rgba_to_u32(255, 0, 0, 255),       // Rojo por defecto
            }
        }
    }

    /// Obtener color de suelo en coordenadas específicas
    pub fn get_floor_color(&self, world_x: f32, world_y: f32) -> u32 {
        if let Some(texture) = &self.floor_texture {
            // Mapear coordenadas del mundo a coordenadas de textura
            let texture_x = (world_x / 30.0) % 1.0; // Repetir cada 30 unidades (block_size)
            let texture_y = (world_y / 30.0) % 1.0;
            self.sample_texture(texture, texture_x, texture_y)
        } else {
            rgba_to_u32(64, 64, 64, 255) // Gris oscuro por defecto
        }
    }

    /// Obtener color de techo/cielo
    pub fn get_ceiling_color(&self, screen_x: f32, screen_y: f32, screen_width: f32, screen_height: f32) -> u32 {
        if let Some(texture) = &self.ceiling_texture {
            // Para el cielo, podemos usar coordenadas de pantalla directamente
            let texture_x = (screen_x / screen_width) % 1.0;
            let texture_y = (screen_y / screen_height) % 1.0;
            self.sample_texture(texture, texture_x, texture_y)
        } else {
            // Gradiente nocturno: azul oscuro arriba, morado/gris abajo
            let progress = screen_y / screen_height;
            let r = (20.0 + 30.0 * (1.0 - progress)) as u8; // Rojo tenue
            let g = (20.0 + 25.0 * (1.0 - progress)) as u8; // Verde tenue
            let b = (50.0 + 80.0 * (1.0 - progress)) as u8; // Azul profundo
            rgba_to_u32(r, g, b, 255)
        }
    }

    /// Función interna para muestrear una textura en coordenadas UV (0.0-1.0)
    fn sample_texture(&self, texture: &TextureData, u: f32, v: f32) -> u32 {
        // Asegurar que las coordenadas estén en rango [0,1)
        let u = u.fract().abs();
        let v = v.fract().abs();

        // Convertir a coordenadas de píxel
        let x = ((u * texture.width as f32) as u32).min(texture.width - 1);
        let y = ((v * texture.height as f32) as u32).min(texture.height - 1);

        // Obtener píxel
        let index = (y * texture.width + x) as usize;
        texture.pixels.get(index).copied().unwrap_or(rgba_to_u32(255, 0, 255, 255)) // Magenta si error
    }

    /// Generar texturas procedurales si no se cargan archivos
    pub fn generate_default_textures(&mut self) {
        // Textura de ladrillo para '+'
        let brick_texture = self.generate_brick_texture(64, 64);
        self.wall_textures.insert('+', brick_texture);

        // Textura de piedra para '#'
        let stone_texture = self.generate_stone_texture(64, 64);
        self.wall_textures.insert('#', stone_texture);

        // Texturas para paredes horizontales y verticales
        let wood_h_texture = self.generate_wood_texture(64, 64, true);
        self.wall_textures.insert('-', wood_h_texture);
        
        let wood_v_texture = self.generate_wood_texture(64, 64, false);
        self.wall_textures.insert('|', wood_v_texture);

        // Textura de suelo
        self.floor_texture = Some(self.generate_floor_texture(64, 64));

        // Textura de cielo
        self.ceiling_texture = Some(self.generate_sky_texture(256, 128));
    }

    /// Generar textura procedural de ladrillo
    fn generate_brick_texture(&self, width: u32, height: u32) -> TextureData {
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        for y in 0..height {
            for x in 0..width {
                let brick_w = width / 8;
                let brick_h = height / 4;
                
                let brick_x = x / brick_w;
                let brick_y = y / brick_h;
                
                // Alternar ladrillos en filas
                let offset = if brick_y % 2 == 0 { 0 } else { brick_w / 2 };
                let local_x = (x + offset) % brick_w;
                let local_y = y % brick_h;
                
                // Bordes del ladrillo
                let is_border = local_x < 2 || local_x >= brick_w - 2 || local_y < 2 || local_y >= brick_h - 2;
                
                let color = if is_border {
                    rgba_to_u32(101, 67, 33, 255) // Marrón oscuro (mortero)
                } else {
                    rgba_to_u32(139, 69, 19, 255) // Marrón (ladrillo)
                };
                
                pixels.push(color);
            }
        }
        
        TextureData { width, height, pixels }
    }

    /// Generar textura procedural de piedra
    fn generate_stone_texture(&self, width: u32, height: u32) -> TextureData {
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        for y in 0..height {
            for x in 0..width {
                // Patrón de ruido simple para piedra
                let noise = ((x * 17 + y * 23) % 64) as f32 / 64.0;
                let base_gray = 128.0 + noise * 64.0;
                
                let gray = base_gray as u8;
                pixels.push(rgba_to_u32(gray, gray, gray, 255));
            }
        }
        
        TextureData { width, height, pixels }
    }

    /// Generar textura procedural de madera
    fn generate_wood_texture(&self, width: u32, height: u32, horizontal: bool) -> TextureData {
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        for y in 0..height {
            for x in 0..width {
                // Determinar la coordenada principal para las vetas
                let main_coord = if horizontal { x } else { y };
                let cross_coord = if horizontal { y } else { x };
                
                // Patrón de vetas de madera
                let grain_period = if horizontal { width / 8 } else { height / 8 };
                let grain = ((main_coord % grain_period) as f32 / grain_period as f32).sin();
                
                // Variación cruzada sutil
                let cross_var = ((cross_coord as f32 * 0.1).sin() * 0.1).abs();
                
                // Color base marrón
                let base_brown = 139.0;
                let variation = grain * 40.0 + cross_var * 20.0;
                
                let r = (base_brown + variation).max(100.0).min(180.0) as u8;
                let g = ((base_brown + variation) * 0.5).max(50.0).min(120.0) as u8;
                let b = ((base_brown + variation) * 0.2).max(10.0).min(60.0) as u8;
                
                pixels.push(rgba_to_u32(r, g, b, 255));
            }
        }
        
        TextureData { width, height, pixels }
    }

    /// Generar textura procedural de suelo
    fn generate_floor_texture(&self, width: u32, height: u32) -> TextureData {
        let mut pixels = Vec::with_capacity((width * height) as usize);
        
        for y in 0..height {
            for x in 0..width {
                // Patrón de baldosas
                let tile_size = width / 4;
                let tile_x = x / tile_size;
                let tile_y = y / tile_size;
                
                let is_dark_tile = (tile_x + tile_y) % 2 == 0;
                
                // Añadir variación dentro de cada baldosa
                let local_x = x % tile_size;
                let local_y = y % tile_size;
                let border_width = 2;
                let is_border = local_x < border_width || local_x >= tile_size - border_width ||
                               local_y < border_width || local_y >= tile_size - border_width;
                
                let base_color = if is_dark_tile { 40 } else { 80 };
                let final_color = if is_border { base_color - 10 } else { base_color };
                
                pixels.push(rgba_to_u32(final_color, final_color, final_color, 255));
            }
        }
        
        TextureData { width, height, pixels }
    }

    fn generate_sky_texture(&self, width: u32, height: u32) -> TextureData {
        let mut pixels = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let progress = y as f32 / height as f32;

                // Zona superior: azul muy oscuro con toque morado
                let (r, g, b) = if progress < 0.3 {
                    let intensity = (progress / 0.3 * 25.0) as u8;
                    (intensity, intensity / 2, 40 + intensity)
                } else if progress < 0.7 {
                    // Transición a azul/morado medio
                    let local_progress = (progress - 0.3) / 0.4;
                    let blue_intensity = (60.0 + local_progress * 40.0) as u8;
                    let purple_fade = (40.0 * (1.0 - local_progress)) as u8;
                    (purple_fade, purple_fade / 2, blue_intensity)
                } else {
                    // Horizonte: azul claro con toque gris/morado
                    let local_progress = (progress - 0.7) / 0.3;
                    let blue_intensity = (100.0 + local_progress * 40.0) as u8;
                    let gray_tint = (local_progress * 30.0) as u8;
                    (gray_tint, gray_tint, blue_intensity)
                };

                let final_r = r as u8;
                let final_g = g as u8;
                let final_b = b as u8;

                pixels.push(rgba_to_u32(final_r, final_g, final_b, 255));
            }
        }

        TextureData { width, height, pixels }
    }
}