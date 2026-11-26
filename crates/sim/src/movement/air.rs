//! (M5-H1) Air units keep apart softly — `systems/units.md > Air Collision`: "Air units use soft separation with
//! other air units — a gentle repulsion force that prevents stacking but does not hard-block"; "SeparationRadius — per
//! unit type, circular, the distance at which air-to-air repulsion activates" (plan decision T81).
//!
//! After every unit has moved (`super::step`), each pair of air units closer than the larger of their two
//! SeparationRadii (`UnitDef::separation_radius`), centre to centre, is pushed apart along the line between them, each
//! unit by half of what the pair lacks, at most [`SEPARATION_PUSH_SU`]; a unit's pushes from all its pairs together are
//! capped at that too (plan decision T81: "by at most 2 su a tick each"). (M5-H2) A unit takes its base's share of each
//! push and of the cap (`UnitBaseDef::separation_strength`): a HoverCraft all of it, a Glider a quarter (design change
//! 0405: "**Gliders push apart slowly** [...] (placeholder: a quarter of the HoverCraft strength), so they drift apart
//! gradually, mostly while circling, and their attack runs and loops are barely bent"). The pushes are summed from
//! where the moves left the units, pair by pair in id order, and applied together, so that no unit's push depends on
//! another's. Nothing blocks: a push moves the unit whatever is there (air units collide with nothing). Two units on
//! the very same point part along the x axis, the lower id to the west. A Glider may be pushed past the map's edge, as
//! it may fly there (`super::glider`); any other air unit (a HoverCraft) stays on the map (design change 0045: "clamp
//! HoverCraft to the map"). A push is no part of the unit's own move: its velocity and momentum stay what its movement
//! gave it, and a unit without a goal stays at rest, pushed.
//!
//! The pairs come from a spatial index: the movement's own grid ([`Layer`]), each air unit listed in the cell of its
//! centre, so that a unit looks only at the cells within the largest radius of it. Units spawned in this tick are
//! pushed against but not pushed, as they don't move in their spawn tick (`docs/movement.md > 4`).

use std::collections::BTreeMap;

use glam::Vec2;

use super::space::Layer;
use crate::defs::{Domain, UnitDef};
use crate::ids::ObjectId;
use crate::map::Map;
use crate::object::Object;
use crate::tuning::air::SEPARATION_PUSH_SU;
use crate::units::SU_PER_GU;

/// An air unit with a SeparationRadius, as the moves left it.
#[derive(Copy, Clone, Debug)]
struct Flyer {
    id: ObjectId,
    pos: Vec2,
    /// Its SeparationRadius, space units.
    radius: f32,
    /// (M5-H2) Its base's share of each push (`UnitBaseDef::separation_strength`).
    strength: f32,
    /// It moves in this tick: it wasn't spawned in it.
    moves: bool,
}

/// The separation's scratch: rebuilt every tick, kept only for its allocations.
#[derive(Debug, Default)]
pub(crate) struct Scratch {
    flyers: Vec<Flyer>,
    layer: Layer,
    pushes: Vec<Vec2>,
    near: Vec<u32>,
}

/// The separation of phase 5d, after the moves (module docs). Units before `first_new` are pushed.
pub(super) fn separate(
    objects: &mut BTreeMap<ObjectId, Object>,
    first_new: ObjectId,
    map: &Map,
    scratch: &mut Scratch,
) {
    let Scratch {
        flyers,
        layer,
        pushes,
        near,
    } = scratch;
    flyers.clear();
    for o in objects.values() {
        let Some(unit) = o.def().unit.as_ref().filter(|_| o.mover.is_some()) else {
            continue;
        };
        let base = unit.base_def();
        if let (Domain::Air, Some(radius), Some(strength)) =
            (base.domain, unit.separation_radius, base.separation_strength)
        {
            flyers.push(Flyer {
                id: o.id,
                pos: o.pos,
                radius,
                strength,
                moves: o.id < first_new,
            });
        }
    }
    if flyers.len() < 2 {
        return;
    }
    layer.reset(map.width(), map.height());
    let mut widest: f32 = 0.0;
    for (index, flyer) in flyers.iter().enumerate() {
        layer.insert(index as u32, flyer.pos, Vec2::ZERO, 0);
        widest = widest.max(flyer.radius);
    }
    pushes.clear();
    pushes.resize(flyers.len(), Vec2::ZERO);
    let reach = Vec2::splat(widest);
    for (i, a) in flyers.iter().enumerate() {
        // Each unit is listed in one cell: the later ones near it, once each, in id order.
        near.clear();
        layer.near(a.pos - reach, a.pos + reach, |j| {
            if j as usize > i {
                near.push(j);
            }
        });
        near.sort_unstable();
        for &j in near.iter() {
            let b = flyers[j as usize];
            let apart = a.radius.max(b.radius);
            let off = a.pos - b.pos;
            let distance_squared = off.length_squared();
            if distance_squared >= apart * apart {
                continue;
            }
            let distance = distance_squared.sqrt();
            let away = if distance > 0.0 { off / distance } else { Vec2::NEG_X };
            let push = away * ((apart - distance) / 2.0).min(SEPARATION_PUSH_SU);
            pushes[i] += push * a.strength;
            pushes[j as usize] -= push * b.strength;
        }
    }
    let size = Vec2::new(map.width() as f32, map.height() as f32) * SU_PER_GU;
    for (flyer, push) in flyers.iter().zip(pushes.iter()) {
        if !flyer.moves || *push == Vec2::ZERO {
            continue;
        }
        let Some(o) = objects.get_mut(&flyer.id) else {
            continue;
        };
        let push = push.clamp_length_max(SEPARATION_PUSH_SU * flyer.strength);
        let mut to = o.pos + push;
        if !(o.def().unit.as_ref()).is_some_and(UnitDef::glides) {
            let half = o.def().half_extents().unwrap_or(Vec2::ZERO).min(size / 2.0);
            to = to.clamp(half, size - half);
        }
        o.pos = to;
    }
}

#[cfg(test)]
mod tests;
