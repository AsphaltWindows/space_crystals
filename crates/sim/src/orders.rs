//! The `Orders` capability: a unit's command queue and the behaviour of its current order —
//! `systems/controls/commands.md > Unit Command`, `systems/controls/behaviors.md`, `docs/movement.md > 3`, `> 4`.
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
//! - [`gather`]: a worker's job, the Gather and Drop Off orders of `factions/colonists/units.md > Drone`;
//! - [`deploy`]: a Drone's walk to the site of the Beacon it deploys into (`crate::deploy`);
//! - [`enter`]: a unit's walk into a Tunnel's entry path (`crate::network`).
//!
//! What a unit does by default against a target, a right-click's and a rally point's order, is [`resolution`]'s.
//!
//! Their working state ([`state`], the design's BaseBehaviorState) is part of the order, or of the unit while it
//! has none ([`Orders::auto`]).

mod deploy;
pub(crate) mod destinations;
mod engage;
mod enter;
pub(crate) mod gather;
mod hash;
pub(crate) mod reach;
pub mod resolution;
pub mod state;
mod travel;
pub(crate) mod world;

use std::collections::{BTreeMap, VecDeque};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::nav::PathGroup;
use crate::object::Object;
use crate::units::{Tick, TilePos};
use crate::{combat, conversion, riot_shield};

use reach::Reach;
pub use reach::{JUDGE_PATHS_PER_TICK, MAX_REJECTED, ReachCounters};
use state::{Approach, AttackOrder, Leg, Patrol};
pub use state::{Auto, Behaviour};
use world::{Unit, World};

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
    /// `> Move`.
    Move(Target),
    /// `> Attack`: a destructible object.
    Attack(ObjectId),
    /// `> AttackMove`: a point.
    AttackMove(Vec2),
    /// `> Patrol`: the far end of the patrol, a point.
    Patrol(Vec2),
    /// `> HoldPosition`.
    Hold,
    /// `> Stop`.
    Stop,
    /// `factions/colonists/units.md > Drone > Unit Commands > Gather`: a worker gathers from a Space Crystals patch
    /// and delivers to its AssignedDropOff, again and again ([`gather`]). The patch is the one the worker heads for
    /// now: the order follows it to another when its patch is occupied, out of reach or gone (design change 0132:
    /// "its gather target follows where it actually mined"). A patch that is gone stays named while the worker
    /// heads for where it was (design change 0134, `state::Gather::spot`).
    Gather(ObjectId),
    /// `> Unit Commands > Drop Off Resources`: a worker delivers the load it carries to this drop-off of its
    /// owner's, an Egress Node ([`gather`]).
    DropOff(ObjectId),
    /// `factions/colonists/economy.md > Beacon Deployment > Deploy Command`: a Drone walks to the site of a Beacon
    /// whose footprint's top-left tile is `origin` and deploys into it there (`crate::deploy`, [`deploy`]).
    Deploy { origin: TilePos },
    /// `> Enter`: "The unit walks to Side A and enters the Tunnel Network" (`factions/syndicate/tunnels.md > Tunnel
    /// Network > Walking in`; [`enter`], `crate::network`): this Tunnel of its owner's, through its entry path.
    Enter(ObjectId),
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
            OrderKind::Hold | OrderKind::Stop | OrderKind::Deploy { .. } => None,
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
            OrderKind::Enter(_) => None,
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

