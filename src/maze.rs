use std::fs::File;
use std::io::{BufRead, BufReader};
use raylib::prelude::*;

use crate::framebuffer::Framebuffer;
use crate::player::Player;
use crate::cast::cast_ray;

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> Maze {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader.lines().map(|line| line.unwrap().chars().collect()).collect()
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    pos: Vector2,
    with_rays: bool,
){
    for (row_index, row) in maze.iter().enumerate(){
        for (col_index, &cell) in row.iter().enumerate(){
            let xo = (col_index * block_size)+pos.x as usize;
            let yo = (row_index * block_size)+pos.y as usize;
            draw_cell(framebuffer, xo, yo, block_size, cell);
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

    let scale = block_size as f32 / 65.0; //Cambiar si la resolución y el maze cambia
    let scaled_pos = Vector2::new(player.pos.x * scale, player.pos.y * scale);
    let triangle_size = block_size as f32 * 0.5;

    let fake_player = Player {
        pos: scaled_pos,
        a: player.a,
        fov: player.fov,
    };

    fake_player.draw_player_triangle(framebuffer, pos, triangle_size, Color::WHITESMOKE);


}


fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char
){
    match cell {
        ' ' => framebuffer.set_current_color(Color::BLACK),
        _ => framebuffer.set_current_color(Color::RED),
    }
    for x in xo..(xo+block_size){
        for y in yo..(yo+block_size){
            framebuffer.set_pixel(x as u32, y as u32);
        }
    }
}