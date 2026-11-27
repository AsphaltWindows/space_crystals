//! (M5-N11) The casts round a caster or ahead of it (`SimEvent::Cast` of an ability whose area is
//! `AbilityArea::AroundCaster` or `AbilityArea::Ahead`: a Prayer-mobile's Inspire and Suppress), shown where they were
//! cast for `FeedbackTuning::cast_ticks`: Inspire's circle of 4 grid units round the vehicle ("every own infantry unit
//! within **4 grid units** of the vehicle") as a gold ring that settles to its radius and fades, over a faint disc;
//! Suppress's arc ("an arc of **90° total** in front of the vehicle's current facing, **5 grid units** long") as a red
//! wedge that fades, its edges brighter. The units the cast took in carry its buff's mark meanwhile (`super::buffs`). A
//! Shrine's cast at a point shows nothing of its own, as before. **Placeholder (not in the design: the casts' look;
//! Q0001)**.
//!
//! The fog: a cast shows only while the tile under the point it was cast from is Visible to the player
//! ([`shows_cast`]), as a blast does (its sighting, `crate::viewer_events`); its owner's own vehicle sees it.

use bevy::prelude::*;
use sc_sim::{AbilityArea, AreaAbility, Map, SU_PER_GU, Sim, Tick};

use super::shows;
use crate::coords::{drawn_at_height, view_to_world};
use crate::fog::Sight;
use crate::shapes::ColorMesh;
use crate::tuning::ClientTuning;
use crate::viewer_events::{Perception, Sighting};
/// Inspire's gold, Suppress's red (the buffs' marks', `super::buffs`), and the width of a ring or an edge, su
/// (placeholders).
const INSPIRE: Srgba = Srgba::rgb(1.0, 0.80, 0.25);
const SUPPRESS: Srgba = Srgba::rgb(1.0, 0.28, 0.22);
const EDGE: f32 = 4.0;
/// The segments of a full circle.
const SEGMENTS: usize = 48;

/// A cast of `ability` from `at` (ground space), facing `facing`, in `tick` (`SimEvent::Cast`).
#[derive(Clone, Debug, PartialEq)]
pub struct CastFlash {
    pub tick: Tick,
    pub ability: AreaAbility,
    pub at: Vec2,
    /// The caster's facing as it cast, radians (`sc_sim::math`: 0 east, clockwise): where Suppress's arc points.
    pub facing: f32,
    /// Whom it reaches, and how: each viewer while the tile under `at` is Visible to them (`crate::viewer_events`).
    pub seen: Sighting,
}

impl CastFlash {
    /// Whether a cast of `ability` shows (module docs): one round its caster or ahead of it.
    pub fn shown_for(ability: AreaAbility) -> bool {
        matches!(
            ability.def().area,
            AbilityArea::AroundCaster | AbilityArea::Ahead { .. }
        )
    }
}

/// Whether `sight` shows `cast` now: for `tuning`'s `FeedbackTuning::cast_ticks` from its tick, as its sighting
/// reaches the player.
pub fn shows_cast(sim: &Sim, sight: &Sight, cast: &CastFlash, tuning: &ClientTuning) -> bool {
    shows(sim, cast.tick, tuning.feedback.cast_ticks) && cast.seen.perceived_by(sight) == Perception::Whole
}

/// The cast `t` of the way through its time (`FeedbackTuning::cast_ticks`; module docs), on the ground at its point's
/// height.
pub(super) fn cast_shape(mesh: &mut ColorMesh, map: &Map, cast: &CastFlash, t: f32, z: f32) {
    let t = t.clamp(0.0, 1.0);
    let def = cast.ability.def();
    let radius = def.radius * SU_PER_GU;
    let height = map.height_at(cast.at);
    // A point `distance` su from the cast's at `angle` (sim space), drawn on the ground, in Bevy world space.
    let point = |angle: f32, distance: f32| {
        view_to_world(drawn_at_height(cast.at + distance * Vec2::from_angle(angle), height))
    };
    let center = point(0.0, 0.0);
    match def.area {
        AbilityArea::Ahead { arc } => {
            let (from, to) = (cast.facing - arc / 2.0, cast.facing + arc / 2.0);
            let steps = 16;
            let angle = |i: usize| from + (to - from) * i as f32 / steps as f32;
            let fill = SUPPRESS.with_alpha(0.28 * (1.0 - t));
            for i in 0..steps {
                mesh.triangle(
                    [center, point(angle(i), radius), point(angle(i + 1), radius)].map(|p| p.extend(z)),
                    fill.into(),
                );
            }
            let edge = SUPPRESS.with_alpha(0.9 * (1.0 - t)).into();
            for side in [from, to] {
                mesh.segment(center, point(side, radius), EDGE, z, edge);
            }
            for i in 0..steps {
                mesh.segment(point(angle(i), radius), point(angle(i + 1), radius), EDGE, z, edge);
            }
        }
        AbilityArea::AroundCaster | AbilityArea::AtPoint => {
            let angle = |i: usize| i as f32 * std::f32::consts::TAU / SEGMENTS as f32;
            let disc: Vec<Vec2> = (0..SEGMENTS).map(|i| point(angle(i), radius)).collect();
            mesh.polygon(&disc, z, INSPIRE.with_alpha(0.16 * (1.0 - t)).into());
            let ring = radius * (0.85 + 0.15 * t);
            let color = INSPIRE.with_alpha(0.9 * (1.0 - t)).into();
            for i in 0..SEGMENTS {
                let (a, b) = (angle(i), angle(i + 1));
                let (inner, outer) = (ring - EDGE / 2.0, ring + EDGE / 2.0);
                let corners = [point(a, inner), point(a, outer), point(b, outer), point(b, inner)];
                mesh.quad(corners.map(|p| p.extend(z)), color);
            }
        }
    }
}
