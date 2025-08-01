use raylib::prelude::*;
use std::f32::consts::PI;
use crate::player::Player;
use crate::maze::Maze;

pub fn process_events(
    window: &RaylibHandle,
    player: &mut Player,
    maze: &Maze,
    block_size: usize
) {
    const MOVE_SPEED: f32 = 5.0;
    const ROTATION_SPEED: f32 = PI / 30.0;

    // Rotar a la izquierda
    if window.is_key_down(KeyboardKey::KEY_LEFT) || window.is_key_down(KeyboardKey::KEY_A) {
        player.a -= ROTATION_SPEED;
    }

    // Rotar a la derecha
    if window.is_key_down(KeyboardKey::KEY_RIGHT) || window.is_key_down(KeyboardKey::KEY_D) {
        player.a += ROTATION_SPEED;
    }

    // Movimiento hacia adelante
    if window.is_key_down(KeyboardKey::KEY_UP) || window.is_key_down(KeyboardKey::KEY_W) {
        let next_x = player.pos.x + MOVE_SPEED * player.a.cos();
        let next_y = player.pos.y + MOVE_SPEED * player.a.sin();

        if can_move_to(next_x, next_y, maze, block_size) {
            player.pos.x = next_x;
            player.pos.y = next_y;
        }
    }

    // Movimiento hacia atrás
    if window.is_key_down(KeyboardKey::KEY_DOWN) || window.is_key_down(KeyboardKey::KEY_S) {
        let next_x = player.pos.x - MOVE_SPEED * player.a.cos();
        let next_y = player.pos.y - MOVE_SPEED * player.a.sin();

        if can_move_to(next_x, next_y, maze, block_size) {
            player.pos.x = next_x;
            player.pos.y = next_y;
        }
    }
}

/// Verifica si el jugador puede moverse a esa posición
fn can_move_to(x: f32, y: f32, maze: &Maze, block_size: usize) -> bool {
    let col = x as usize / block_size;
    let row = y as usize / block_size;

    if row >= maze.len() || col >= maze[0].len() {
        return false; // fuera de los límites del laberinto
    }

    maze[row][col] == ' ' // solo se puede mover si es un espacio vacío
}
