use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pixel_buffer: Vec<u32>, // RGBA como u32
    texture: Option<Texture2D>, // Textura reutilizable
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let size = (width * height) as usize;
        let bg_rgba = color_to_u32(background_color);
        
        Framebuffer {
            width,
            height,
            pixel_buffer: vec![bg_rgba; size],
            texture: None,
            background_color,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        let bg_rgba = color_to_u32(self.background_color);
        self.pixel_buffer.fill(bg_rgba);
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.pixel_buffer[index] = color_to_u32(self.current_color);
        }
    }

    // Versión más rápida con color directo
    #[inline]
    pub fn set_pixel_fast(&mut self, x: u32, y: u32, color_rgba: u32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.pixel_buffer[index] = color_rgba;
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    // Función para crear/actualizar la textura solo cuando sea necesario
    fn ensure_texture(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        if self.texture.is_none() {
            // Crear textura vacía
            let img = Image::gen_image_color(self.width as i32, self.height as i32, Color::WHITE);
            self.texture = Some(rl.load_texture_from_image(thread, &img)
                .expect("No se pudo crear textura"));
        }
    }

    pub fn swap_buffers<F: FnOnce(&mut RaylibDrawHandle)>(
        &mut self, // Cambiado a &mut para poder modificar la textura
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        draw_overlay: F
    ) {
        // Asegurar que tenemos una textura
        self.ensure_texture(window, raylib_thread);
        
        if let Some(ref mut texture) = self.texture {
            // Actualizar los datos de la textura existente
            unsafe {
                // Convertir nuestro buffer a formato que raylib entienda
                let raw_data = std::slice::from_raw_parts(
                    self.pixel_buffer.as_ptr() as *const u8,
                    self.pixel_buffer.len() * 4
                );
                
                // Actualizar textura existente (mucho más rápido que crear nueva)
                raylib::ffi::UpdateTexture(*texture.as_ref(), raw_data.as_ptr() as *const _);
            }

            let mut d = window.begin_drawing(raylib_thread);
            d.clear_background(self.background_color);
            d.draw_texture(texture, 0, 0, Color::WHITE);
            draw_overlay(&mut d);
        }
    }

    // Algoritmo de línea optimizado (Bresenham)
    pub fn draw_line(&mut self, from: Vector2, to: Vector2, color: Color) {
        let color_rgba = color_to_u32(color);
        self.draw_line_fast(from, to, color_rgba);
    }

    pub fn draw_line_fast(&mut self, from: Vector2, to: Vector2, color_rgba: u32) {
        let mut x0 = from.x as i32;
        let mut y0 = from.y as i32;
        let x1 = to.x as i32;
        let y1 = to.y as i32;

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x0 >= 0 && y0 >= 0 && x0 < self.width as i32 && y0 < self.height as i32 {
                self.set_pixel_fast(x0 as u32, y0 as u32, color_rgba);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    // Función para dibujar columnas verticales (común en raycasting)
    pub fn draw_vertical_line(&mut self, x: u32, y_start: u32, y_end: u32, color_rgba: u32) {
        if x >= self.width {
            return;
        }
        
        let start = y_start.min(y_end).min(self.height - 1);
        let end = y_start.max(y_end).min(self.height - 1);
        
        for y in start..=end {
            let index = (y * self.width + x) as usize;
            self.pixel_buffer[index] = color_rgba;
        }
    }
}

// Función helper para convertir Color a u32 RGBA
#[inline]
fn color_to_u32(color: Color) -> u32 {
    ((color.a as u32) << 24) | 
    ((color.b as u32) << 16) | 
    ((color.g as u32) << 8) | 
    (color.r as u32)
}

// Función helper para crear colores u32 directamente
#[inline]
pub fn rgba_to_u32(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}