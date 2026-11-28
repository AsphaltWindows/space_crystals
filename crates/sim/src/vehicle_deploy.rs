//! The vehicle deploy model — plan decisions T44 (M4 plan) and T87 (M6 plan), slice M6-GA: a unit of a pair of types
//! becomes the pair's other type in place, by an order that takes its type's frames and can't be cancelled. The
//! Artillery Rig's Deploy and Undeploy (`factions/gdo/units/artillery_rig.md > Modes`, design changes 0211, 0213, 0214
//! and 0263): "**Deploy is an order, not a mode switch** [...]: with a mode switch the unit's type changes
//! **instantly** and the unit catches up in its own time. With deployment, **the deploying process is what actually
//! changes the unit**: it stays the mobile type until the process completes." Kept apart from the toggles
//! (`crate::toggle`: the Riot Shield, a Weapon Conversion), from a transformation (`crate::transform`) and from the
//! Drone's Deploy (`crate::deploy`: "a Drone becomes a Beacon **immediately** and the transformation **can be
//! cancelled**").
//!
//! - **Data, not code**: each type of a pair has a row, `UnitDef::vehicle_deploy` ([`VehicleDeploy`]: the type it
//!   becomes, `into`, and its frames). Nothing here names a type: the Artillery Rig's two rows make it deploy (48 frames
//!   into the Artillery Rig (Deployed)) and undeploy (48 frames back); slice M6-SL's Laser Battery and slice M6-ST2's
//!   Transport are a row on each of their types.
//! - **The order** (`OrderKind::VehicleDeploy`, the panel's R: "**R: Deploy** (mobile) / **R: Undeploy** (deployed)"):
//!   "**Queueable**: Shift-Deploy waits until the earlier orders are done (e.g. move here, then deploy). Without Shift
//!   it replaces the current orders and starts at once." It names the type the unit becomes, so that only the units
//!   that deploy into it take it ([`can_take`]: a Deploy pressed twice is no Undeploy).
//! - **While it deploys** ([`is_deploying`]; "the unit can neither move nor attack while switching"): from its start in
//!   phase 5a (`orders::vehicle_deploy`) the unit stands where it stopped, with no goal (a vehicle with momentum brakes
//!   to a stop first, as at any order that stops it: `Mover::stop`); it engages nothing, its turret locks nothing and
//!   picks nothing (it can't attack now, `orders::world::Unit::attrs`), so phase 5e starts no attack sequence for it, a
//!   shot already under way landing as it would; and with an order it yields to no one and nothing pushes it
//!   (`movement::yielding`). It stays its type ("It stays the mobile type until the process completes"), fully
//!   attackable.
//! - **Not cancellable** (design change 0213; "once deploying has started it always runs to completion; Stop doesn't
//!   interrupt it"): an order given meanwhile goes behind it (`Orders::give`, `Sim::drop_orders`): "the **last order
//!   given** (without Shift; each new one replaces the previous) is **held**, and it is the first thing the unit tries
//!   once deployment finishes. Shift-queued orders line up behind it as usual" (design change 0214). A Stop is held as
//!   any order: it takes the held orders off, and the deploy goes on.
//! - **The gate** (`Sim::check_order`; T44: "the gate skips its per-unit capability filters for a deploying unit; its
//!   target checks still apply"): design change 0214's "Orders arriving during deploying are stored, not executed or
//!   rejected. When deploying completes, they are validated against the deployed type." A unit that deploys takes an
//!   order whatever its type can do now, so that the mobile Rig holds an Attack it will carry out deployed, but a
//!   vehicle deploy ([`Reject::Deploying`]: "**Undeploy is only available once deployment is complete**"). Every other
//!   unit takes an order its type can carry out ([`can_take`]), which adds to the gate's own rules that a unit whose
//!   type can't move ignores an order that would take it somewhere ([`moves`], [`immobile`]: "one it can't carry out
//!   (e.g. **Move**, since a deployed unit can't move)").
//! - **Completion** (the step right after the Modes step, before phase 5a, `Sim::step_vehicle_deploys`; T44): the
//!   frame that is the deploy's last done, the unit becomes a unit of type `into` (`Sim::change_kind`: the same object,
//!   its orders and its move kept, its attack the new type's, a `SimEvent::KindChanged`, which moves it to its new
//!   SelectionGroup on the client, "with separate SelectionGroups"), its deploy order ends, and each order of its queue
//!   is checked against the new type ([`can_take`], the gate's rule): one it can't carry out is dropped, with a
//!   `SimEvent::OrderDropped`, which the client says as "the usual unavailable-command feedback". The unit acts as its
//!   new type from that tick's first phase on, its held order first.
//! - **Timing** (as `docs/architecture/economy.md > Economy > Recipe timing` counts frames): a deploy of N frames given in tick `t`
//!   to a unit with nothing before it starts in phase 5a of `t`; the unit deploys in ticks `t` to `t + N - 1` as its old
//!   type, and is its new type from tick `t + N` on, whose step completes the deploy.
//! - **A site** (plan decision T87, slice M6-GP's Prospector: "the order's first half is `deploy.rs`'s Drone
//!   pipeline [...]; its second half is T44's in-place 64 frames; completion is the unit-to-structure `change_kind`"):
//!   not built. Its place is the order's start, where a deploy that needs a site would walk to it first and check it
//!   on arrival, and its completion, where the type it becomes would be a structure placed on that site
//!   (`Sim::step_vehicle_deploys`).

