//! The `Orders` capability: a unit's command queue and the behaviour of its current order —
//! `systems/controls/commands.md > Unit Command`, `systems/controls/behaviors.md`, `docs/movement.md > 3`,
//! `docs/movement.md > 4`.
//!
//! A player's `Command::Order` (phase 4) only writes queues ([`Orders::give`]) and decides where each unit of a
//! group goes ([`destinations`], the magic box); it never moves anything. Phase 5a ([`update`]) completes orders,
//! starts the next one and runs the behaviours, which write `Mover::goal` and read `Mover::status`.
//!
//! **Behaviours**, each one tick at a time, on the snapshot of the game that [`world`] takes when the phase
//! begins:
//! - [`engage`]: AttackingObject (the approach, the chase and its ends), idle auto-targeting with its leash, and
//!   Hold Position;
//! - [`travel`]: MovingToObject (Follow for a unit), AttackMovingToLocation and Patrolling;
//! - Move to a point, and Stop, need only the mover's status;
//! - [`gather`]: a worker's job, the Gather and Drop Off orders of `factions/colonists/units/drone.md`;
//! - [`build`]: a worker's walk to the site of what it builds, a Drone's Beacon (`crate::deploy`) or an Agent's Tunnel
//!   (`crate::construction::site`);
//! - [`enter`]: a unit's walk into a Tunnel's entry path (`crate::network`);
//! - [`attack_ground`]: AttackingLocation, an Attack Ground's walk into range and its shots, until another order
//!   (slice M4-O; design change 0293).
//!
//! What a unit does by default against a target, a right-click's and a rally point's order, is [`resolution`]'s.
//!
//! Their working state ([`state`], the design's BaseBehaviorState) is part of the order, or of the unit while it
//! has none ([`Orders::auto`]).

mod attack_ground;
pub mod build;
pub(crate) mod destinations;
mod engage;
mod enter;
pub(crate) mod gather;
mod hash;
pub(crate) mod reach;
pub mod resolution;
mod run;
pub mod state;
mod travel;
pub(crate) mod world;

use std::collections::VecDeque;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::scenario::BuildingOrientation;
use crate::units::{Tick, TilePos};

pub use reach::{JUDGE_PATHS_PER_TICK, MAX_REJECTED, ReachCounters};
pub(crate) use run::update;
pub use state::{Auto, Behaviour};

/// `docs/movement.md > 2`, G13: orders waiting in one unit's queue, besides the current one. A technical cap: an
/// order queued beyond it is dropped for that unit.
pub const MAX_QUEUED_ORDERS: usize = 32;

/// Units one order command may name, duplicates included. Technical: selections have no size limit
/// (`systems/controls/selection_and_hud.md`), so this only bounds the work of one command.
pub const MAX_ORDER_UNITS: usize = 1024;

/// The units of one order command share it (`docs/movement.md > 3`). Allocated from `Sim::next_group`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrderGroup(pub u32);

/// What a move order goes to (`systems/controls/commands.md > Move`: a location or an object).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Target {
    /// A point, space units: where the player clicked.
    Point(Vec2),
    Object(ObjectId),
}

