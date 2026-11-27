//! (M6-GD) The leash ring (plan §GD: "the client: [...] the leash ring"; `factions/gdo/units/drone_operator.md >
//! Placeholder stats`: "**leash range 8 grid units**, measured **edge to edge** like ranges"): while the local player
//! has a launched drone selected, or the operator it is on the leash of, a ring on the ground round the operator where
//! the leash ends, each operator's once ([`leashes`]). It is the line the drone's centre crosses as its Silhouette comes
//! its leash from the operator's, edge to edge (`sc_sim::tether::past_leash`): both Silhouettes are square, so they
//! never turn, and the line is the square of their two half extents rounded by the leash ([`ring`]). Past it the drone
//! falls and explodes ("falls and explodes"). Each point is drawn at the terrain's height, as a cast's circle is
//! ([`super::cast`]), at the back of the indicators. Placeholder look (Q0001): a thin pale blue line.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use sc_sim::{Map, Object, ObjectId, PlayerId, SU_PER_GU, Sim, tether};

use crate::coords::{drawn_position, view_to_world};
use crate::selection::Selection;
use crate::shapes::ColorMesh;

/// The line's width, su, its colour, and the segments of each of its four arcs (placeholders).
const WIDTH: f32 = 2.5;
pub const LEASH_COLOR: Color = Color::srgba(0.55, 0.85, 1.0, 0.8);
const ARC_SEGMENTS: usize = 24;

/// A drone on its leash, by its operator's id and its own.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Leash {
    pub operator: ObjectId,
    pub drone: ObjectId,
}

/// The leashes whose rings show to `player`, whose `selection` this is (module docs): of its selected drones, and of its
/// selected operators whose drones are out, each operator's once, in the operators' id order.
pub fn leashes(sim: &Sim, selection: &Selection, player: PlayerId) -> Vec<Leash> {
    let leash = |o: &Object| match (o.tether, o.drone) {
        (Some(tether), _) => Some(Leash {
            operator: tether.operator,
            drone: o.id,
        }),
        (None, Some(drone)) => Some(Leash { operator: o.id, drone }),
        (None, None) => None,
    };
    let held = |leash: &Leash| (sim.object(leash.operator)).is_some_and(|o| o.drone == Some(leash.drone));
    let mut leashes: Vec<Leash> = (selection.ids())
        .filter_map(|id| sim.object(id))
        .filter(|o| o.owner == Some(player))
        .filter_map(leash)
        .filter(held)
        .collect();
    leashes.sort_unstable();
    leashes.dedup();
    leashes
}

/// Where `leash`'s ring runs on the ground, at the render `alpha` (module docs): the points of its four arcs, each the
/// leash's radius round a corner of the square of the operator's and the drone's half extents, from the east round to
/// the east again, its last point its first; the straight lines between the arcs join their ends. None if either has
/// left the game, or the drone's type has no leash.
pub fn ring(sim: &Sim, leash: Leash, alpha: f32) -> Option<Vec<Vec2>> {
    let operator = sim.object(leash.operator)?;
    let drone = sim.object(leash.drone)?;
    let reach = tether::def_of(drone.kind)?.leash * SU_PER_GU;
    let half = |o: &Object| o.kind.def().half_extents().unwrap_or(Vec2::ZERO);
    let half = half(operator) + half(drone);
    let centre = operator.prev_pos.lerp(operator.pos, alpha);
    let corners = [
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
    ];
    let mut points: Vec<Vec2> = (corners.into_iter().enumerate())
        .flat_map(|(quarter, corner)| {
            (0..=ARC_SEGMENTS).map(move |i| {
                let a = (quarter as f32 + i as f32 / ARC_SEGMENTS as f32) * FRAC_PI_2;
                centre + half * corner + Vec2::new(a.cos(), a.sin()) * reach
            })
        })
        .collect();
    points.push(points[0]);
    Some(points)
}

/// Draws a ring through the ground `points` ([`ring`]), each at the terrain's height, at `z`.
pub(super) fn draw(mesh: &mut ColorMesh, map: &Map, points: &[Vec2], z: f32) {
    let drawn: Vec<Vec2> = (points.iter())
        .map(|&p| view_to_world(drawn_position(map, p)))
        .collect();
    for pair in drawn.windows(2) {
        mesh.segment(pair[0], pair[1], WIDTH, z, LEASH_COLOR);
    }
}
