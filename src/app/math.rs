use std::{f32::consts::PI, ops::RangeInclusive};

/// 所有可用的曲线（插值）类型  
/// 请参阅 [Interpolation - libGDX](https://libgdx.com/wiki/math-utils/interpolation)
#[derive(Default, Debug, Clone, Copy)]
pub enum Interpolation {
    #[default]
    Linear,
    Sine,
    Quadratic,

    Circle,
    CircleIn,
    CircleOut,

    Elastic,
    ElasticIn,
    ElasticOut,

    Exp {
        value: f32,
    },
    ExpIn {
        value: f32,
    },
    ExpOut {
        value: f32,
    },

    Pow {
        value: i32,
    },
    PowIn {
        value: i32,
    },
    PowOut {
        value: i32,
    },
    Pow2InInverse,
    Pow2OutInverse,
    Pow3InInverse,
    Pow3OutInverse,

    Swing,
    SwingIn,
    SwingOut,
}

impl Interpolation {
    pub fn apply(&self, a: f32) -> f32 {
        match self {
            Self::Linear => a,
            Self::Circle => circle(a),
            Self::CircleIn => circle_in(a),
            Self::CircleOut => circle_out(a),
            Self::Elastic => elastic(2.0, 10.0, 7, 1.0, a),
            Self::ElasticIn => elastic_in(2.0, 10.0, 6, 1.0, a),
            Self::ElasticOut => elastic_out(2.0, 10.0, 7, 1.0, a),
            Self::Sine => (1.0 - (a * PI).cos()) / 2.0,
            Self::Swing => swing(1.5, a),
            Self::SwingIn => swing_in(2.0, a),
            Self::SwingOut => swing_out(2.0, a),
            Self::Exp { value } => exp(2.0, *value, a),
            Self::ExpIn { value } => exp_in(2.0, *value, a),
            Self::ExpOut { value } => exp_out(2.0, *value, a),
            Self::Pow { value } => pow(*value, a),
            Self::PowIn { value } => pow_in(*value, a),
            Self::PowOut { value } => pow_out(*value, a),
            Self::Pow2InInverse => pow2_in_inverse(a),
            Self::Pow2OutInverse => pow2_out_inverse(a),
            Self::Pow3InInverse => pow3_in_inverse(a),
            Self::Pow3OutInverse => pow3_out_inverse(a),
            Self::Quadratic => quadratic(a),
        }
    }
}

#[inline]
pub fn circle(mut a: f32) -> f32 {
    if a <= 0.5 {
        a *= 2.0;
        return (1.0 - (1.0 - a * a).sqrt()) / 2.0;
    }
    a -= 1.0;
    a *= 2.0;
    ((1.0 - a * a).sqrt() + 1.0) / 2.0
}

#[inline]
pub fn circle_in(a: f32) -> f32 {
    1.0 - (1.0 - a * a).sqrt()
}

#[inline]
pub fn circle_out(mut a: f32) -> f32 {
    a -= 1.0;
    (1.0 - a * a).sqrt()
}

#[inline]
pub fn elastic(value: f32, power: f32, bounces: i32, scale: f32, mut a: f32) -> f32 {
    let bounces = bounces as f32 * PI * if bounces % 2 == 0 { 1.0 } else { -1.0 };

    if a <= 0.5 {
        a *= 2.0;
        return value.powf(power * (a - 1.0)) * (a * bounces).sin() * scale / 2.0;
    }
    a = 1.0 - a;
    a *= 2.0;
    1.0 - value.powf(power * (a - 1.0)) * (a * bounces).sin() * scale / 2.0
}

#[inline]
pub fn elastic_in(value: f32, power: f32, bounces: i32, scale: f32, a: f32) -> f32 {
    let bounces = bounces as f32 * PI * if bounces % 2 == 0 { 1.0 } else { -1.0 };

    if a >= 0.99 {
        return 1.0;
    }
    value.powf(power * (a - 1.0)) * (a * bounces).sin() * scale / 2.0
}

#[inline]
pub fn elastic_out(value: f32, power: f32, bounces: i32, scale: f32, mut a: f32) -> f32 {
    let bounces = bounces as f32 * PI * if bounces % 2 == 0 { 1.0 } else { -1.0 };

    if a == 0.0 {
        return 0.0;
    }
    a = 1.0 - a;
    1.0 - value.powf(power * (a - 1.0)) * (a * bounces).sin() * scale
}

#[inline]
pub fn swing(mut scale: f32, mut a: f32) -> f32 {
    scale *= 2.0;

    if a <= 0.5 {
        a *= 2.0;
        return a * a * ((scale + 1.0) * a - scale) / 2.0;
    }
    a -= 1.0;
    a *= 2.0;
    a * a * ((scale + 1.0) * a + scale) / 2.0 + 1.0
}

#[inline]
pub fn swing_in(scale: f32, a: f32) -> f32 {
    a * a * ((scale + 1.0) * a - scale)
}

#[inline]
pub fn swing_out(scale: f32, mut a: f32) -> f32 {
    a -= 1.0;
    a * a * ((scale + 1.0) * a + scale) + 1.0
}

#[inline]
pub fn exp(value: f32, power: f32, a: f32) -> f32 {
    let min = value.powf(-power);
    let scale = 1.0 / (1.0 - min);

    if a <= 0.5 {
        return value.powf(power * (a * 2.0 - 1.0)) * scale / 2.0;
    }
    (2.0 - (value.powf(-power * (a * 2.0 - 1.0)) - min) * scale) / 2.0
}

#[inline]
pub fn exp_in(value: f32, power: f32, a: f32) -> f32 {
    let min = value.powf(-power);
    let scale = 1.0 / (1.0 - min);

    (value.powf(power * (a - 1.0)) - min) * scale
}

#[inline]
pub fn exp_out(value: f32, power: f32, a: f32) -> f32 {
    let min = value.powf(-power);
    let scale = 1.0 / (1.0 - min);

    1.0 - (value.powf(-power * a) - min) * scale
}

#[inline]
pub fn pow(power: i32, a: f32) -> f32 {
    if a <= 0.5 {
        return (a * 2.0).powi(power) / 2.0;
    }
    ((a - 1.0) * 2.0).powi(power) / if power % 2 == 0 { -2.0 } else { 2.0 } + 1.0
}

#[inline]
pub fn pow_in(power: i32, a: f32) -> f32 {
    a.powi(power)
}

#[inline]
pub fn pow_out(power: i32, a: f32) -> f32 {
    (a - 1.0).powi(power) * if power % 2 == 0 { -1.0 } else { 1.0 } + 1.0
}

#[inline]
pub fn pow2_in_inverse(a: f32) -> f32 {
    if a < 0.000001 {
        return 0.0;
    }
    a.sqrt()
}

#[inline]
pub fn pow2_out_inverse(a: f32) -> f32 {
    if a < 0.000001 {
        return 0.0;
    }
    if a > 1.0 {
        return 1.0;
    }
    1.0 - (-(a - 1.0)).sqrt()
}

#[inline]
pub fn pow3_in_inverse(a: f32) -> f32 {
    if a < 0.000001 {
        return 0.0;
    }
    a.cbrt()
}

#[inline]
pub fn pow3_out_inverse(a: f32) -> f32 {
    if a < 0.000001 {
        return 0.0;
    }
    if a > 1.0 {
        return 1.0;
    }
    1.0 - (-(a - 1.0)).cbrt()
}

#[inline]
pub fn quadratic(a: f32) -> f32 {
    (-10.0 * a * a + 10.0 * a) * 0.8
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
