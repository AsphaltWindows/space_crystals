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