use crate::augmentation::Research;
use crate::combat;
use crate::defs::{Domain, ObjectKind, VehicleDeploy};
use crate::economy::RATIO_ONE;
use crate::object::Object;
use crate::orders::{Behaviour, OrderKind};
use crate::progress::Progress;
use crate::sim::Reject;
use crate::{aircraft, carry, tether};

/// The deploy of a `kind` unit, its def's (`UnitDef::vehicle_deploy`): the type it becomes and the frames that takes.
/// `None` for a type that doesn't deploy.
pub fn def_of(kind: ObjectKind) -> Option<VehicleDeploy> {
    kind.def().unit.as_ref()?.vehicle_deploy
}

/// Whether a `kind` unit deploys into a unit of type `into`: what a vehicle deploy order asks of the units it goes to.
pub fn deploys_into(kind: ObjectKind, into: ObjectKind) -> bool {
    def_of(kind).is_some_and(|deploy| deploy.into == into)
}

/// Whether `o` deploys now (module docs): its current order is a vehicle deploy that has started (`Orders::deploying`).
pub fn is_deploying(o: &Object) -> bool {
    o.orders.as_ref().is_some_and(|orders| orders.deploying())
}

/// `o`'s deploy under way, if it deploys: the type it becomes and the frames done of its type's, which the client shows
/// (its InfoPanel's line and its R's bar).
pub fn under_way(o: &Object) -> Option<(ObjectKind, Progress)> {
    let active = o.orders.as_ref()?.current.as_ref()?;
    match (active.order.kind, &active.state) {
        (OrderKind::VehicleDeploy { into }, Behaviour::VehicleDeploy(progress)) => Some((into, *progress)),
        _ => None,
    }
}

/// Whether a `kind` unit can't move at all, by its def (plan decision T87: "The deployed kind is immobile by its def,
/// so "can't move" and "never yields" follow from the existing rules"): its MaxSpeed is 0. The Artillery Rig
/// (Deployed) ("**Deployed:** cannot move") and slice M6-SK's Laser Battery (Deployed). `false` for a type that isn't a
/// unit.
pub fn immobile(kind: ObjectKind) -> bool {
    kind.def()
        .unit
        .as_ref()
        .is_some_and(|unit| unit.movement.max_speed() <= 0.0)
}

/// Whether an order of `kind` takes its unit somewhere, its point being where the unit goes: a unit that can't move
/// can't carry one out ([`immobile`]; `factions/gdo/units/artillery_rig.md > Modes`: "one it can't carry out (e.g.
/// **Move**, since a deployed unit can't move)"). A Move, to a point or to an object, an Attack Move, a Patrol and a
/// Reverse; a worker's jobs, an Enter, and (M6-GH1) an aircraft's flight to its Airfield. Not an Attack nor an Attack
/// Ground, which a unit in range carries out where it stands (an immobile unit's with its target inside its MinRange
/// fails: `systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange`, design change 0182), nor a
/// Hold or a Stop, nor the orders carried out where the unit stands. **Reading**: the design names the Move alone; an
/// Attack Move and a Patrol are moves too, whose attacks are on the way.
pub fn moves(kind: OrderKind) -> bool {
    match kind {
        OrderKind::Move(_) | OrderKind::AttackMove(_) | OrderKind::Patrol(_) | OrderKind::Reverse(_) => true,
        OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } | OrderKind::Enter(_) => true,
        OrderKind::Build { .. } | OrderKind::Assist(_) | OrderKind::Repair(_) => true,
        OrderKind::Rebase(_) | OrderKind::ReturnToBase => true,
        // A chopper's flight to a spot.
        OrderKind::PickUpSupplies(_) | OrderKind::DropOffSupplies(_) | OrderKind::AttachToTower(_) => true,
        OrderKind::Attack(_) | OrderKind::AttackGround(_) | OrderKind::Hold | OrderKind::Stop => false,
        OrderKind::Shroud | OrderKind::TargetLock(_) | OrderKind::Launch | OrderKind::VehicleDeploy { .. } => false,
        // (M6-ST1) A carrier unloads where it comes to rest.
        OrderKind::UnloadAll => false,
        // A thrust's burst (`crate::thrust`).
        OrderKind::Thrust(_) => true,
    }
}

