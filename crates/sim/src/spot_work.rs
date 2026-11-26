//! Work at a footprint from a spot beside it (plan decisions T59 and T62, design change 0339, slice M5-E): what a unit
//! does that builds a site from outside — `factions/cults/economy.md > Cults Building Mechanics` ("Cults buildings are
//! constructed on-site by Recruits working from outside the structure"; "Construction speed scales proportionally with
//! the number of Recruits currently assigned"), `factions/cults/economy.md > Cults Building Mechanics > Group
//! construction rules` (design change 0339), `factions/cults/economy.md > Assist Construction Command` ("Once the
//! selected Recruits reach the building they join construction from outside"). Two rules, each written once over an
//! order and its target, not for one order alone, so that every order worked so reuses them: a group's Build, an
//! Assist, and a Repair (slice M5-N8, `crate::repair`: "several repairers on one target repair **faster, scaling
//! linearly** (2 repairers = twice the rate), as construction does", design change 0321):
//!
//! - **The claim of a spot** ([`work_of`], [`Claim`]): each unit working at a footprint walks to its own spot beside
//!   it, which its order's state holds (`orders::state::SiteWalk::spot`, `orders::state::SpotWork::spot`) and which no
//!   other unit of its owner working at the same footprint may take: the claims are derived from the orders, never
//!   stored apart. The spots' step (`Sim::step_spots`) gives each unit without one, those nearest the footprint first,
//!   the first free spot of 0339's search (`placement::Wall`): "each Recruit walks to the **part of the footprint
//!   closest to it**; if that part is taken (blocked), it **wall-hugs** along the footprint until it reaches a part
//!   where it can work". The search runs along the builders' wall, `tuning::construction::BUILD_STANDOFF_SU` off the
//!   footprint, so that builders stand a little off it and off each other; once that wall is full, along a wall
//!   further out, where a unit waits and doesn't count (0339: "Recruits still walking, waiting for a spot or doing
//!   anything else add nothing"), the row next to the builders left free as the way round them. A unit walks round the
//!   footprint to its spot, never over it. Every `tuning::construction::RESPOT_TICKS`, a unit whose walk is over within
//!   reach takes the place where it stands as its spot, and one whose walk is over out of reach looks again along the
//!   builders' wall and walks to the free spot it finds.
//! - **The count of those at work** ([`counts`], [`at_work`]): the units whose current order works at a target,
//!   standing within `tuning::construction::BUILD_REACH_SU` of its edge with their walk over (0339: "a building needs a
//!   fixed number of **Recruit-frames** (BaseConstructionTime with 1 Recruit). Every frame, each Recruit that is
//!   building during that frame adds 1. Recruits still walking, waiting for a spot or doing anything else add
//!   nothing"), derived in each tick, so that n of them give exactly n times one's rate. A site built from outside
//!   gains a frame for each (`sim/sites.rs`), and a repaired object its share of HP for each repairer
//!   (`sim/repairs.rs`). The edge is a footprint's, or a unit's Silhouette for a vehicle's repairers, which work at no
//!   spot but follow the vehicle (`orders::repair`).
//!
//! Where a unit works ([`WorkAt`]) is a footprint: a group's Build's, before its site starts (its ticket's,
//! `sim/builds/shared.rs`), or a structure's (an Assist's site, a Repair's building). An order on a structure that no
//! longer suits it ([`suits`]: an Assist's site complete, a repaired building at full HP, either not its owner's) ends
//! in the spots' step.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::augmentation::Research;
use crate::construction::site::shares_builds;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::MoveStatus;
use crate::object::{Object, Placement};
use crate::orders::OrderKind;
use crate::orders::build::build_site;
use crate::orders::state::{Behaviour, RepairWork};
use crate::repair;
use crate::shape::Shape;
use crate::tuning::construction::BUILD_REACH_SU;

/// What a unit works at from a spot beside it (module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WorkAt {
    /// The footprint of a group's Build whose site hasn't started (plan decision T59).
    Footprint(Placement),
    /// A structure, by its footprint: an Assist's site, a Repair's building.
    Object(ObjectId),
}

/// The work of a unit's current order from a spot beside a footprint ([`work_of`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Work {
    pub order: OrderKind,
    pub at: WorkAt,
    /// The spot it claimed there, once the spots' step gave it one.
    pub spot: Option<Vec2>,
}

/// The work `o`'s current order does from a spot beside a footprint (module docs): a Build of a worker that shares
/// builds (`WorkerDef::shares_builds`), before its site starts, an Assist, and a building's Repair
/// (`RepairWork::Spot`). `None` for every other order, a vehicle's Repair among them, and for an object without
/// one.
pub fn work_of(o: &Object) -> Option<Work> {
    let active = o.orders.as_ref()?.current.as_ref()?;
    let order = active.order.kind;
    let (at, spot) = match (order, &active.state) {
        (OrderKind::Build { .. }, Behaviour::Build(walk)) if shares_builds(o.kind) => {
            (WorkAt::Footprint(build_site(order)?.1), walk.spot)
        }
        (OrderKind::Assist(site), Behaviour::Assist(work)) => (WorkAt::Object(site), work.spot),
        (OrderKind::Repair(target), Behaviour::Repair(RepairWork::Spot(work))) => (WorkAt::Object(target), work.spot),
        _ => return None,
    };
    Some(Work { order, at, spot })
}

