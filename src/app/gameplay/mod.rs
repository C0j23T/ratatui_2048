use std::time::Duration;

use rand::{Rng, seq::SliceRandom};

use crate::app::structs::{AnimationCell, Cell, CellAnimationType, Vec2};

pub mod colors;
pub mod fx;
pub mod movement;

pub type Grid = Vec<Vec<Cell>>;

pub fn start_up(cells: &mut Grid) -> AnimationCell {
    let mut rng = rand::rng();
    let x = rng.random_range(0..4_usize);
    let y = rng.random_range(0..4_usize);

    cells[x][y] = Cell::new(2);
    AnimationCell {
        src: Vec2 { x, y },
        animation_type: CellAnimationType::Popup,
        dest: None,
        value: 2,
        duration: Duration::default(),
    }
}

#[allow(clippy::needless_range_loop)]
pub fn add_cell(cells: &mut Grid) -> Option<AnimationCell> {
    let mut empty_cells = Vec::new();
    for i in 0..4 {
        for j in 0..4 {
            if cells[i][j].empty() {
                empty_cells.push(Vec2 { x: i, y: j });
            }
        }
    }

    let mut rng = rand::rng();
    empty_cells.shuffle(&mut rng);
    if !empty_cells.is_empty() {
        let coord = empty_cells[0];
        cells[coord.x][coord.y] = Cell::new(2);
        return Some(AnimationCell {
            src: Vec2 {
                x: coord.x,
                y: coord.y,
            },
            animation_type: CellAnimationType::Popup,
            dest: None,
            value: 2,
            duration: Duration::default(),
        });
    }
    None
}

pub fn check_game_over(cells: &mut Grid) -> bool {
    if cells.iter().any(|x| x.iter().any(|y| y.empty())) {
        return false;
    }

    let mut game_over = true;

    for i in 0..3 {
        for j in 0..3 {
            if cells[i][j] == cells[i][j + 1] {
                game_over = false;
            }
            if cells[i][j] == cells[i + 1][j] {
                game_over = false;
            }
        }
    }
    for i in 1..4 {
        for j in 1..4 {
            if cells[i][j] == cells[i][j - 1] {
                game_over = false;
            }
            if cells[i][j] == cells[i - 1][j] {
                game_over = false;
            }
        }
    }

    game_over
}