/// T44's one `can_take(object, order)`: why `o`, as it is now, can't carry out an order of `kind`, if it can't; `target`
/// is the domain of an Attack's target as `o`'s owner knows it (a structure stands on the ground), and `research` its
/// owner's levels, which its weapon follows (`combat::weapon`). The gate asks it of every unit an order goes to but the
/// ones that deploy (`Sim::check_order`), and a completed deploy of each order its unit held or queued
/// (`Sim::step_vehicle_deploys`), so that both judge an order by the same rules: those of the unit on its own that a
/// change of type can break, each the gate's.
/// - **A vehicle deploy**: its type deploys into the type the order names, else [`Reject::LacksAbility`]; and it doesn't
///   deploy now, else [`Reject::Deploying`] ("**Undeploy is only available once deployment is complete**").
/// - **An Attack**: its type takes Attack orders (`combat::takes_attack_orders`, the mobile Rig's doesn't: "has no
///   attack"), else [`Reject::CannotAttack`]; its weapon targets the target's domain (`systems/combat.md > ValidTarget`
///   3, the deployed Rig's shell "TargetDomain: Ground"), else [`Reject::CannotHit`]; and (M6-GH1) an aircraft has ammo
///   left, else [`Reject::OutOfAmmo`].
/// - **An Attack Ground**: its weapon can target the ground (`combat::can_target_ground`), else
///   [`Reject::CannotTargetGround`]; with ammo, as an Attack.
/// - **A Reverse**: it can back up (`UnitDef::can_reverse`), else [`Reject::CannotReverse`].
/// - (M6-ST1) **An Unload All**: it is a carrier that unloads now (`carry::loads`: the Subterranean Transport above
///   ground, "**F: Unload All** (above ground only; they come out around it)"), else [`Reject::LacksAbility`].
/// - **An order that takes it somewhere** ([`moves`]): its type can move ([`immobile`]), else [`Reject::CannotMove`];
///   (M6-GD) and its drone isn't out (`tether::holds`), else [`Reject::DroneOut`].
///
/// The rules of the other orders, a worker's, a caster's, a launcher's, an aircraft's flight home, are the gate's
/// alone (`Sim::check_order`): no pair's types differ in them, neither of the Rig's being a worker, a caster, a
/// launcher or an aircraft.
pub fn can_take(o: &Object, kind: OrderKind, target: Option<Domain>, research: Research) -> Result<(), Reject> {
    match kind {
        OrderKind::VehicleDeploy { into } => {
            if !deploys_into(o.kind, into) {
                return Err(Reject::LacksAbility);
            }
            if is_deploying(o) {
                return Err(Reject::Deploying);
            }
        }
        OrderKind::Attack(id) => {
            if !combat::takes_attack_orders(o.kind) {
                return Err(Reject::CannotAttack);
            }
            let weapon = combat::weapon(o, research);
            if let Some(domain) = target
                && !combat::can_hit(weapon.map(|attrs| attrs.target_domain), domain)
            {
                return Err(Reject::CannotHit(id));
            }
            if !aircraft::armed(o) {
                return Err(Reject::OutOfAmmo);
            }
        }
        OrderKind::AttackGround(_) => {
            let weapon = combat::weapon(o, research);
            if !weapon.is_some_and(|attrs| combat::can_target_ground(attrs.attack_type)) {
                return Err(Reject::CannotTargetGround);
            }
            if !aircraft::armed(o) {
                return Err(Reject::OutOfAmmo);
            }
        }
        OrderKind::Reverse(_) if !o.def().unit.as_ref().is_some_and(|unit| unit.can_reverse()) => {
            return Err(Reject::CannotReverse);
        }
        OrderKind::UnloadAll if !carry::loads(o) => return Err(Reject::LacksAbility),
        _ => {}
    }
    if moves(kind) && immobile(o.kind) {
        return Err(Reject::CannotMove);
    }
    if !tether::allows(kind) && tether::holds(o) {
        return Err(Reject::DroneOut);
    }
    Ok(())
}

/// The state a deploy of a `kind` unit starts with, as its order starts in phase 5a (`orders::vehicle_deploy`): none of
/// its type's frames done. A unit of a type that doesn't deploy has none to do (a save edited by hand): its deploy
/// completes at the next step, into the type its order names.
pub(crate) fn begin(kind: ObjectKind) -> Progress {
    Progress::new(def_of(kind).map_or(0, |deploy| deploy.frames))
}

/// One frame of `o`'s deploy, if it deploys (the step before phase 5a, `Sim::step_vehicle_deploys`): the type it becomes
/// once this frame was the last of its deploy's, `None` while frames are left or for a unit that doesn't deploy.
pub(crate) fn step(o: &mut Object) -> Option<ObjectKind> {
    let active = o.orders.as_mut()?.current.as_mut()?;
    match (active.order.kind, &mut active.state) {
        (OrderKind::VehicleDeploy { into }, Behaviour::VehicleDeploy(progress)) => {
            progress.step(RATIO_ONE).then_some(into)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