/// The orders of `systems/controls/commands.md` that units have so far (`docs/movement.md > 3`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OrderKind {
    /// `systems/controls/commands.md > Move`.
    Move(Target),
    /// `systems/controls/commands.md > Attack`: a destructible object.
    Attack(ObjectId),
    /// `systems/controls/commands.md > AttackMove`: a point.
    AttackMove(Vec2),
    /// `systems/controls/commands.md > Patrol`: the far end of the patrol, a point.
    Patrol(Vec2),
    /// `systems/controls/commands.md > HoldPosition`.
    Hold,
    /// `systems/controls/commands.md > Stop`.
    Stop,
    /// `factions/colonists/units/drone.md > Unit Commands > Gather`: a worker gathers from a Space Crystals patch
    /// and delivers to its AssignedDropOff, again and again ([`gather`]). The patch is the one the worker heads for
    /// now: the order follows it to another when its patch is occupied, out of reach or gone (design change 0132:
    /// "its gather target follows where it actually mined"). A patch that is gone stays named while the worker
    /// heads for where it was (design change 0134, `state::Gather::spot`).
    Gather(ObjectId),
    /// `factions/colonists/units/drone.md > Unit Commands > Drop Off Resources`: a worker delivers the load it carries
    /// to this drop-off of its owner's, an Egress Node ([`gather`]).
    DropOff(ObjectId),
    /// `factions/colonists/economy.md > Beacon Deployment > Deploy Command`: a Drone walks to the site of a Beacon
    /// whose footprint's top-left tile is `origin` and deploys into it there (`crate::deploy`, [`build`]).
    Deploy { origin: TilePos },
    /// `systems/controls/commands.md > Enter`: "The unit walks to Side A and enters the Tunnel Network"
    /// (`factions/syndicate/tunnels.md > Tunnel Network > Walking in`; [`enter`], `crate::network`): this Tunnel of its
    /// owner's, through its entry path.
    Enter(ObjectId),
    /// (M4-O) `systems/controls/commands.md > AttackGround`: "Attack a specific ground location", the point clicked,
    /// space units, with no snapping (plan decision T47). Only units whose attack can target the ground take it
    /// (`combat::can_target_ground`, the gate); [`attack_ground`] runs it.
    AttackGround(Vec2),
    /// `core/entities.md > Placement Validation > Worker-Built Structures` (plan decision T51, slice M4-E): a worker
    /// walks to the site of a structure of `kind` whose footprint's top-left tile is `origin`, turned by `orientation`,
    /// and builds it there (`crate::construction::site`, [`build`]): an Agent's Tunnel
    /// (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`). Its price is paid when it is given
    /// (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, design change 0233).
    Build {
        kind: ObjectKind,
        origin: TilePos,
        #[serde(default, skip_serializing_if = "BuildingOrientation::is_default")]
        orientation: BuildingOrientation,
    },
}

impl OrderKind {
    /// The object the order names, if any.
    pub fn target(self) -> Option<ObjectId> {
        match self {
            OrderKind::Move(Target::Object(id))
            | OrderKind::Attack(id)
            | OrderKind::Gather(id)
            | OrderKind::DropOff(id)
            | OrderKind::Enter(id) => Some(id),
            OrderKind::Move(Target::Point(_)) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => None,
            OrderKind::Hold | OrderKind::Stop | OrderKind::Deploy { .. } | OrderKind::Build { .. } => None,
            OrderKind::AttackGround(_) => None,
        }
    }
}

/// One unit's order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub kind: OrderKind,
    pub group: OrderGroup,
    /// For a location order (Move to a point, AttackMove, Patrol) given to a group: this unit's own destination,
    /// its place in the group's shape (`systems/controls/commands.md > Group Destinations (Magic Box)`,
    /// [`destinations`]). `None` means the clicked point itself: a single unit, or a click inside the group's box.
    pub slot: Option<Vec2>,
}

