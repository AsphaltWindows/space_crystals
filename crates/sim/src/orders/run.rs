//! Phase 5a (`docs/movement.md > 4`), the orders' part of the tick ([`update`]): each unit's current order completes
//! once its behaviour is done, its queued orders start until one is still running, and the behaviour of that one, or
//! the idle one, runs for the tick (`crate::orders`, module docs).

use std::collections::BTreeMap;

use glam::Vec2;

use super::reach::Reach;
use super::state::{Approach, AttackGround, AttackOrder, Auto, Behaviour, Leg, Patrol};
use super::world::{Unit, World};
use super::{
    ActiveOrder, Order, OrderKind, Orders, Target, assist, attack_ground, build, engage, enter, gather, travel,
};
use crate::deploy;
use crate::ids::ObjectId;
use crate::mover::{MoveGoal, MoveStatus};
use crate::nav::PathGroup;
use crate::object::Object;
use crate::units::Tick;
use crate::{combat, conversion, riot_shield};

/// Phase 5a (`docs/movement.md > 4`): for every unit before `first_new`, in id order, complete the current order
/// if its behaviour is done, start queued orders until one is still running, and run that behaviour, or the idle
/// one, for the tick. The behaviours write the unit's goal (`Mover::goal`) and the target its attack engages
/// (`AttackState::target`), and read `Mover::status`, which phase 5d set last tick. Everything they read of other
/// objects comes from one snapshot, `world`, taken from `objects` just before: the positions the last tick left and
/// the vision of its phase 5f.
pub(crate) fn update(objects: &mut BTreeMap<ObjectId, Object>, first_new: ObjectId, world: &World, reach: &mut Reach) {
    reach.begin_tick();
    // `factions/colonists/units/drone.md > Patch Occupancy (one Drone per patch)`: in id order within the tick.
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

pub(super) fn update_unit(
    orders: &mut Orders,
    unit: &mut Unit,
    world: &World,
    claims: &mut gather::Claims,
    reach: &mut Reach,
) {
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
pub(super) fn start(order: &Order, unit: &mut Unit, world: &World) -> Behaviour {
    let tick: Tick = world.tick;
    // `systems/controls/command_behaviors.md > AttackingObject`: the attack engages the order's target; every other
    // order disengages it (interrupting an attack being aimed, `systems/combat.md > Aiming`) and whatever the unit had
    // acquired by itself.
    // (M4-O) An Attack Ground engages its location instead (`systems/controls/command_behaviors.md >
    // AttackingLocation`).
    match order.kind {
        OrderKind::AttackGround(at) => unit.engage_ground(at),
        OrderKind::Attack(id) => unit.engage(Some(id)),
        _ => unit.engage(None),
    }
    match order.kind {
        // `systems/controls/command_behaviors.md > MovingToLocation`: the unit goes to its own destination (the magic
        // box); the group's path field is anchored on the clicked point.
        OrderKind::Move(Target::Point(click)) => {
            let goal = MoveGoal::Point(order.slot.unwrap_or(click));
            unit.mover.go_to(goal, Some(path_group(order, click)), tick);
            return Behaviour::Plain;
        }
        // `systems/controls/command_behaviors.md > StoppingBehavior`, `systems/controls/command_behaviors.md >
        // HoldingPosition`: no goal. Infinite deceleration stops the unit at once.
        OrderKind::Stop | OrderKind::Hold => {}
        // The others set off in their first tick ([`run`]), from a standstill.
        OrderKind::Attack(_) | OrderKind::Move(Target::Object(_)) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) => {
        }
        OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } | OrderKind::Enter(_) => {}
        OrderKind::AttackGround(_) => {}
        OrderKind::Build { .. } | OrderKind::Assist(_) => {}
    }
    unit.mover.stop();
    match order.kind {
        OrderKind::Move(Target::Point(_)) | OrderKind::Stop => Behaviour::Plain,
        OrderKind::Hold => Behaviour::Hold { target: None },
        OrderKind::Attack(_) => Behaviour::Attack(AttackOrder::default()),
        OrderKind::Move(Target::Object(_)) => Behaviour::Approach(Approach::default()),
        // `systems/controls/command_behaviors.md > AttackMovingToLocation`: to this unit's own destination.
        OrderKind::AttackMove(click) => Behaviour::AttackMove(Leg::new(unit.pos, order.slot.unwrap_or(click))),
        // `systems/controls/command_behaviors.md > Patrolling` 1 (design change 0036): the origin is where the unit is
        // now, as the Patrol starts executing; for a queued Patrol, where the order before it ended.
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
        // `factions/colonists/units/drone.md > Gathering`: the worker's job.
        OrderKind::Gather(_) | OrderKind::DropOff(_) => Behaviour::Gather(gather::begin(order.kind, unit, world)),
        // `factions/colonists/economy.md > Beacon Deployment > Deploy Command`: to the footprint's centre.
        OrderKind::Deploy { origin } => Behaviour::Deploy(build::begin(&deploy::site(origin), unit, tick)),
        // `core/entities.md > Worker-Built Structures`: to the footprint's centre too (plan decision T51). A kind
        // without a footprint (only a save edited by hand has one) has no walk: the order ends in its first tick.
        OrderKind::Build { .. } => match build::build_site(order.kind) {
            Some((_, site)) => Behaviour::Build(build::begin(&site, unit, tick)),
            None => Behaviour::Plain,
        },
        // `systems/controls/commands.md > Enter`: to the Tunnel's entry path.
        OrderKind::Enter(tunnel) => Behaviour::Enter(enter::begin(tunnel, unit, world)),
        // `systems/controls/command_behaviors.md > AttackingLocation`: from a standstill, as an Attack.
        OrderKind::AttackGround(_) => Behaviour::AttackGround(AttackGround::default()),
        // (M5-E) `factions/cults/economy.md > Assist Construction Command`: to its own spot beside the site, which the
        // spots' step gives it (`crate::spot_work`).
        OrderKind::Assist(_) => Behaviour::Assist(assist::begin()),
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
        // `systems/controls/command_behaviors.md > MovingToLocation`: complete when stopped at the target, or after
        // failing to make progress (`systems/controls/command_behaviors.md > Failure`).
        (OrderKind::Move(Target::Point(_)), _) => unit.mover.status != MoveStatus::Moving,
        // `systems/controls/command_behaviors.md > StoppingBehavior`: complete when stopped, at once with infinite
        // deceleration.
        (OrderKind::Stop, _) => true,
        // `systems/controls/command_behaviors.md > HoldingPosition`: never completes; only a new command ends it.
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
        (OrderKind::Deploy { origin }, Behaviour::Deploy(state)) => build::run(&deploy::site(origin), state, unit),
        (OrderKind::Build { .. }, Behaviour::Build(state)) => match build::build_site(order.kind) {
            Some((_, site)) => build::run(&site, state, unit),
            None => true,
        },
        // The walk into the entry path; the step right after this phase takes the unit into the network or ends the
        // order (`Sim::step_entries`).
        (OrderKind::Enter(_), Behaviour::Enter(state)) => enter::run(state, unit),
        // (M4-O) The walk into range and the shots, until another order (design change 0293).
        (OrderKind::AttackGround(at), Behaviour::AttackGround(state)) => attack_ground::run(unit, at, state, world),
        // (M5-E) Building from its spot; the site's completion or removal ends the order (`assist`).
        (OrderKind::Assist(_), Behaviour::Assist(state)) => assist::run(state),
        // A state that isn't its order's (a save edited by hand): the order ends.
        (OrderKind::Hold | OrderKind::Attack(_) | OrderKind::AttackMove(_) | OrderKind::Patrol(_), _)
        | (OrderKind::Move(Target::Object(_)) | OrderKind::Gather(_) | OrderKind::DropOff(_), _)
        | (OrderKind::Deploy { .. } | OrderKind::Enter(_) | OrderKind::AttackGround(_), _)
        | (OrderKind::Build { .. } | OrderKind::Assist(_), _) => true,
    }
}