/// Phase 5a (`docs/movement.md > 4`): for every unit before `first_new`, in id order, complete the current order
/// if its behaviour is done, start queued orders until one is still running, and run that behaviour, or the idle
/// one, for the tick. The behaviours write the unit's goal (`Mover::goal`) and the target its attack engages
/// (`AttackState::target`), and read `Mover::status`, which phase 5d set last tick. Everything they read of other
/// objects comes from one snapshot, `world`, taken from `objects` just before: the positions the last tick left and
/// the vision of its phase 5f.
pub(crate) fn update(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId, world: &World, reach: &mut Reach) {
    reach.begin_tick();
    // `factions/colonists/units.md > Drone > Patch Occupancy (one Drone per patch)`: in id order within the tick.
    let mut claims = gather::Claims::default();
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let may_attack = riot_shield::allows_attack(o);
        let weapon = combat::weapon(o);
        let converting = conversion::is_converting(o);
        let (Some(orders), Some(mover)) = (&mut o.orders, &mut o.mover) else {
            continue;
        };
        let mut unit = Unit {
            id: o.id,
            owner: o.owner,
            kind: o.kind,
            pos: o.pos,
            facing: o.facing,
            mover,
            weapon,
            attack: o.attack.as_mut(),
            may_attack,
            converting,
            worker: o.worker.as_mut(),
        };
        reach.begin_unit();
        update_unit(orders, &mut unit, world, &mut claims, reach);
    }
}

fn update_unit(orders: &mut Orders, unit: &mut Unit, world: &World, claims: &mut gather::Claims, reach: &mut Reach) {
    // Each pass completes an order or starts one from the queue, which is bounded, so this ends.
    loop {
        if let Some(active) = &mut orders.current {
            if !run(active, &mut orders.auto, unit, world, claims, reach) {
                return;
            }
            orders.current = None;
        }
        let Some(order) = orders.queue.pop_front() else {
            // No order left: the unit keeps its last move's outcome, and looks after itself.
            engage::idle(unit, &mut orders.auto, world);
            return;
        };
        orders.auto = Auto::default();
        let state = start(&order, unit, world);
        orders.current = Some(ActiveOrder {
            order,
            since: world.tick,
            state,
        });
    }
}

/// The path group of `order`: its order group, anchored on the clicked point `click`, which the group's
/// destinations surround (`docs/movement.md > 5.5`).
fn path_group(order: &Order, click: Vec2) -> PathGroup {
    PathGroup {
        id: order.group.0,
        anchor: click,
    }
}

/// A behaviour begins (`systems/controls/behaviors.md`): its state as it starts. [`run`] follows in the same tick.
fn start(order: &Order, unit: &mut Unit, world: &World) -> Behaviour {
    let tick: Tick = world.tick;
    // `> AttackingObject`: the attack engages the order's target; every other order disengages it (interrupting an
    // attack being aimed, `systems/combat.md > Aiming`) and whatever the unit had acquired by itself.
    unit.engage(match order.kind {
        OrderKind::Attack(id) => Some(id),
        _ => None,
    });
    match order.kind {
        // `> MovingToLocation`: the unit goes to its own destination (the magic box); the group's path field is
        // anchored on the clicked point.
        OrderKind::Move(Target::Point(click)) => {
            let goal = MoveGoal::Point(order.slot.unwrap_or(click));
            unit.mover.go_to(goal, Some(path_group(order, click)), tick);
            return Behaviour::Plain;
        }
        // `> StoppingBehavior`, `> HoldingPosition`: no goal. Infinite deceleration stops the unit at once.
        OrderKind::Stop | OrderKind::Hold => {}
        // The others set off in their first tick ([`run`]), from a standstill.
        OrderKind::Attack(_) | OrderKind::Move(Target::Object(_)) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => {
        }
        OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } | OrderKind::Enter(_) => {}
    }
    unit.mover.stop();
    match order.kind {
        OrderKind::Move(Target::Point(_)) | OrderKind::Stop => Behaviour::Plain,
        OrderKind::Hold => Behaviour::Hold { target: None },
        OrderKind::Attack(_) => Behaviour::Attack(AttackOrder::default()),
        OrderKind::Move(Target::Object(_)) => Behaviour::Approach(Approach::default()),
        // `> AttackMovingToLocation`: to this unit's own destination.
        OrderKind::AttackMove(click) => Behaviour::AttackMove(Leg::new(unit.pos, order.slot.unwrap_or(click))),
        // `> Patrolling` 1 (design change 0036): the origin is where the unit is now, as the Patrol starts
        // executing; for a queued Patrol, where the order before it ended.
        OrderKind::Patrol(click) => {
            let destination = order.slot.unwrap_or(click);
            Behaviour::Patrol(Patrol {
                origin: unit.pos,
                destination,
                outbound: true,
                failed_legs: 0,
                leg: Leg::new(unit.pos, destination),
            })
        }
        // `factions/colonists/units.md > Drone > Gathering`: the worker's job.
        OrderKind::Gather(_) | OrderKind::DropOff(_) => Behaviour::Gather(gather::begin(order.kind, unit, world)),
        // `factions/colonists/economy.md > Beacon Deployment > Deploy Command`: to the footprint's centre.
        OrderKind::Deploy { origin } => Behaviour::Deploy(deploy::begin(origin, unit, tick)),
        // `systems/controls/commands.md > Enter`: to the Tunnel's entry path.
        OrderKind::Enter(tunnel) => Behaviour::Enter(enter::begin(tunnel, unit, world)),
    }
}

