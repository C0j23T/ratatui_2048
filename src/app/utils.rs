use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::layout::Rect;

use super::math::lerpf;

pub fn get_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .try_into()
        .unwrap()
}

pub fn rect_scale(rect: Rect, factor: f32) -> Rect {
    if factor < 0.0 || !factor.is_finite() {
        return rect;
    }

    let new_width = ((rect.width as f32) * factor).round() as u16;
    let new_height = ((rect.height as f32) * factor).round() as u16;

    let center_x = rect.x as f32 + rect.width as f32 / 2.0;
    let center_y = rect.y as f32 + rect.height as f32 / 2.0;

    let new_x = (center_x - new_width as f32 / 2.0).round() as i32;
    let new_y = (center_y - new_height as f32 / 2.0).round() as i32;

    if new_x < 0 || new_x > u16::MAX as i32 || new_y < 0 || new_y > u16::MAX as i32 {
        return rect;
    }

    Rect {
        x: new_x as u16,
        y: new_y as u16,
        width: new_width,
        height: new_height,
    }
}

pub fn rect_move(src: Rect, dest: Rect, progress: f32) -> Rect {
    if progress <= 0.0 || !progress.is_finite() {
        return src;
    }
    if progress >= 1.0 {
        return dest;
    }
    Rect {
        x: lerpf(src.x as f32..=dest.x as f32, progress).round() as u16,
        y: lerpf(src.y as f32..=dest.y as f32, progress).round() as u16,
        width: lerpf(src.width as f32..=dest.width as f32, progress).round() as u16,
        height: lerpf(src.height as f32..=dest.height as f32, progress).round() as u16,
    }
}
