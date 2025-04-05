use std::ops::RangeInclusive;

/// 所有可用的曲线（插值）类型  
/// 请参阅 [Interpolation - libGDX](https://libgdx.com/wiki/math-utils/interpolation)
#[derive(Default, Debug, Clone, Copy)]
pub enum Interpolation {
    #[default]
    Linear,
    ExpOut {
        value: f32,
    },
    PowOut {
        value: i32,
    },
    SwingOut,
}

impl Interpolation {
    pub fn apply(&self, a: f32) -> f32 {
        match self {
            Self::Linear => a,
            Self::SwingOut => swing_out(2.0, a),
            Self::ExpOut { value } => exp_out(2.0, *value, a),
            Self::PowOut { value } => pow_out(*value, a),
        }
    }
}

#[inline]
pub fn swing_out(scale: f32, mut a: f32) -> f32 {
    a -= 1.0;
    a * a * ((scale + 1.0) * a + scale) + 1.0
}

#[inline]
pub fn exp_out(value: f32, power: f32, a: f32) -> f32 {
    let min = value.powf(-power);
    let scale = 1.0 / (1.0 - min);

    1.0 - (value.powf(-power * a) - min) * scale
}

#[inline]
pub fn pow_out(power: i32, a: f32) -> f32 {
    (a - 1.0).powi(power) * if power % 2 == 0 { -1.0 } else { 1.0 } + 1.0
}

#[inline]
pub fn inverse_lerp(range: RangeInclusive<f32>, value: f32) -> f32 {
    let min = *range.start();
    let max = *range.end();
    if min == max {
        1.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

#[inline]
pub fn inverse_lerp_f64(range: RangeInclusive<f64>, value: f64) -> f64 {
    let min = *range.start();
    let max = *range.end();
    if min == max {
        1.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

#[inline]
pub fn lerpf(range: impl Into<RangeInclusive<f32>>, t: f32) -> f32 {
    let range = range.into();
    (1.0 - t) * *range.start() + t * *range.end()
}