/// One tick of the current order's behaviour. Returns whether it is complete, so that the next order may start.
/// `auto` is what the unit does on its own once it has no order: an order that ends may leave it something
/// (`engage::attack`: the target it gave up).
fn run(
    active: &mut ActiveOrder,
    auto: &mut Auto,
    unit: &mut Unit,
    world: &World,
    claims: &mut gather::Claims,
    reach: &mut Reach,
) -> bool {
    let order = &mut active.order;
    match (order.kind, &mut active.state) {
        // `> MovingToLocation`: complete when stopped at the target, or after failing to make progress
        // (`> Failure`).
        (OrderKind::Move(Target::Point(_)), _) => unit.mover.status != MoveStatus::Moving,
        // `> StoppingBehavior`: complete when stopped, at once with infinite deceleration.
        (OrderKind::Stop, _) => true,
        // `> HoldingPosition`: never completes; only a new command ends it.
        (OrderKind::Hold, Behaviour::Hold { target }) => {
            engage::hold(unit, target, world);
            false
        }
        (OrderKind::Attack(id), Behaviour::Attack(state)) => engage::attack(unit, id, state, auto, world),
        (OrderKind::Move(Target::Object(id)), Behaviour::Approach(state)) => travel::approach(unit, id, state, world),
        (OrderKind::AttackMove(click), Behaviour::AttackMove(leg)) => {
            travel::march(unit, leg, Some(path_group(order, click)), world, reach) != travel::LegStatus::Running
        }
        (OrderKind::Patrol(click), Behaviour::Patrol(state)) => {
            travel::patrol(unit, state, path_group(order, click), world, reach)
        }
        // A worker's job; the behaviour moves the order on to another patch as it must. Orders queued behind a
        // Gather wait until it ends on its own (design change 0123, `gather`).
        (OrderKind::Gather(_) | OrderKind::DropOff(_), Behaviour::Gather(state)) => {
            gather::run(&mut order.kind, state, unit, world, claims)
        }
        // The walk to the site; the step right after this phase ends the order (`Sim::step_deployments`).
        (OrderKind::Deploy { origin }, Behaviour::Deploy(state)) => deploy::run(origin, state, unit),
        // The walk into the entry path; the step right after this phase takes the unit into the network or ends the
        // order (`Sim::step_entries`).
        (OrderKind::Enter(_), Behaviour::Enter(state)) => enter::run(state, unit),
        // A state that isn't its order's (a save edited by hand): the order ends.
        (OrderKind::Hold | OrderKind::Attack(_) | OrderKind::AttackMove(_) | OrderKind::Patrol(_), _)
        | (OrderKind::Move(Target::Object(_)) | OrderKind::Gather(_) | OrderKind::DropOff(_), _)
        | (OrderKind::Deploy { .. } | OrderKind::Enter(_), _) => true,
    }
}

#[cfg(test)]
mod tests;
