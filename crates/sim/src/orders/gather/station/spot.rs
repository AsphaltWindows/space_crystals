//! Where a worker stands to pick up at a Supply Delivery Station (slice M4-I; `factions/syndicate/units/agent.md >
//! Gathering > Supplies`). "Any number pick up at once, as at patches" (design change 0256 point 1), and Agents
//! collide with each other (question Q0076), so a leg to a station heads for a **free spot**: the point nearest to the
//! worker where its Silhouette stands against one of the station's four faces, or beside a face's end within its
//! PickUpReach of the station (the hatch's rule, `legs::return_leg`), clear of
//! - the structures and map resources its owner knows of (`World::known_fixed`, as for `legs::approach_point`);
//! - the units its owner sees there in the worker's own domain, those picking up or waiting and any other;
//! - the spots that the workers before it in this tick's phase 5a set off for ([`Claims`]), so that workers sent
//!   together spread round the station.
//!
//! A leg to a patch heads for the point nearest to the worker wholly within a face (`legs::approach_point`). A
//! second worker from the same side heads for the first one's place, and its leg ends short behind it, out of reach:
//! at a station it would wait for the first to leave, though both may pick up at once.
//!
//! A spot is claimed only in the tick its leg begins. A worker that sets off later may head for a spot that one on its
//! way already makes for; arriving second, it stands short of it, out of reach, and sets off for another at its next
//! re-check (`station::wait_at_station`). With every spot taken, the leg is a patch's (`legs::leg`), and the worker
//! waits as close as it gets. Like the patch's legs, a spot takes no account of terrain: a station stands on flat
//! ground (`core/map.md > Elevation > Map resources only on flat terrain`), and a spot off it is one the mover gets as
//! near to as it can.

use glam::Vec2;

use crate::shape::Shape;

use super::super::super::state::Gather;
use super::super::super::world::{KnownStation, Unit, World};
use super::super::Claims;
use super::super::legs::{leg, leg_to};

/// How far a spot keeps the worker's Silhouette from what stands beside it, and inside its reach, space units: one,
/// for the rounding.
const CLEARANCE: f32 = 1.0;

/// The leg to `station` for a worker whose PickUpReach is `reach` (module docs): it begins, unless one is under way,
/// to the free spot nearest to the worker, which it claims for this tick; with none free, to the point nearest to it
/// wholly within a face, as a leg to a patch.
pub(super) fn station_leg(
    state: &mut Gather,
    unit: &mut Unit,
    station: &KnownStation,
    reach: f32,
    world: &World,
    claims: &mut Claims,
) {
    if state.moving {
        return;
    }
    match free_spot(unit, station, reach, world, claims) {
        Some(spot) => {
            let half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
            claims.take_spot(station.id, Shape::rect(spot, half));
            leg_to(state, unit, spot, world);
        }
        None => leg(state, unit, &station.shape, None, world),
    }
}

/// The free spot nearest to `unit` at `station` (module docs), the first of the faces north, south, west and east
/// among equals; `None` if there's none.
fn free_spot(unit: &Unit, station: &KnownStation, reach: f32, world: &World, claims: &Claims) -> Option<Vec2> {
    let half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO);
    let (lo, hi) = station.shape.bounds();
    // How far the worker's centre may stand beyond a face's end: its near edge stays within its reach.
    let beside = Vec2::splat((reach - CLEARANCE).max(0.0)) + half;
    let obstacles = obstacles(unit, station, beside + half, world, claims);
    let mut best: Option<(f32, Vec2)> = None;
    // Each face as the axis it runs along (0 for x) and where the worker's centre stands across it.
    for (along, across) in [
        (0, lo.y - half.y),
        (0, hi.y + half.y),
        (1, lo.x - half.x),
        (1, hi.x + half.x),
    ] {
        let other = 1 - along;
        let span = (lo[along] - beside[along], hi[along] + beside[along]);
        // The stretches of the face where the worker's centre has no room, for what stands in its way.
        let blocked: Vec<(f32, f32)> = (obstacles.iter())
            .map(Shape::bounds)
            .filter(|(o_lo, o_hi)| {
                o_lo[other] < across + half[other] + CLEARANCE && across - half[other] - CLEARANCE < o_hi[other]
            })
            .map(|(o_lo, o_hi)| {
                (
                    o_lo[along] - half[along] - CLEARANCE,
                    o_hi[along] + half[along] + CLEARANCE,
                )
            })
            .collect();
        let free = |at: f32| span.0 <= at && at <= span.1 && !blocked.iter().any(|&(from, to)| from < at && at < to);
        // Where the worker would stand on this face with nothing in its way, and the ends of each stretch in it.
        let wanted = unit.pos[along].clamp(span.0, span.1);
        let candidates = std::iter::once(wanted).chain(blocked.iter().flat_map(|&(from, to)| [from, to]));
        for at in candidates.filter(|&at| free(at)) {
            let mut spot = Vec2::ZERO;
            spot[along] = at;
            spot[other] = across;
            let distance = spot.distance_squared(unit.pos);
            if best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, spot));
            }
        }
    }
    best.map(|(_, spot)| spot)
}

/// What may stand where the worker would (module docs), within `margin` of `station`: the structures and map
/// resources its owner knows of but the station; the units its owner sees in the worker's domain but the worker; and
/// the spots claimed at the station in this tick.
fn obstacles(unit: &Unit, station: &KnownStation, margin: Vec2, world: &World, claims: &Claims) -> Vec<Shape> {
    let (lo, hi) = station.shape.bounds();
    let (lo, hi) = (lo - margin, hi + margin);
    let near = |shape: &Shape| {
        let (o_lo, o_hi) = shape.bounds();
        o_lo.x < hi.x && lo.x < o_hi.x && o_lo.y < hi.y && lo.y < o_hi.y
    };
    let domain = world.get(unit.id).map(|own| own.domain);
    let fixed = (world.known_fixed(unit.owner).into_iter()).filter(|shape| *shape != station.shape && near(shape));
    let units = (world.units())
        .filter(|view| view.id != unit.id && Some(view.domain) == domain && near(&view.shape))
        .filter(|view| world.sees(unit.owner, view))
        .map(|view| view.shape);
    let claimed = claims.spots(station.id).filter(near);
    fixed.chain(units).chain(claimed).collect()
}
