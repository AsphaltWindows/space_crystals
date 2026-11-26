//! Work at a footprint from a spot beside it (plan decisions T59 and T62, slice M5-E): what a unit does that builds a
//! site from outside — `factions/cults/economy.md > Cults Building Mechanics` ("Cults buildings are constructed on-site
//! by Recruits working from outside the structure"; "Construction speed scales proportionally with the number of
//! Recruits currently assigned"), `factions/cults/economy.md > Assist Construction Command` ("Once the selected Recruits
//! reach the building they join construction from outside"). Two rules, each written once over an order and its
//! target, not for one order alone, so that every order worked so reuses them (a group's Build and an Assist today;
//! the Repair of slice N8, whose "repairers stack linearly", design change 0321):
//!
//! - **The claim of a spot** ([`work_of`], [`Claim`]): each unit working at a footprint walks to its own spot beside
//!   it, which its order's state holds (`orders::state::SiteWalk::spot`, `orders::state::SpotWork::spot`) and which no
//!   other unit of its owner working at the same footprint may take: the claims are derived from the orders, never
//!   stored apart. The spots' step (`Sim::step_spots`) gives each unit without one, those nearest the footprint first,
//!   the first free spot among T62's (`placement::Spots`; **Placeholder (Q0097 point 1)**: "each builder at its own
//!   spot within 8 su of the footprint's edge, the spots searched in a fixed order around it"): those of a box
//!   `tuning::construction::BUILD_STANDOFF_SU` bigger than the unit's, so that builders stand a little off the wall
//!   and each other; beside the footprint's sides only, so that its corners stay free for those walking round; ring 0
//!   first, nearest the unit first, and once it is full a ring from the second on, where a unit waits and doesn't
//!   count (**Placeholder (Q0097 point 8)**: "others wait further out and don't count"), ring 1 left free as the way
//!   round the builders. Every `tuning::construction::RESPOT_TICKS`, a unit whose walk is over within reach takes the
//!   place where it stands as its spot, and one whose walk is over out of reach looks again on ring 0 and walks to the
//!   free spot it finds.
//! - **The count within reach** ([`counts`]): the units whose current order works at a target, standing within
//!   `tuning::construction::BUILD_REACH_SU` of its footprint's edge (**Placeholder (Q0097 point 5)**: "Only those at
//!   the site"), derived in each tick, so that n of them give exactly n times one's rate. A site built from outside
//!   gains a frame for each (`sim/sites.rs`).
//!
//! Where a unit works ([`WorkAt`]) is a footprint: a group's Build's, before its site starts (its ticket's,
//! `sim/builds/shared.rs`), or a structure's (an Assist's site). An order on a structure that no longer suits it
//! ([`suits`]: an Assist's site complete, or not its owner's) ends in the spots' step.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::construction::site::shares_builds;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Object, Placement};
use crate::orders::OrderKind;
use crate::orders::build::build_site;
use crate::orders::state::Behaviour;
use crate::shape::Shape;
use crate::tuning::construction::BUILD_REACH_SU;

/// What a unit works at from a spot beside it (module docs).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WorkAt {
    /// The footprint of a group's Build whose site hasn't started (plan decision T59).
    Footprint(Placement),
    /// A structure, by its footprint: an Assist's site.
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
/// builds (`WorkerDef::shares_builds`), before its site starts, and an Assist. `None` for every other order, and for an
/// object without one.
pub fn work_of(o: &Object) -> Option<Work> {
    let active = o.orders.as_ref()?.current.as_ref()?;
    let order = active.order.kind;
    let (at, spot) = match (order, &active.state) {
        (OrderKind::Build { .. }, Behaviour::Build(walk)) if shares_builds(o.kind) => {
            (WorkAt::Footprint(build_site(order)?.1), walk.spot)
        }
        (OrderKind::Assist(site), Behaviour::Assist(work)) => (WorkAt::Object(site), work.spot),
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
        _ => None,
    }
}

/// Whether `order`, carried by a unit of `owner`'s, works at `target` (module docs): an Assist at a site of the same
/// owner's that its builders build from outside (`Object::is_built_from_outside`; **Placeholder (Q0097 point 7)**:
/// "Assist goes only to a site that has started"). The orders that work at no structure suit none.
pub fn suits(order: OrderKind, owner: Option<PlayerId>, target: &Object) -> bool {
    match order {
        OrderKind::Assist(_) => owner.is_some() && target.owner == owner && target.is_built_from_outside(),
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
    /// The unit's half extents, its box at the spot.
    pub half: Vec2,
}

impl Claim {
    /// Whether this claim keeps a unit of `owner`'s working at the footprint `at`, whose box has half extents `half`,
    /// from the spot `spot`: the two boxes would overlap (touching is clear), and the claim is another unit's of the
    /// same owner at the same footprint. Another player's units claim nothing against it: where they stand is the
    /// placement check's (`Sim::check_unit_fits`), so no claim tells a player of another's work.
    pub fn keeps(&self, unit: ObjectId, owner: Option<PlayerId>, at: &Placement, spot: Vec2, half: Vec2) -> bool {
        self.unit != unit
            && self.owner == owner
            && self.at == *at
            && (self.spot - spot).abs().cmplt(self.half + half).all()
    }
}

/// The count within reach (module docs): for each target, the units of `objects` whose current order `target_of` maps
/// to it, which it suits ([`suits`]), standing within reach of its footprint's edge ([`within_reach`]). Builders of a
/// site built from outside by `target_of` = an Assist's site.
pub fn counts(
    objects: &BTreeMap<ObjectId, Object>,
    target_of: impl Fn(OrderKind) -> Option<ObjectId>,
) -> BTreeMap<ObjectId, u32> {
    let mut counts = BTreeMap::new();
    for o in objects.values() {
        let Some(active) = o.orders.as_ref().and_then(|orders| orders.current.as_ref()) else {
            continue;
        };
        let Some(target) = target_of(active.order.kind) else {
            continue;
        };
        let Some(t) = objects.get(&target).filter(|t| suits(active.order.kind, o.owner, t)) else {
            continue;
        };
        if t.placement.is_some_and(|at| within_reach(&Shape::of(o), &at)) {
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

#[cfg(test)]
mod tests;
