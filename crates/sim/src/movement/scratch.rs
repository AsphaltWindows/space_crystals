//! The scratch of phase 5d (`docs/movement.md > 3`) and the bodies it is built from: every unit as the others see it
//! this tick ([`Body`], [`body_of`]), the objects the units head for ([`Chased`]), the collision layers, the turn
//! budget phase 5e reads ([`Turns`]) and (M5-V4) the tick's crushes. Rebuilt from the objects every tick by
//! `super::step`.

use glam::Vec2;

use super::space::{self, Body, Layer};
use super::{air, collide, crush, stuck, yielding};
use crate::augmentation::Research;
use crate::defs::{Domain, MovementAttrs, UnitDef};
use crate::ids::{ObjectId, PlayerId};
use crate::mover::{MoveGoal, MoveStatus, Mover, PathState};
use crate::object::Object;
use crate::orders::gather;
use crate::production::ExitDoor;
use crate::shape::Shape;

/// The work of one movement phase: output for tests and the perf overlay, never part of the state or its hash
/// (`docs/movement.md > 4`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MoveCounters {
    /// Units with a goal and a path that the step moved, or tried to.
    pub steered: u32,
    /// Box sweeps against the units of a layer.
    pub unit_sweeps: u32,
    /// Box-against-box tests inside those sweeps.
    pub pair_tests: u32,
    /// Box sweeps against the tiles.
    pub tile_sweeps: u32,
    /// (G) Oncoming units looked at by lane seeking (`collide::World::lane`).
    pub lane_tests: u32,
}

/// The collision layers with hard collision (`docs/movement.md > 6.1`): ground and underground. Air units have
/// none: they collide with nothing, and (M5-H1) keep apart softly after the moves ([`air`]).
pub(super) const LAYERS: usize = 2;

pub(super) fn layer_of(domain: Domain) -> Option<usize> {
    match domain {
        Domain::Ground => Some(0),
        Domain::Underground => Some(1),
        Domain::Air => None,
    }
}

/// The object a unit heads for (`MoveGoal::Object`), as phase 5d finds it this tick.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Chased {
    pub id: ObjectId,
    /// Its centre: what paths to it are planned to (`Mover::planned_target`).
    pub pos: Vec2,
    /// Reach and the occupied-target rule are measured to its edge (`shape`, design changes 0031–0033).
    pub shape: Shape,
}

/// (G) The one turn budget of a tick (`docs/movement.md > 6.3`): a unit turns at most its [`turn_rate`] per tick in
/// all. Phase 5d notes here the angle it turned each unit, and phase 5e's Aiming turns a unit only by what is left
/// of the budget (`combat::step`). By ascending id; a unit that didn't turn isn't listed, nor (M5-V2) one whose model
/// turns it by its speed ([`turn_rate`] 0: a Glider, a FixedTurnRadiusMovement unit), which has no budget to spend.
/// Listed, it would leave its aim a negative budget, which `math::turn_toward` spends as a positive one: the aim of a
/// base would turn it in place by as much again.
///
/// Transient, not state: 5d writes it afresh and 5e reads it within the same tick, so it never outlives the tick. A
/// save between two ticks has none of it to keep, and the state hash nothing to cover: it follows from the state the
/// tick began with and the tick's inputs, which the hash and the replay already fix.
#[derive(Debug, Default)]
pub(crate) struct Turns(pub(super) Vec<(ObjectId, f32)>);

impl Turns {
    /// The angle phase 5d turned unit `id` in this tick, radians.
    pub(crate) fn angle(&self, id: ObjectId) -> f32 {
        self.0
            .binary_search_by_key(&id, |&(id, _)| id)
            .map_or(0.0, |i| self.0[i].1)
    }
}

