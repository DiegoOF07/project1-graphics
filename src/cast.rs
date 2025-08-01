use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
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
    framebuffer.set_current_color(Color::new(212, 255, 247, 100));

    while d < max_distance {
        let dx = d * a.cos();
        let dy = d * a.sin();

        let x = player.pos.x + dx + init_pos.x;
        let y = player.pos.y + dy + init_pos.y;

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
            framebuffer.set_pixel(x as u32, y as u32);
        }
        d += 0.1;
        
    }
    Intersect { distance: max_distance, impact: ' ' }
}


