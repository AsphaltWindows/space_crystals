//! The `Orders` capability: a unit's command queue and the behaviour of its current order —
//! `systems/controls/commands.md > Unit Command`, `systems/controls/behaviors.md`, `docs/movement.md > 3`, `> 4`.
//!
//! A player's `Command::Order` (phase 4) only writes queues ([`Orders::give`]) and decides where each unit of a
//! group goes ([`destinations`], the magic box); it never moves anything. Phase 5a ([`update`]) completes orders,
//! starts the next one and runs the behaviours, which write `Mover::goal` and read `Mover::status`.
//!
//! **Behaviours** (slice E), each one tick at a time, on the snapshot of the game that [`world`] takes when the phase
//! begins:
//! - [`engage`]: AttackingObject (the approach, the chase and its ends), idle auto-targeting with its leash, and
//!   Hold Position;
//! - [`travel`]: MovingToObject (Follow for a unit), AttackMovingToLocation and Patrolling;
//! - Move to a point, and Stop, need only the mover's status.
//!
//! Their working state ([`state`], the design's BaseBehaviorState) is part of the order, or of the unit while it
//! has none ([`Orders::auto`]).

pub(crate) mod destinations;
mod engage;
pub mod state;
mod travel;
pub(crate) mod world;

use std::collections::{BTreeMap, VecDeque};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::nav::PathGroup;
use crate::object::Object;
use crate::units::Tick;

use state::{Approach, AttackOrder, Leg, Patrol};
pub use state::{Auto, Behaviour};
use world::{Unit, World};

/// `docs/movement.md > 2`, G13: orders waiting in one unit's queue, besides the current one. A technical cap: an
/// order queued beyond it is dropped for that unit.
pub const MAX_QUEUED_ORDERS: usize = 32;

/// Units one order command may name, duplicates included. Technical: selections have no size limit
/// (`systems/controls/selection_and_hud.md`), so this only bounds the work of one command.
pub const MAX_ORDER_UNITS: usize = 1024;

/// `systems/controls/behaviors.md > BaseAutoTargeting > Global Constant: IdleLeashDistance`: 4 grid units. An idle
/// unit farther than this from where it acquired its target disengages and returns.
pub const IDLE_LEASH_GU: f32 = 4.0;

/// `systems/controls/behaviors.md > AttackMovingToLocation > Global Constant: AttackMoveLeashDistance`: 6 grid
/// units, measured from the leg's planned path.
pub const ATTACK_MOVE_LEASH_GU: f32 = 6.0;

/// `systems/controls/behaviors.md > MovingToObject > Proximity` (design change 0032): "within proximity" is a gap
/// of at most 8 space units between the unit's Silhouette and the object's shape, edge to edge.
pub const PROXIMITY_SU: f32 = 8.0;

/// `systems/controls/behaviors.md > Patrolling > Failed legs` (design change 0037): this many failed legs in a row
/// end a Patrol.
pub const PATROL_FAILED_LEGS: u8 = 2;

/// A unit that acquired an enemy by itself (idle, or on an AttackMove or Patrol) and couldn't get to it doesn't
/// acquire that same enemy again for this many ticks, so that it doesn't set off for it again every tick.
/// Placeholder, unspecified: the design doesn't cover an auto-acquired target out of reach. 2 s.
pub const GIVE_UP_RETRY_TICKS: Tick = 32;

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
        }
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
pub(crate) fn update(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId, world: &World) {
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        let (Some(orders), Some(mover)) = (&mut o.orders, &mut o.mover) else {
            continue;
        };
        let mut unit = Unit {
            owner: o.owner,
            kind: o.kind,
            pos: o.pos,
            facing: o.facing,
            mover,
            attack: o.attack.as_mut(),
        };
        update_unit(orders, &mut unit, world);
    }
}