/// The spot of `o`'s work ([`work_of`]), for the spots' step to give it one.
pub(crate) fn spot_mut(o: &mut Object) -> Option<&mut Option<Vec2>> {
    let shares = shares_builds(o.kind);
    let active = o.orders.as_mut()?.current.as_mut()?;
    let order = active.order.kind;
    match &mut active.state {
        Behaviour::Build(walk) if shares && matches!(order, OrderKind::Build { .. }) => Some(&mut walk.spot),
        Behaviour::Assist(work) if matches!(order, OrderKind::Assist(_)) => Some(&mut work.spot),
        Behaviour::Repair(RepairWork::Spot(work)) if matches!(order, OrderKind::Repair(_)) => Some(&mut work.spot),
        _ => None,
    }
}

/// Whether `order`, carried by `unit`, works at `target` (module docs): an Assist at a site of the same owner's that
/// its builders build from outside (`Object::is_built_from_outside`; design change 0339, `factions/cults/economy.md >
/// Cults Building Mechanics > Group construction rules`: "**Assist** targets only a building **already under
/// construction**, not a ghost whose order hasn't started"); (M5-N8) a Repair at an object that `unit` can repair now
/// (`repair::can_repair`: its owner's, complete and damaged, of a kind its type repairs; (M5-N1a) damaged below its
/// MaxHP at its owner's levels in `research`). The orders that work at no target suit none.
pub fn suits(order: OrderKind, unit: &Object, target: &Object, research: Research) -> bool {
    let owner = unit.owner;
    match order {
        OrderKind::Assist(_) => owner.is_some() && target.owner == owner && target.is_built_from_outside(),
        OrderKind::Repair(_) => repair::can_repair(unit, target, research),
        _ => false,
    }
}

/// Whether a unit whose shape is `shape` is within reach of the footprint `at` (module docs): within
/// `tuning::construction::BUILD_REACH_SU` of its edge, edge to edge (`crate::shape`).
pub fn within_reach(shape: &Shape, at: &Placement) -> bool {
    shape.gap(&Shape::footprint(at)) <= BUILD_REACH_SU
}

/// A spot a unit claims beside a footprint it works at (module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Claim {
    pub unit: ObjectId,
    pub owner: Option<PlayerId>,
    /// The footprint it works at.
    pub at: Placement,
    pub spot: Vec2,
    /// Half the room it keeps clear at the spot along each axis: its box, and `tuning::construction::BUILD_STANDOFF_SU`
    /// round it.
    pub room: Vec2,
}

impl Claim {
    /// Whether this claim keeps a unit of `owner`'s working at the footprint `at`, whose room has half extents `room`,
    /// from the spot `spot`: the two rooms would overlap (touching is clear), and the claim is another unit's of the
    /// same owner at the same footprint. Another player's units claim nothing against it: where they stand is the
    /// placement check's (`Sim::check_unit_fits`), so no claim tells a player of another's work.
    pub fn keeps(&self, unit: ObjectId, owner: Option<PlayerId>, at: &Placement, spot: Vec2, room: Vec2) -> bool {
        self.unit != unit
            && self.owner == owner
            && self.at == *at
            && (self.spot - spot).abs().cmplt(self.room + room).all()
    }
}

/// The target `o` is at work on now (module docs): the one its current order works at by `target_of`, if the order
/// suits it ([`suits`], at the owners' levels in `research`), `o` standing within reach of its edge (`Shape::of`: a
/// structure's footprint, as [`within_reach`] has it, or a unit's Silhouette), its walk over (no move on its way,
/// `MoveStatus::Moving`). `None` for a unit at work on nothing.
pub fn at_work(
    o: &Object,
    objects: &BTreeMap<ObjectId, Object>,
    research: Research,
    target_of: impl Fn(OrderKind) -> Option<ObjectId>,
) -> Option<ObjectId> {
    let order = o.orders.as_ref()?.current.as_ref()?.order.kind;
    let target = target_of(order)?;
    let t = objects.get(&target).filter(|t| suits(order, o, t, research))?;
    let walking = o.mover.as_ref().is_some_and(|mover| mover.status == MoveStatus::Moving);
    (!walking && Shape::of(o).gap(&Shape::of(t)) <= BUILD_REACH_SU).then_some(target)
}

/// The count of those at work (module docs): for each target, the units of `objects` at work on it by `target_of`
/// ([`at_work`]). Builders of a site built from outside by `target_of` = an Assist's site ([`assisted`]).
pub fn counts(
    objects: &BTreeMap<ObjectId, Object>,
    research: Research,
    target_of: impl Fn(OrderKind) -> Option<ObjectId>,
) -> BTreeMap<ObjectId, u32> {
    let mut counts = BTreeMap::new();
    for o in objects.values() {
        if let Some(target) = at_work(o, objects, research, &target_of) {
            *counts.entry(target).or_insert(0) += 1;
        }
    }
    counts
}

/// The site an order assists, if it is an Assist: the target of the count of a site's builders ([`counts`]).
pub fn assisted(order: OrderKind) -> Option<ObjectId> {
    match order {
        OrderKind::Assist(site) => Some(site),
        _ => None,
    }
}

/// (M5-N8) The object an order repairs, if it is a Repair: the target of the count of its repairers ([`at_work`]).
pub fn repaired(order: OrderKind) -> Option<ObjectId> {
    match order {
        OrderKind::Repair(target) => Some(target),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
