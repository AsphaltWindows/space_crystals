//! What the movement step holds of a unit: [`Body`], and how its Silhouette lies if it turns with its facing
//! ([`Oriented`]) — `docs/movement.md > 6.1`, `docs/movement.md > 3`. The two types alone, split out of `super` (plan
//! decision T28); their code is there, with the grid's and the geometry's.

use glam::Vec2;

use crate::ids::{ObjectId, PlayerId};
use crate::units::Tick;

/// A unit as the collision step sees it: a copy, so the step needs no second borrow of the object map
/// (`docs/movement.md > 4`). Units already moved this tick are listed at their new position, the rest at their old
/// one.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Body {
    pub id: ObjectId,
    /// For friendly yielding (`super::yielding`) and the exceptions of [`blocks`](super::blocks) (crushing spares own and allied
    /// units).
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
    /// The half extents of its box: half its Silhouette, or (M5-V1) for one that turns with its facing
    /// ([`Body::oriented`]) of the bounding box of the turned Silhouette (`shape::bounding_half`).
    pub half: Vec2,
    /// (M5-V1) For a unit whose Silhouette turns with its facing (`defs::ObjectDef::silhouette_turns`): how it lies.
    /// `None` for a square one, whose shape is its box, `half`.
    pub oriented: Option<Oriented>,
    /// Its latest displacement: this tick's if it already moved, else last tick's.
    pub velocity: Vec2,
    /// (G) Where it is heading: the vector from it to its steering waypoint (any length), zero for a unit without
    /// a goal or waiting for its path. Unlike `velocity` it doesn't turn with a step around a blocker, so it tells
    /// an oncoming unit from one that only steps aside (`super::collide::World::lane`, [`Body::heads_against`]).
    pub heading: Vec2,
    /// `Mover::is_stationary`.
    pub stationary: bool,
    /// For a unit on its way (it has a goal): the tick of its latest progress, `Mover::last_progress`. Units
    /// waiting behind it share it (`docs/movement.md > 6.5`).
    pub progress: Option<Tick>,
    /// For a unit whose latest move ended `Arrived`: that move's order group (`Mover::group`).
    pub arrived_with: Option<u32>,
    /// Whether it steps aside for a blocked unit of its owner (`super::yielding::can_yield`, design changes 0022
    /// and 0023).
    pub yields: bool,
    /// Whether it is a Drone on a Gather job (`orders::gather::on_gather_job`, design change 0106): two such
    /// bodies don't collide ([`blocks`](super::blocks)).
    pub gathers: bool,
    /// (M3-H2) Whether those waiting behind it share its `progress` (`super::stuck::leads`): it has got on since
    /// its leg began.
    pub leads: bool,
    /// (M3-G2) `Mover::last_progress`, whether or not the unit is on its way: for one that stands, about when it
    /// got there (its last leg's latest progress, or the tick that leg began). A unit queueing behind it reads
    /// from it whether it came to stand in its way since its last attempt at a way round (`super::stuck`).
    pub moved: Tick,
    /// (M5-V4) Whether its move this tick crushes (`super::crush::drives`): set on the moving unit's own body for its
    /// turn alone, false on every other body, as [`blocks`](super::blocks) reads only the mover's.
    pub crushes: bool,
    /// (M5-V4) `UnitBaseDef::crushable`: a crusher's move goes through it if it is an enemy's ([`blocks`](super::blocks)).
    pub crushable: bool,
}

/// (M5-V1) How a Silhouette that turns with its unit's facing lies (`systems/units.md > UnitCollision > Silhouette
/// Rotation`): its length along the facing.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Oriented {
    /// Half its length and half its width (`defs::ObjectDef::half_extents`).
    pub half: Vec2,
    /// The facing it lies at, radians.
    pub facing: f32,
    /// That facing as a unit vector (`math::facing_vector`): the direction of its length.
    pub axis: Vec2,
}