/// The scratch of phase 5d (`docs/movement.md > 3`): rebuilt from the objects every tick. Kept between ticks only
/// to reuse its allocations, except [`Turns`], which phase 5e of the same tick reads.
#[derive(Debug, Default)]
pub(crate) struct Scratch {
    /// Every unit, in id order.
    pub(super) bodies: Vec<Body>,
    /// For each of `bodies`: the object it heads for, if its goal is one that exists.
    pub(super) chased: Vec<Option<Chased>>,
    /// For each of `bodies`: the exit doorway it is still in (`Mover::exit`, design changes 0080 and 0145).
    pub(super) doors: Vec<Option<ExitDoor>>,
    /// For each of `bodies`: the target its attack engages (`AttackState::target`), for queueing behind attackers
    /// (`stuck`).
    pub(super) engaged: Vec<Option<ObjectId>>,
    pub(super) layers: [Layer; LAYERS],
    /// Units in the way of the unit being moved.
    pub(super) touched: Vec<usize>,
    /// The lanes of the oncoming units ahead of the unit being moved (`collide::World::lane`).
    pub(super) lanes: Vec<collide::Lane>,
    /// The sidesteps asked of idle units this tick (`yielding`), applied when every unit has moved.
    pub(super) requests: Vec<yielding::Request>,
    /// (M4-S2) The footprints of the lowered pop-up turrets that want to rise and have a unit over them
    /// (`popup::is_blocked`), each with its owner: their owners' idle units there are asked to step off
    /// (`yielding::make_way`).
    pub(super) rising: Vec<(Option<PlayerId>, Vec2, Vec2)>,
    /// The angle each unit turned in this tick's phase 5d, for 5e.
    pub(super) turns: Turns,
    pub(super) counters: MoveCounters,
    /// (M5-H1) The air units' separation ([`air`]).
    pub(super) air: air::Scratch,
    /// (M5-V4) This tick's crushes, in the order they came ([`crush`]).
    pub(super) crushed: Vec<crush::Crush>,
}

impl Scratch {
    pub(crate) fn counters(&self) -> MoveCounters {
        self.counters
    }

    pub(crate) fn turns(&self) -> &Turns {
        &self.turns
    }
}

/// `Body::heading`: the way to the unit's steering waypoint, if it has a path to follow.
fn heading_of(pos: Vec2, mover: &Mover) -> Vec2 {
    if mover.goal == MoveGoal::None || mover.path_state != PathState::Ready {
        return Vec2::ZERO;
    }
    let target = mover.path.get(usize::from(mover.next));
    target.map_or(Vec2::ZERO, |&target| target - pos)
}

/// (M5-V1) How unit `o`'s Silhouette, of half extents `half`, lies if it turns with the facing
/// (`defs::ObjectDef::silhouette_turns`): along its facing. `None` for a square one.
pub(super) fn oriented_of(o: &Object, half: Vec2) -> Option<space::Oriented> {
    (o.def().silhouette_turns()).then(|| space::Oriented::facing(half, o.facing))
}

/// A unit as the others see it this tick; `movement`, its movement numbers (`Object::movement`); `half`, half its
/// Silhouette. (M5-V1) One that turns with the facing lies along it ([`oriented_of`]), its box the bounding box of
/// that.
pub(super) fn body_of(o: &Object, mover: &Mover, movement: &MovementAttrs, half: Vec2) -> Body {
    // Also while it waits for a path: a unit that just planned again is still on its way.
    let on_its_way = mover.goal != MoveGoal::None;
    let oriented = oriented_of(o, half);
    Body {
        id: o.id,
        owner: o.owner,
        pos: o.pos,
        half: oriented.map_or(half, |oriented| oriented.bound_half()),
        oriented,
        velocity: mover.velocity,
        heading: heading_of(o.pos, mover),
        stationary: mover.is_stationary(),
        progress: on_its_way.then_some(mover.last_progress),
        arrived_with: mover.group.filter(|_| mover.status == MoveStatus::Arrived),
        yields: yielding::can_yield(o, mover, movement),
        gathers: gather::on_gather_job(o),
        leads: stuck::leads(mover),
        moved: mover.last_progress,
        // (M5-V4) Set for the unit's own turn alone (`super::crush::drives`).
        crushes: false,
        crushable: o.def().unit.as_ref().is_some_and(|unit| unit.base_def().crushable),
    }
}

/// A unit that takes part in movement: its definition, (M5-N1a) its movement numbers as its Augmentations make them
/// at its owner's levels in `research` (`Object::movement_ref`: its kind's own, or made in `slot`), and its half
/// extents.
pub(super) fn unit_of<'a>(
    o: &Object,
    research: Research,
    slot: &'a mut Option<MovementAttrs>,
) -> Option<(&'static UnitDef, &'a MovementAttrs, Vec2)> {
    let def = o.kind.def();
    o.mover.as_ref()?;
    Some((def.unit.as_ref()?, o.movement_ref(research, slot)?, def.half_extents()?))
}
