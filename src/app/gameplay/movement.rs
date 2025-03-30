use std::time::Duration;

use crate::app::structs::{AnimationCell, Cell, CellAnimationType, CellMotionDirection, Vec2};

use super::Grid;

pub fn move_up(cells: &mut Grid) -> (Vec<AnimationCell>, u16) {
    let mut animations = Vec::new();
    let mut total_score = 0;
    for i in 0..4 {
        let mut tmp = Vec::new();
        for line in cells.iter_mut() {
            let ptr = line.as_mut_ptr();
            tmp.push(unsafe { &mut *ptr.add(i) });
        }
        let (anims, score) = swipe_line(&mut tmp, CellMotionDirection::Up, i);
        animations.extend(anims);
        total_score += score;
    }
    (animations, total_score)
}

pub fn move_down(cells: &mut Grid) -> (Vec<AnimationCell>, u16) {
    let mut animations = Vec::new();
    let mut total_score = 0;
    for i in 0..4 {
        let mut tmp = Vec::new();
        for j in (0..4).rev() {
            let ptr = cells[j].as_mut_ptr();
            tmp.push(unsafe { &mut *ptr.add(i) });
        }
        let (anims, score) = swipe_line(&mut tmp, CellMotionDirection::Down, i);
        animations.extend(anims);
        total_score += score;
    }
    (animations, total_score)
}

pub fn move_left(cells: &mut Grid) -> (Vec<AnimationCell>, u16) {
    let mut animations = Vec::new();
    let mut total_score = 0;
    for (i, row) in cells.iter_mut().enumerate() {
        let mut tmp = row.iter_mut().collect::<Vec<_>>();
        let (anims, score) = swipe_line(&mut tmp, CellMotionDirection::Left, i);
        animations.extend(anims);
        total_score += score;
    }
    (animations, total_score)
}

pub fn move_right(cells: &mut Grid) -> (Vec<AnimationCell>, u16) {
    let mut animations = Vec::new();
    let mut total_score = 0;
    for (i, row) in cells.iter_mut().enumerate() {
        let mut tmp = row.iter_mut().rev().collect::<Vec<_>>();
        let (anims, score) = swipe_line(&mut tmp, CellMotionDirection::Right, i);
        animations.extend(anims);
        total_score += score;
    }
    (animations, total_score)
}

fn swipe_line(
    cells: &mut [&mut Cell],
    direction: CellMotionDirection,
    index: usize,
) -> (Vec<AnimationCell>, u16) {
    let mut animations = Vec::new();
    let mut score = 0;

    for i in 1..4 {
        if cells[i].empty() {
            continue;
        }

        let mut temp = i - 1;
        while temp > 0 && cells[temp].empty() {
            temp -= 1;
        }

        let (temp_coord, i_coord) = match &direction {
            CellMotionDirection::Up => (Vec2 { x: temp, y: index }, Vec2 { x: i, y: index }),
            CellMotionDirection::Down => (
                Vec2 {
                    x: 3 - temp,
                    y: index,
                },
                Vec2 { x: 3 - i, y: index },
            ),
            CellMotionDirection::Left => (Vec2 { x: index, y: temp }, Vec2 { x: index, y: i }),
            CellMotionDirection::Right => (
                Vec2 {
                    x: index,
                    y: 3 - temp,
                },
                Vec2 { x: index, y: 3 - i },
            ),
        };

        if cells[temp] == cells[i] {
            cells[temp].set(*cells[temp] * 2_u16);
            score = cells[temp].get();

            animations.push(AnimationCell {
                src: i_coord,
                value: cells[i].get(),
                animation_type: CellAnimationType::Move,
                dest: Some(temp_coord),
                duration: Duration::default(),
            });

            *cells[i] = Cell::default();
        } else if cells[temp].empty() {
            cells[temp].set(*cells[i]);

            animations.push(AnimationCell {
                src: i_coord,
                value: cells[i].get(),
                animation_type: CellAnimationType::Move,
                dest: Some(temp_coord),
                duration: Duration::default(),
            });

            *cells[i] = Cell::default();
        } else if temp + 1 != i {
            temp += 1;
            cells[temp].set(*cells[i]);

            let temp_coord = match &direction {
                CellMotionDirection::Up => Vec2 { x: temp, y: index },
                CellMotionDirection::Down => Vec2 {
                    x: 3 - temp,
                    y: index,
                },
                CellMotionDirection::Left => Vec2 { x: index, y: temp },
                CellMotionDirection::Right => Vec2 {
                    x: index,
                    y: 3 - temp + 1,
                },
            };
            animations.push(AnimationCell {
                src: i_coord,
                value: cells[i].get(),
                animation_type: CellAnimationType::Move,
                dest: Some(temp_coord),
                duration: Duration::default(),
            });

            *cells[i] = Cell::default();
        }
    }

    (animations, score)
}
