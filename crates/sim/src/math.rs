//! Deterministic math helpers.
//!
//! Basic float arithmetic and `sqrt` are IEEE-exact everywhere, but the platform's
//! `sin`/`cos`/`atan2`/`powf`… are not. The sim must call these wrappers (pure-Rust
//! `libm`) instead, so results match across machines. `glam` is built with its
//! `libm` feature for the same reason.

use glam::Vec2;

pub const PI: f32 = core::f32::consts::PI;
pub const TAU: f32 = core::f32::consts::TAU;

pub fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

pub fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

pub fn atan2(y: f32, x: f32) -> f32 {
    libm::atan2f(y, x)
}

/// The arcsine of `x`, radians, `[-PI/2, PI/2]`; NaN outside `[-1, 1]`.
pub fn asin(x: f32) -> f32 {
    libm::asinf(x)
}

/// The square root of `x`, through `libm` like the rest of this module (IEEE-exact everywhere, as `f32::sqrt` is).
pub fn sqrt(x: f32) -> f32 {
    libm::sqrtf(x)
}

/// The cube root of `x` (the surges' `used^(2/3)`, `crate::surge::damage`), through `libm` like the rest of this
/// module.
pub fn cbrt(x: f32) -> f32 {
    libm::cbrtf(x)
}

pub fn degrees_to_radians(deg: f32) -> f32 {
    deg * (PI / 180.0)
}

/// Normalize an angle to `(-PI, PI]`.
pub fn wrap_angle(a: f32) -> f32 {
    let mut a = libm::remainderf(a, TAU);
    if a <= -PI {
        a += TAU;
    }
    a
}

/// Unit vector for a facing angle (0 = east, positive = clockwise on screen, +y south).
pub fn facing_vector(angle: f32) -> Vec2 {
    Vec2::new(cos(angle), sin(angle))
}

/// Facing angle of a direction vector.
pub fn vector_angle(v: Vec2) -> f32 {
    atan2(v.y, v.x)
}

/// One turn towards a direction ([`turn_toward`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Turn {
    /// The angle turned, radians: at most the budget.
    pub angle: f32,
    /// Whether the facing is now the wanted direction, else it turned the whole budget and has further to go.
    pub done: bool,
}

/// Turns `facing` towards `wanted` (both in `(-PI, PI]`) the shorter way, by at most `budget` radians (`f32::INFINITY`
/// for no limit): exactly onto `wanted` when it is within the budget.
pub fn turn_toward(facing: &mut f32, wanted: f32, budget: f32) -> Turn {
    let turn = wrap_angle(wanted - *facing);
    if turn.abs() <= budget {
        *facing = wanted;
        Turn {
            angle: turn.abs(),
            done: true,
        }
    } else {
        *facing = wrap_angle(*facing + budget.copysign(turn));
        Turn {
            angle: budget,
            done: false,
        }
    }
}
