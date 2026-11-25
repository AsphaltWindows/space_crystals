//! Path knowledge (`systems/controls/behaviors.md > PathKnowledge`; the verification points of design changes 0040
//! and 0042): paths are planned only from what the unit's owner knows, units collide with what is really there, and
//! a unit re-routes when its owner discovers an obstacle. Design change 0038's own points are in
//! `resources_block.rs`.
//!
//! The maps here are 40 × 20 with a mountain wall along column 25, a gap in it at rows 6 and 7 and open ground at
//! its south end, rows 18 and 19: the gap is the short way east, the south end the long one.
//!
//! One test binary:
//! - `plans`: what a plan goes by: a hidden wall seen on the way (0040), the owner's own structures, and a crystal
//!   field never seen (0042);
//! - `remembered`: what the player knows of, gone out of its sight, routed around until seen gone (0040, 0042);
//! - `collision`: units collide with what is really there, known or not;
//! - `grids`: the players' grids, built with their first walking unit, and a save with stale knowledge.

mod collision;
mod grids;
mod plans;
mod remembered;

use sc_sim::nav::NavClass;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SU_PER_GU, Sim, SimEvent, SpawnAt, Target,
    TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
/// The Peacekeeper's half silhouette (24 × 24 su).
const HALF: f32 = 12.0;
/// The column of the wall, and the two rows of its gap.
const WALL: i32 = 25;
const GAP: i32 = 6;
/// The first row of the open ground at the wall's south end.
const SOUTH: i32 = 18;

/// The 40 × 20 map: with the wall, or all open ground.
fn map(wall: bool) -> String {
    (0..20)
        .map(|y| {
            (0..40)
                .map(|x| {
                    let solid = wall && x == WALL && y < SOUTH && y != GAP && y != GAP + 1;
                    if solid { 'M' } else { '.' }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// A point given in tiles: `(25.5, 7.0)` is the middle of the gap.
fn at(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

/// West of the wall, out of sight of the gap; on the gap's line east of the wall, out of sight of it too; and
/// beside the gap, in sight of it.
const WEST: (f32, f32) = (8.5, 7.0);
const EAST: (f32, f32) = (36.5, 7.0);
const BY_THE_GAP: (f32, f32) = (29.5, 7.0);

fn point((x, y): (f32, f32)) -> Vec2 {
    at(x, y)
}

fn move_to(t: &mut TestSim, unit: u32, to: Vec2) {
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, &[ObjectId(unit)], OrderKind::Move(Target::Point(to)), false);
}

/// Whether the unit's box at `pos` overlaps the footprint `(x, y, w, h)` in tiles. Touching is free.
fn overlaps(pos: Vec2, (x, y, w, h): (i32, i32, i32, i32)) -> bool {
    let lo = Vec2::new(x as f32, y as f32) * SU_PER_GU;
    let hi = lo + Vec2::new(w as f32, h as f32) * SU_PER_GU;
    pos.x - HALF < hi.x && pos.x + HALF > lo.x && pos.y - HALF < hi.y && pos.y + HALF > lo.y
}

/// Steps until unit `id` no longer moves, and returns where it was at the end of each tick.
fn walk(t: &mut TestSim, id: u32) -> Vec<Vec2> {
    let mut track = vec![t.step().pos(id)];
    while t.status(id) == MoveStatus::Moving {
        assert!(track.len() < 2000, "still moving after 2000 ticks");
        track.push(t.step().pos(id));
    }
    track
}

fn went_south(track: &[Vec2]) -> bool {
    track.iter().any(|p| p.y > SOUTH as f32 * SU_PER_GU)
}

fn went_through_the_gap(track: &[Vec2]) -> bool {
    track
        .iter()
        .any(|&p| TilePos::containing(p).x == WALL && (GAP..GAP + 2).contains(&TilePos::containing(p).y))
}

fn version(sim: &Sim, player: PlayerId) -> u32 {
    sim.path_knowledge().version(player).unwrap()
}

fn failed_moves(t: &TestSim) -> usize {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::MoveFailed { .. }))
        .count()
}

/// Whether `player` plans around tile `(x, y)`: its known grid blocks it.
fn known_blocked(sim: &Sim, player: PlayerId, x: i32, y: i32) -> bool {
    sim.known_nav(player)
        .unwrap()
        .grid(NavClass::GroundRugged, 1)
        .unwrap()
        .is_blocked(TilePos::new(x, y))
}