impl Order {
    /// The point a location order sends this unit to: its own destination, else the clicked point.
    pub fn destination(&self) -> Option<Vec2> {
        match self.kind {
            OrderKind::Move(Target::Point(click)) | OrderKind::AttackMove(click) | OrderKind::Patrol(click) => {
                Some(self.slot.unwrap_or(click))
            }
            OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => None,
            OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } => None,
            OrderKind::Build { .. } => None,
            // The unit goes only as near the location as its attack needs (`attack_ground`).
            OrderKind::Enter(_) | OrderKind::AttackGround(_) => None,
        }
    }

    /// The object the order names, if any.
    pub fn target(&self) -> Option<ObjectId> {
        self.kind.target()
    }

    /// The one rule for an order that names an object no longer in the game (design changes 0134 and 0139): whether
    /// this order goes on naming its target ([`Order::target`]) once the target has left the game. `state` is its
    /// behaviour's if it is the current order; `remembered`, whether its holder's owner remembers the target out of
    /// its sight (`PlayerVision::remembered_record`). The removal path (`Sim::remove_objects`), the end of the tick
    /// (`Sim::follow_targets`) and `testing::dangling_references` all go by it; every other reference to an object
    /// that is gone goes with the object.
    /// - **Remembered** (0139, `core/vision.md > Explored > Targeting remembered objects`: "Nothing about the real
    ///   state leaks"): an Attack, a Move to an object and a Gather go by what their holder's owner knows of the
    ///   target (`world::World::known`): to its last-known position, as if it stood, until the owner sees its place.
    ///   Then each ends "the way it would for a vanished target".
    /// - **A Gather on its way to a spot** (`state::Gather::spot`: where its patch is or was, as its owner last knew
    ///   it): it goes on there and looks round, whatever its owner knows now. That is how a Gather ends for a vanished
    ///   target, a patch that ran out (design change 0134) or one seen gone (0139: "Gather: the redirect rules").
    pub fn outlives_target(&self, state: Option<&Behaviour>, remembered: bool) -> bool {
        let to_spot = matches!(state, Some(Behaviour::Gather(gather)) if gather.spot.is_some());
        let by_knowledge = matches!(
            self.kind,
            OrderKind::Attack(_) | OrderKind::Move(Target::Object(_)) | OrderKind::Gather(_)
        );
        to_spot || (remembered && by_knowledge)
    }
}

/// The order a unit is carrying out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveOrder {
    pub order: Order,
    /// The tick it started.
    pub since: Tick,
    /// What its behaviour keeps from tick to tick (`systems/controls/behaviors.md > BaseBehaviorState`).
    pub state: Behaviour,
}

/// A unit's orders (`systems/controls/commands.md > Unit Command`): the current one, and the queue of those
/// that start, in order, when it completes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Orders {
    /// At most [`MAX_QUEUED_ORDERS`].
    pub queue: VecDeque<Order>,
    pub current: Option<ActiveOrder>,
    /// `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`: what the unit does on its own
    /// while it has no order. Every order that starts resets it.
    pub auto: Auto,
}

impl Orders {
    /// `systems/controls/commands.md > Unit Command`: with Shift (`queue`) the order is appended to the queue;
    /// without, it replaces the whole queue and the current order. Returns `false` if the queue was full and the
    /// order was dropped. The new order starts in phase 5a.
    pub fn give(&mut self, order: Order, queue: bool) -> bool {
        if !queue {
            self.queue.clear();
            self.current = None;
        }
        if self.queue.len() >= MAX_QUEUED_ORDERS {
            return false;
        }
        self.queue.push_back(order);
        true
    }

    /// Whether the unit has no order at all. It may still be fighting or returning by itself ([`Orders::auto`]).
    pub fn is_idle(&self) -> bool {
        self.current.is_none() && self.queue.is_empty()
    }

    /// The objects the behaviours' state refers to, besides the targets the orders themselves name: the enemies a
    /// unit engaged by itself.
    pub fn behaviour_targets(&self) -> Vec<ObjectId> {
        let mut ids = self.auto.targets();
        if let Some(active) = &self.current {
            ids.extend(active.state.targets());
        }
        ids
    }

    /// The removal path (`Sim::remove_objects`): the behaviours' state drops every reference to an object for which
    /// `is_gone` holds. Orders that name one are the removal path's own business.
    pub(crate) fn forget(&mut self, is_gone: &impl Fn(ObjectId) -> bool) {
        self.auto.forget(is_gone);
        if let Some(active) = &mut self.current {
            active.state.forget(is_gone);
        }
    }
}

#[cfg(test)]
mod tests;
