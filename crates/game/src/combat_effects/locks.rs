//! (M5-K) The feedback of the Target Locks under way (`factions/gdo/units/rocketman.md > Target Lock`): on the target
//! of each lock of one of the player's own units (`sc_sim::AttackState::target_lock`), while the player sees it, a
//! reticle round its body. While locking ("the Rocketman aims at the target continuously for LockOnDuration"): an
//! amber arc that fills clockwise from the top as LockOnDuration runs, four ticks that close in on the target with it,
//! and a faint aiming line from the unit to the target. Once locked: a red ring, its ticks home, and no line. Each
//! lock has its own, several on one target drawn over each other. **Placeholder (not in the design: who sees a lock,
//! and its look; Q0001)**: only the unit's owner, whose order the lock is.

use bevy::prelude::*;
use sc_sim::{Object, SU_PER_GU, Sim};

use crate::coords::{drawn_at_height, view_to_world};
use crate::fog::Sight;
use crate::object_render::body_height;
use crate::shapes::ColorMesh;

/// The reticle's gap outside the target's silhouette, how far out its ticks start, their length, and the widths of
/// its arc and its line, su (placeholders). The gap clears the order's marker, which rings the target 6 su out while
/// the unit is selected (`command_ui::indicators`, the Target Lock's being the Attack's), so that it never hides the
/// arc.
const GAP: f32 = 12.0;
const TICKS_OUT: f32 = 10.0;
const TICK_LENGTH: f32 = 6.0;
const ARC_WIDTH: f32 = 2.5;
const LINE_WIDTH: f32 = 1.5;
/// The segments of a full ring.
const SEGMENTS: usize = 40;
/// A lock's colours: amber while it locks, red once locked (placeholders).
const LOCKING: Srgba = Srgba::rgb(1.0, 0.74, 0.2);
const LOCKED: Srgba = Srgba::rgb(1.0, 0.22, 0.18);

/// A Target Lock's reticle (module docs), view space.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LockMark {
    /// The middle of the target's body as drawn now, and the reticle's radius round it.
    pub center: Vec2,
    pub radius: f32,
    /// The middle of the locking unit's body, where its aiming line leaves.
    pub from: Vec2,
    /// How far its locking has come: its ticks over LockOnDuration (`sc_sim::defs::TargetLockDef::lock_on_ticks`), 1
    /// once locked.
    pub progress: f32,
    pub locked: bool,
}

/// The reticles to draw (module docs): one for each Target Lock of a unit of `sight`'s player on a target the player
/// sees, both at their interpolated positions.
pub fn lock_marks(sim: &Sim, sight: &Sight, alpha: f32) -> Vec<LockMark> {
    (sim.objects())
        .filter(|o| o.owner == Some(sight.player()))
        .filter_map(|o| {
            let lock = o.attack.as_ref()?.target_lock?;
            let def = o.def().unit.as_ref()?.target_lock?;
            let target = sim.object(lock.target).filter(|t| sight.sees(t))?;
            let progress = match lock.locked {
                true => 1.0,
                false => (lock.elapsed as f32 / def.lock_on_ticks.max(1) as f32).min(1.0),
            };
            Some(LockMark {
                center: body_center(sim, target, alpha),
                radius: reach(target) + GAP,
                from: body_center(sim, o, alpha),
                progress,
                locked: lock.locked,
            })
        })
        .collect()
}

/// The middle of `o`'s body as drawn now: at its interpolated position, at its body's height (an aircraft's in the
/// air), view space.
fn body_center(sim: &Sim, o: &Object, alpha: f32) -> Vec2 {
    let pos = o.prev_pos.lerp(o.pos, alpha);
    drawn_at_height(pos, body_height(sim.map(), o.kind, pos, o.placement.as_ref()))
}

/// How far `o`'s body reaches from its middle, su: a unit's silhouette's corners however it turns, a structure's
/// footprint's longer half.
fn reach(o: &Object) -> f32 {
    match (o.def().half_extents(), o.placement.as_ref()) {
        (Some(half), _) => half.length(),
        (None, Some(p)) => p.w.max(p.h) as f32 * SU_PER_GU / 2.0,
        (None, None) => SU_PER_GU / 2.0,
    }
}

/// Draws `mark` (module docs).
pub(super) fn lock_shape(mesh: &mut ColorMesh, mark: &LockMark, z: f32) {
    let center = view_to_world(mark.center);
    let color = if mark.locked { LOCKED } else { LOCKING };
    if !mark.locked {
        let line = color.with_alpha(0.35).into();
        mesh.segment(view_to_world(mark.from), center, LINE_WIDTH, z, line);
    }
    // The arc, clockwise from the top on screen (Bevy's world is +y up).
    let filled = (mark.progress * SEGMENTS as f32).ceil() as usize;
    let (inner, outer) = (mark.radius - ARC_WIDTH / 2.0, mark.radius + ARC_WIDTH / 2.0);
    let along = |i: usize| {
        let a = std::f32::consts::FRAC_PI_2 - i as f32 * std::f32::consts::TAU / SEGMENTS as f32;
        Vec2::new(a.cos(), a.sin())
    };
    for i in 0..filled.min(SEGMENTS) {
        let (a, b) = (along(i), along(i + 1));
        let corners = [
            center + a * inner,
            center + a * outer,
            center + b * outer,
            center + b * inner,
        ];
        mesh.quad(corners.map(|p| p.extend(z)), color.with_alpha(0.9).into());
    }
    // The four ticks, from TICKS_OUT beyond the ring in to it as the locking runs.
    let out = mark.radius + ARC_WIDTH + TICKS_OUT * (1.0 - mark.progress);
    for dir in [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y] {
        let (from, to) = (center + dir * (out + TICK_LENGTH), center + dir * out);
        mesh.segment(from, to, ARC_WIDTH + 0.5, z, color.into());
    }
}

#[cfg(test)]
mod tests;