fn update_unit(orders: &mut Orders, unit: &mut Unit, world: &World) {
    // Each pass completes an order or starts one from the queue, which is bounded, so this ends.
    loop {
        if let Some(active) = &mut orders.current {
            if !run(active, unit, world) {
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
        let state = start(&order, unit, world.tick);
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
fn start(order: &Order, unit: &mut Unit, tick: Tick) -> Behaviour {
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
    }
}

/// One tick of the current order's behaviour. Returns whether it is complete, so that the next order may start.
fn run(active: &mut ActiveOrder, unit: &mut Unit, world: &World) -> bool {
    let order = &active.order;
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
        (OrderKind::Attack(id), Behaviour::Attack(state)) => engage::attack(unit, id, state, world),
        (OrderKind::Move(Target::Object(id)), Behaviour::Approach(state)) => travel::approach(unit, id, state, world),
        (OrderKind::AttackMove(click), Behaviour::AttackMove(leg)) => {
            travel::march(unit, leg, Some(path_group(order, click)), world) != travel::LegStatus::Running
        }
        (OrderKind::Patrol(click), Behaviour::Patrol(state)) => {
            travel::patrol(unit, state, path_group(order, click), world)
        }
        // A state that isn't its order's (a save edited by hand): the order ends.
        (OrderKind::Hold | OrderKind::Attack(_) | OrderKind::AttackMove(_) | OrderKind::Patrol(_), _)
        | (OrderKind::Move(Target::Object(_)), _) => true,
    }
}

impl StateHash for OrderGroup {
    fn hash(&self, h: &mut StateHasher) {
        self.0.hash(h);
    }
}

impl StateHash for Target {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Target::Point(p) => {
                h.tag(0);
                p.hash(h);
            }
            Target::Object(id) => {
                h.tag(1);
                id.hash(h);
            }
        }
    }
}

impl StateHash for OrderKind {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            OrderKind::Move(target) => {
                h.tag(0);
                target.hash(h);
            }
            OrderKind::Attack(id) => {
                h.tag(1);
                id.hash(h);
            }
            OrderKind::AttackMove(p) => {
                h.tag(2);
                p.hash(h);
            }
            OrderKind::Patrol(p) => {
                h.tag(3);
                p.hash(h);
            }
            OrderKind::Hold => h.tag(4),
            OrderKind::Stop => h.tag(5),
        }
    }
}

impl StateHash for Order {
    fn hash(&self, h: &mut StateHasher) {
        let Order { kind, group, slot } = self;
        kind.hash(h);
        group.hash(h);
        slot.hash(h);
    }
}

impl StateHash for ActiveOrder {
    fn hash(&self, h: &mut StateHasher) {
        let ActiveOrder { order, since, state } = self;
        order.hash(h);
        since.hash(h);
        state.hash(h);
    }
}

