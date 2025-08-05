use raylib::prelude::*;
use std::f32::consts::PI;
use crate::player::Player;
use crate::game::Maze;

pub fn process_events(
    window: &mut RaylibHandle,
    player: &mut Player,
    maze: &Maze,
    last_mouse_x: &mut i32,
    block_size: usize
) {
    // Constantes de movimiento
    const MOVE_SPEED: f32 = 3.5;
    const KEYBOARD_ROTATION_SPEED: f32 = PI / 60.0;
    const MOUSE_SENSITIVITY: f32 = 0.002;
    const MAX_ROTATION_PER_FRAME: f32 = PI / 20.0;

    // === ROTACIÓN CON MOUSE ===
    let current_mouse_x = window.get_mouse_x();
    let screen_width = window.get_screen_width();
    let screen_center_x = screen_width / 2;
    
    // Inicializar last_mouse_x si es la primera vez
    if *last_mouse_x == 0 {
        *last_mouse_x = current_mouse_x;
        return;
    }
    
    // Calcular delta 
    let raw_delta = current_mouse_x - *last_mouse_x;
    let mut clamped_rotation = 0.0;

    // Si hubo un movimiento válido del mouse
    if raw_delta.abs() > 2 && raw_delta.abs() < 200 {
        let rotation_amount = raw_delta as f32 * MOUSE_SENSITIVITY;
        clamped_rotation = rotation_amount.clamp(-MAX_ROTATION_PER_FRAME, MAX_ROTATION_PER_FRAME);
    } else {
        // Detectar si estamos en una zona cercana al borde
        let edge_threshold = 60; // píxeles
        if current_mouse_x <= edge_threshold {
            clamped_rotation = -MAX_ROTATION_PER_FRAME / 2.0; // rotar a la izquierda lentamente
        } else if current_mouse_x >= screen_width - edge_threshold {
            clamped_rotation = MAX_ROTATION_PER_FRAME / 2.0; // rotar a la derecha lentamente
        }
    }

    player.a += clamped_rotation;
    
    // Actualizar last_mouse_x
    *last_mouse_x = current_mouse_x;

    // === ROTACIÓN CON TECLADO (más lenta) ===
    if window.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= KEYBOARD_ROTATION_SPEED;
    }
    if window.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += KEYBOARD_ROTATION_SPEED;
    }

    let mut move_x = 0.0;
    let mut move_y = 0.0;

    // Movimiento hacia adelante/atrás
    if window.is_key_down(KeyboardKey::KEY_UP) || window.is_key_down(KeyboardKey::KEY_W) {
        move_x += MOVE_SPEED * player.a.cos();
        move_y += MOVE_SPEED * player.a.sin();
    }
    if window.is_key_down(KeyboardKey::KEY_DOWN) || window.is_key_down(KeyboardKey::KEY_S) {
        move_x -= MOVE_SPEED * player.a.cos();
        move_y -= MOVE_SPEED * player.a.sin();
    }

    // Strafe (movimiento lateral)
    if window.is_key_down(KeyboardKey::KEY_Q) {
        // Strafe izquierda (perpendicular a la dirección de vista)
        move_x += MOVE_SPEED * (player.a - PI/2.0).cos();
        move_y += MOVE_SPEED * (player.a - PI/2.0).sin();
    }
    if window.is_key_down(KeyboardKey::KEY_E) {
        // Strafe derecha
        move_x += MOVE_SPEED * (player.a + PI/2.0).cos();
        move_y += MOVE_SPEED * (player.a + PI/2.0).sin();
    }

    // Mantener A y D solo para strafe
    if window.is_key_down(KeyboardKey::KEY_A) {
        // Strafe izquierda
        move_x += MOVE_SPEED * (player.a - PI/2.0).cos();
        move_y += MOVE_SPEED * (player.a - PI/2.0).sin();
    }
    if window.is_key_down(KeyboardKey::KEY_D) {
        // Strafe derecha
        move_x += MOVE_SPEED * (player.a + PI/2.0).cos();
        move_y += MOVE_SPEED * (player.a + PI/2.0).sin();
    }

    // === SISTEMA DE COLISIÓN ===
    // Verificar movimiento en X e Y por separado para mejor control
    let new_x = player.pos.x + move_x;
    let new_y = player.pos.y + move_y;

    // Intentar mover en X
    if can_move_to(new_x, player.pos.y, maze, block_size) {
        player.pos.x = new_x;
    }
    
    // Intentar mover en Y
    if can_move_to(player.pos.x, new_y, maze, block_size) {
        player.pos.y = new_y;
    }

    // Normalizar el ángulo para evitar overflow
    while player.a > 2.0 * PI {
        player.a -= 2.0 * PI;
    }
    while player.a < 0.0 {
        player.a += 2.0 * PI;
    }
}

/// Verifica si el jugador puede moverse a esa posición con un pequeño buffer
fn can_move_to(x: f32, y: f32, maze: &Maze, block_size: usize) -> bool {
    // Buffer de colisión para que el jugador no se pegue exactamente a las paredes
    let collision_buffer = 8.0;
    
    // Verificar múltiples puntos alrededor del jugador (hitbox)
    let check_points = [
        (x - collision_buffer, y - collision_buffer), // Esquina superior izquierda
        (x + collision_buffer, y - collision_buffer), // Esquina superior derecha
        (x - collision_buffer, y + collision_buffer), // Esquina inferior izquierda
        (x + collision_buffer, y + collision_buffer), // Esquina inferior derecha
        (x, y), // Centro
    ];

    for (check_x, check_y) in check_points.iter() {
        let col = (*check_x as usize) / block_size;
        let row = (*check_y as usize) / block_size;

        if row >= maze.len() || col >= maze[0].len() {
            return false; // Fuera de límites
        }

        if maze[row][col] != ' ' {
            return false; // Colisión con pared
        }
    }

    true
}