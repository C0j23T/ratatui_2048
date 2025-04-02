use std::{ops::Mul, time::Duration};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Player {
    pub id: i32,
    pub name: String,
    pub best_score: i32,
    pub best_time: i64,
    pub best_timestamp: i64,
    pub records: Vec<PlayerRecord>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerRecord {
    pub score: i32,
    pub time: i64,
    pub timestamp: i64,
}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.best_score != other.best_score {
            self.best_score.cmp(&other.best_score)
        } else {
            self.best_time.cmp(&other.best_time)
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, Copy)]
pub struct Cell {
    value: u16,
}

impl Cell {
    pub fn new(value: u16) -> Self {
        Self { value }
    }

    pub fn get(&self) -> u16 {
        self.value
    }

    pub fn set_v(&mut self, value: u16) {
        self.value = value;
    }

    pub fn set(&mut self, value: Self) {
        self.value = value.value;
    }

    pub fn empty(&self) -> bool {
        self.value == 0
    }
}

impl Mul<u16> for Cell {
    type Output = Self;
    fn mul(self, rhs: u16) -> Self::Output {
        Self {
            value: self.value * rhs,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy)]
pub enum CellMotionDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellAnimationType {
    Popup,
    Move,
}

impl PartialOrd for CellAnimationType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellAnimationType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            return std::cmp::Ordering::Equal;
        }
        if self == &CellAnimationType::Popup && other == &CellAnimationType::Move {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

#[derive(Clone, Copy)]
pub struct AnimationCell {
    pub src: Vec2,
    pub value: u16,
    pub animation_type: CellAnimationType,
    pub dest: Option<Vec2>,
    pub duration: Duration,
}