impl StateHash for Orders {
    fn hash(&self, h: &mut StateHasher) {
        let Orders { queue, current, auto } = self;
        queue.hash(h);
        current.hash(h);
        auto.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::AttackState;
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::map::Map;
    use crate::mover::Mover;
    use crate::vision::Vision;

    fn order(kind: OrderKind) -> Order {
        Order {
            kind,
            group: OrderGroup(0),
            slot: None,
        }
    }

    fn move_to(x: f32, y: f32) -> Order {
        order(OrderKind::Move(Target::Point(Vec2::new(x, y))))
    }

    /// `systems/controls/commands.md > Unit Command`.
    #[test]
    fn shift_appends_and_otherwise_the_order_replaces_the_queue() {
        let mut orders = Orders::default();
        assert!(orders.is_idle());
        assert!(orders.give(move_to(1.0, 1.0), true));
        assert!(orders.give(move_to(2.0, 2.0), true));
        orders.current = Some(ActiveOrder {
            order: move_to(0.0, 0.0),
            since: 1,
            state: Behaviour::Plain,
        });
        assert_eq!(orders.queue.len(), 2);
        assert!(orders.give(order(OrderKind::Stop), false));
        assert_eq!(orders.current, None);
        assert_eq!(orders.queue, [order(OrderKind::Stop)]);
    }

    #[test]
    fn the_queue_is_capped() {
        let mut orders = Orders::default();
        for i in 0..MAX_QUEUED_ORDERS {
            assert!(orders.give(move_to(i as f32, 0.0), true));
        }
        assert!(!orders.give(move_to(99.0, 0.0), true));
        assert_eq!(orders.queue.len(), MAX_QUEUED_ORDERS);
        // Replacing always fits.
        assert!(orders.give(move_to(99.0, 0.0), false));
        assert_eq!(orders.queue.len(), 1);
    }

    /// One tick of phase 5a for a lone Peacekeeper of player 1 at the origin of an empty map: no other object, so
    /// nothing to see, engage or follow.
    fn tick(orders: &mut Orders, mover: &mut Mover, attack: Option<&mut AttackState>, tick: Tick) {
        let map = Map::from_ascii("....\n....").unwrap();
        let vision = Vision::new(&map, [PlayerId(1)]);
        let world = World::new(&map, &vision, &BTreeMap::new(), tick, true);
        let mut unit = Unit {
            owner: Some(PlayerId(1)),
            kind: ObjectKind::Peacekeeper,
            pos: Vec2::new(40.0, 40.0),
            facing: 0.0,
            mover,
            attack,
        };
        update_unit(orders, &mut unit, &world);
    }

    #[test]
    fn moves_complete_on_arrival_and_the_next_order_starts_at_once() {
        let mut orders = Orders::default();
        let mut mover = Mover::default();
        orders.give(move_to(10.0, 0.0), true);
        orders.give(move_to(20.0, 0.0), true);
        tick(&mut orders, &mut mover, None, 1);
        assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(10.0, 0.0)));
        assert_eq!(orders.queue.len(), 1);
        // Still on the way: nothing changes.
        tick(&mut orders, &mut mover, None, 2);
        assert_eq!(orders.current.as_ref().map(|a| a.since), Some(1));
        mover.finish(MoveStatus::Arrived);
        tick(&mut orders, &mut mover, None, 3);
        assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(20.0, 0.0)));
        assert_eq!(orders.current.as_ref().map(|a| a.since), Some(3));
        mover.finish(MoveStatus::Failed);
        tick(&mut orders, &mut mover, None, 4);
        assert!(orders.is_idle());
        assert_eq!(mover.status, MoveStatus::Failed, "the last outcome stays readable");
    }

    #[test]
    fn stop_completes_at_once_and_hold_never_does() {
        let mut orders = Orders::default();
        let mut mover = Mover::default();
        orders.give(move_to(10.0, 0.0), true);
        tick(&mut orders, &mut mover, None, 1);
        orders.give(order(OrderKind::Stop), true);
        orders.give(order(OrderKind::Hold), false);
        orders.give(move_to(30.0, 0.0), true);
        tick(&mut orders, &mut mover, None, 2);
        assert_eq!(mover.goal, MoveGoal::None);
        assert_eq!(orders.current.as_ref().map(|a| a.order.kind), Some(OrderKind::Hold));
        for t in 3..50 {
            tick(&mut orders, &mut mover, None, t);
        }
        assert_eq!(orders.queue.len(), 1, "orders queued behind Hold wait");
        assert_eq!(mover.goal, MoveGoal::None);

        let mut stopping = Orders::default();
        stopping.give(order(OrderKind::Stop), true);
        stopping.give(move_to(5.0, 5.0), true);
        tick(&mut stopping, &mut mover, None, 60);
        assert_eq!(
            mover.goal,
            MoveGoal::Point(Vec2::new(5.0, 5.0)),
            "Stop completed in its tick"
        );
    }

    /// `systems/controls/behaviors.md > AttackingObject`: an Attack engages its target; a target that doesn't exist
    /// completes it, and the next order engages its own. Any other order disengages.
    #[test]
    fn attack_engages_its_target_and_another_order_disengages() {
        let mut mover = Mover::default();
        let mut attack = AttackState::default();
        let mut state = start(
            &order(OrderKind::Attack(ObjectId(3))),
            &mut unit(&mut mover, Some(&mut attack)),
            2,
        );
        assert_eq!(attack.target, Some(ObjectId(3)));
        assert_eq!(state, Behaviour::Attack(AttackOrder::default()));
        state = start(&move_to(5.0, 5.0), &mut unit(&mut mover, Some(&mut attack)), 3);
        assert_eq!(
            (attack.target, state),
            (None, Behaviour::Plain),
            "another order disengages"
        );
        // Nothing exists in this world: both Attacks complete at once, and the unit is idle with nothing engaged.
        let mut orders = Orders::default();
        orders.give(order(OrderKind::Attack(ObjectId(3))), true);
        orders.give(order(OrderKind::Attack(ObjectId(4))), true);
        tick(&mut orders, &mut mover, Some(&mut attack), 4);
        assert!(orders.is_idle());
        assert_eq!((attack.target, orders.auto), (None, Auto::default()));
        assert_eq!(mover.goal, MoveGoal::None);
    }

    fn unit<'a>(mover: &'a mut Mover, attack: Option<&'a mut AttackState>) -> Unit<'a> {
        Unit {
            owner: Some(PlayerId(1)),
            kind: ObjectKind::Peacekeeper,
            pos: Vec2::new(40.0, 40.0),
            facing: 0.0,
            mover,
            attack,
        }
    }

    /// A unit that can't attack (no `AttackState`) completes Attack at once; so does a Move to an object that is
    /// gone. AttackMove and Patrol set off.
    #[test]
    fn orders_without_a_target_or_an_attack_complete_and_the_others_set_off() {
        let mut orders = Orders::default();
        let mut mover = Mover::default();
        mover.go_to(MoveGoal::Point(Vec2::ONE), None, 1);
        for kind in [
            OrderKind::Move(Target::Object(ObjectId(3))),
            OrderKind::Attack(ObjectId(3)),
        ] {
            orders.give(order(kind), true);
        }
        tick(&mut orders, &mut mover, None, 2);
        assert!(orders.is_idle());
        assert_eq!(mover.goal, MoveGoal::None, "they stop the unit");
        let to = Vec2::new(100.0, 40.0);
        orders.give(order(OrderKind::AttackMove(to)), true);
        orders.give(order(OrderKind::Patrol(Vec2::new(40.0, 100.0))), true);
        tick(&mut orders, &mut mover, None, 3);
        assert_eq!(mover.goal, MoveGoal::Point(to));
        assert!(matches!(
            orders.current.as_ref().map(|a| &a.state),
            Some(Behaviour::AttackMove(leg)) if leg.to == to && leg.marching
        ));
        // The AttackMove arrives: the Patrol starts from here (design change 0036) and never completes.
        mover.finish(MoveStatus::Arrived);
        tick(&mut orders, &mut mover, None, 9);
        let Some(Behaviour::Patrol(patrol)) = orders.current.as_ref().map(|a| &a.state) else {
            panic!("the Patrol runs: {orders:?}");
        };
        assert_eq!((patrol.origin, patrol.outbound), (Vec2::new(40.0, 40.0), true));
        assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(40.0, 100.0)));
        // A leg arrives: the unit turns around in the same tick. A leg that fails counts as arrived (0037) ...
        mover.finish(MoveStatus::Arrived);
        tick(&mut orders, &mut mover, None, 20);
        assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(40.0, 40.0)));
        mover.finish(MoveStatus::Failed);
        tick(&mut orders, &mut mover, None, 30);
        assert_eq!(mover.goal, MoveGoal::Point(Vec2::new(40.0, 100.0)));
        // ... an arrival in between starts the count again ...
        mover.finish(MoveStatus::Arrived);
        tick(&mut orders, &mut mover, None, 40);
        mover.finish(MoveStatus::Failed);
        tick(&mut orders, &mut mover, None, 50);
        assert!(!orders.is_idle());
        // ... and two failed legs in a row end the Patrol.
        mover.finish(MoveStatus::Failed);
        tick(&mut orders, &mut mover, None, 60);
        assert!(orders.is_idle());
    }

    /// `systems/controls/commands.md > Group Destinations (Magic Box)`: a location order's own destination
    /// overrides the clicked point.
    #[test]
    fn a_units_own_destination_overrides_the_clicked_point() {
        let click = Vec2::new(1.0, 2.0);
        for kind in [
            OrderKind::Move(Target::Point(click)),
            OrderKind::AttackMove(click),
            OrderKind::Patrol(click),
        ] {
            let mut o = order(kind);
            assert_eq!(o.destination(), Some(click));
            o.slot = Some(Vec2::new(3.0, 4.0));
            assert_eq!(o.destination(), Some(Vec2::new(3.0, 4.0)));
        }
        assert_eq!(order(OrderKind::Hold).destination(), None);
        assert_eq!(order(OrderKind::Attack(ObjectId(1))).destination(), None);
    }
}
