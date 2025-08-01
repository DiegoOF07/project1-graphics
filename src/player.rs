use raylib::prelude::*;
use std::f32::consts::PI;
use crate::framebuffer::Framebuffer;


pub struct Player{
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32
}

impl Player {
    pub fn new(pos: Vector2) -> Self{
        Player { 
            pos,
            a: PI / 3.0,
            fov: PI / 3.0
        }
    }

    pub fn draw_player_triangle(
        &self,
        framebuffer: &mut Framebuffer,
        pos: Vector2,
        size: f32,
        color: Color
    ) {
        let center_x = &self.pos.x + pos.x;
        let center_y = &self.pos.y + pos.y;

        let a = &self.a;
        let left = a + 5.0 * PI / 6.0;
        let right = a - 5.0 * PI / 6.0;

        let tip = Vector2::new(center_x + size * a.cos(), center_y + size * a.sin());
        let left_pt = Vector2::new(center_x + size * 0.6 * left.cos(), center_y + size * 0.6 * left.sin());
        let right_pt = Vector2::new(center_x + size * 0.6 * right.cos(), center_y + size * 0.6 * right.sin());

        framebuffer.draw_line(tip, left_pt, color);
        framebuffer.draw_line(left_pt, right_pt, color);
        framebuffer.draw_line(right_pt, tip, color);
    }
}


