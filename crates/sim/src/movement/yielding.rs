//! Friendly yielding — `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (design changes
//! 0022, 0023 and 0027), `docs/movement.md > 6.6`.
//!
//! A moving unit blocked by an **idle** unit of its own owner asks it to make way ([`ask`]), and the idle unit takes
//! **one short sidestep** ([`sidestep`]): a real move through the same collision predicates as every other
//! ([`World::is_free`]), so collision stays hard. Enemies never yield, and moving units never yield or swap.
//!
//! - **Who yields** ([`can_yield`], [`yields_to`]): 0023 and 0027.
//! - **Requests** are collected while the units move, in id order, and applied when all have moved ([`apply`]): the
//!   first request a unit gets wins, so the lowest id that asked. The sidestep starts in the next tick, in the
//!   unit's own turn.
//! - **(G) Walking through a group**: a mover aims between the units of an idle group in its way (`World::lane`),
//!   and those it is about to walk into are asked a moment before, both sides at once, to step straight across
//!   its way ([`ask_ahead`]). The group parts instead of being shoved along.
//! - **Dense idle groups**: a sidestep that another idle unit of the owner blocks passes the request on, the same
//!   way, at most `YIELD_CHAIN_DEPTH` units deep. A unit that can't get out of the way stands, and the mover treats
//!   it as an ordinary blocker (`docs/movement.md > 6.5`).

use std::collections::BTreeMap;

use glam::Vec2;

use super::collide::{Moved, World};
use super::space::Body;
use crate::combat::AttackPhase;
use crate::defs::UnitDef;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::{
    Locomotion, MoveGoal, Mover, Orientation, Side, Sidestep, YIELD_CHAIN_DEPTH, YIELD_MIN_SIDESTEP_SU,
    YIELD_SIDESTEP_WIDTHS,
};
use crate::object::Object;

/// A sidestep asked of a unit this tick.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Request {
    /// The unit asked, an index into the step's bodies.
    yielder: usize,
    dir: Vec2,
    dist: f32,
    depth: u8,
    for_group: Option<u32>,
}

/// `systems/combat.md > AttackPhase`: whether `o` is in an attack sequence that keeps it busy: Aiming, Firing or in
/// Cooldown. While Reloading it is free to do other things. (For a unit that attacks from its base this is
/// `!combat::may_move`; a turret unit attacking from where it stands is busy too, design change 0023.)
fn in_attack_sequence(o: &Object) -> bool {
    let phase = o.attack.as_ref().and_then(|attack| attack.phase);
    matches!(
        phase,
        Some(AttackPhase::Aiming { .. } | AttackPhase::Firing { .. } | AttackPhase::Cooldown)
    )
}

/// `> Friendly yielding > Who yields` (design changes 0022 and 0023): whether `o` is a unit that steps aside for a
/// blocked unit of its owner.
/// - It has an owner: nobody's units never yield.
/// - It has **no active command**: no current order and none queued. Hold Position is a command, so units on Hold
///   never yield (deliberate walls stay possible), and neither does a unit that is moving.
/// - It isn't busy with a stationary task: not in an attack sequence ([`in_attack_sequence`]). Mining and
///   constructing will be orders.
/// - **Unit modes don't matter**, but it must be able to move in its current mode (`Mover::can_move`, the hook for a
///   deployed Artillery Rig).
/// - It isn't already taking a sidestep: one short sidestep per request.
/// - (M3-D2) It isn't walking out of a portal's energy doorway (`Mover::exit_heading`, design change 0145): that walk
///   is its only move until it is out (`doorway::emerge`).
pub(super) fn can_yield(o: &Object, mover: &Mover, unit: &UnitDef) -> bool {
    o.owner.is_some()
        && o.orders.as_ref().is_some_and(|orders| orders.is_idle())
        && mover.goal == MoveGoal::None
        && mover.sidestep.is_none()
        && mover.exit_heading.is_none()
        && !in_attack_sequence(o)
        && mover.can_move(unit.movement.max_speed())
}

/// Whether `other` yields to a moving unit of `owner` that moves under the order group `group`.
/// - `> Friendly yielding`: only to its own owner's units (no teams yet, so no allies; TBD in the design).
/// - `> No yielding within the same group order` (design change 0027): a unit that arrived under a group order
///   doesn't yield to members of the same order that are still arriving; a later order clears this
///   (`Body::arrived_with`).
pub(super) fn yields_to(other: &Body, owner: Option<PlayerId>, group: Option<u32>) -> bool {
    other.yields && owner.is_some() && other.owner == owner && !(group.is_some() && other.arrived_with == group)
}

/// The longest sidestep of a unit with half extents `half`: `YIELD_SIDESTEP_WIDTHS` of its silhouette.
fn longest(half: Vec2) -> f32 {
    (half.max_element() * 2.0 * YIELD_SIDESTEP_WIDTHS).max(YIELD_MIN_SIDESTEP_SU)
}

/// Friendly yielding (0022): an idle unit whose centre is within this many space units of the line a mover walks
/// along stands "in the middle" of its way, and steps to the side the mover isn't going round on; farther out it
/// steps to the side it is already on (tuning const).
const YIELD_CENTRED_SU: f32 = 4.0;

/// `> Friendly yielding` (design change 0022): the unit of `world` wanted to move `d` from `pos`, and `moved` is
/// what became of it. If a unit that yields to it was in its way, that unit is asked for a sidestep:
/// - **Direction**: half across the mover's way and half along it, so the unit gets out of the lane and away from
///   the mover at once. A sidestep straight across the way left a line of movers facing a group as wide as itself
///   with nowhere to send it: each unit stepped into its neighbour's lane. Across the way it goes to the side of
///   the mover's line it already stands on; in the middle of the way ([`YIELD_CENTRED_SU`]), to the side the mover
///   isn't going round on (the side it stepped to this tick, else its `Mover::avoid_side`), so the two never chase
///   each other.
/// - **(G) Straight across** instead, with no step along the way, when the mover only clips the unit (its line
///   passes beside the unit's box) or walks between the units of a group (`in_lanes`: `World::lane` gave it a
///   lane). Half the step along the way is what shoved an idle group ahead of a group walking through it, a few
///   widths at a time; a mover that aims between two units needs each a little to its side, and nothing more.
/// - **Distance**: as far as takes its box out of the mover's lane, at most `YIELD_SIDESTEP_WIDTHS` of its own
///   width. A mover still blocked when the sidestep is over asks again.
pub(super) fn ask(
    world: &World,
    pos: Vec2,
    d: Vec2,
    moved: &Moved,
    in_lanes: bool,
    mover: &Mover,
    requests: &mut Vec<Request>,
) {
    let Some(index) = moved.blocker else {
        return;
    };
    let (me, other) = (world.body(), &world.bodies[index]);
    if !yields_to(other, me.owner, mover.group) {
        return;
    }
    let Some(ahead) = d.try_normalize() else {
        return;
    };
    // The mover's Right: clockwise on screen.
    let right = Vec2::new(-ahead.y, ahead.x);
    let offset = (other.pos - pos).dot(right);
    let side = if offset.abs() > YIELD_CENTRED_SU {
        offset.signum()
    } else {
        let stepped = (moved.pos - pos).dot(right);
        let going_right = stepped > 0.0 || (stepped == 0.0 && mover.avoid_side == Side::Right);
        if going_right { -1.0 } else { 1.0 }
    };
    // The lane of the mover's box, widened by the other's: half its width across the way.
    let reach = me.half + other.half;
    let lane = right.x.abs() * reach.x + right.y.abs() * reach.y;
    let across = lane - side * offset;
    let own = right.x.abs() * other.half.x + right.y.abs() * other.half.y;
    let clipped = in_lanes || offset.abs() >= own;
    let (dir, dist) = if clipped {
        (right * side, across)
    } else {
        // At 45° to the way, the step is √2 times what it gains across it.
        (
            (right * side + ahead) * core::f32::consts::FRAC_1_SQRT_2,
            across * core::f32::consts::SQRT_2,
        )
    };
    requests.push(Request {
        yielder: index,
        dir,
        dist: dist.clamp(YIELD_MIN_SIDESTEP_SU, longest(other.half)),
        depth: 0,
        for_group: mover.group,
    });
}

/// (G) `> Friendly yielding`, for a mover walking between the units of an idle group (`World::lane`, whose lanes
/// `world` still holds; `shift` is the line the mover is taking, to the Right of its way `d`): the idle units of
/// its owner that it is about to be blocked by, within two steps, are asked now, both sides of its gap at once,
/// instead of one after the other as each stops it. Each steps **straight across** its way, away from the mover's
/// line, as far as its box reaches into the mover's lane. So the group parts, and nobody is shoved along. A unit
/// exactly on the line is left to [`ask`], which knows the side the mover goes round on.
pub(super) fn ask_ahead(world: &World, d: Vec2, shift: f32, mover: &Mover, requests: &mut Vec<Request>) {
    let step = d.length();
    let Some(ahead) = d.try_normalize() else {
        return;
    };
    let right = Vec2::new(-ahead.y, ahead.x);
    for lane in world.lanes.iter() {
        let (centre, reach) = lane.centre_and_reach();
        let offset = centre - shift;
        if !lane.standing || lane.free > 2.0 * step || offset == 0.0 || offset.abs() >= reach {
            continue;
        }
        let other = &world.bodies[lane.index as usize];
        requests.push(Request {
            yielder: lane.index as usize,
            dir: right * offset.signum(),
            dist: (reach - offset.abs()).clamp(YIELD_MIN_SIDESTEP_SU, longest(other.half)),
            depth: 0,
            for_group: mover.group,
        });
    }
}

/// One tick of a yielding unit's sidestep (`> Friendly yielding`): up to its speed along the sidestep's direction,
/// the whole step if it is free, else each axis as far as tiles and units allow. Its facing stays (every M1 unit
/// turns in place; TODO(movement models): bases that can't step sideways). An idle unit of its owner in its way is
/// asked on, the same way ([`YIELD_CHAIN_DEPTH`]). The sidestep ends when its distance is done or its time is up;
/// the unit stays where it got to, with no way back (design change 0022: one short sidestep).
pub(super) fn sidestep(
    pos: &mut Vec2,
    mover: &mut Mover,
    speed: f32,
    world: &mut World,
    touched: &mut Vec<usize>,
    requests: &mut Vec<Request>,
) {
    let Some(mut step) = mover.sidestep else {
        return;
    };
    let d = step.dir * speed.min(step.left);
    touched.clear();
    let at = world.shuffle(*pos, d, touched);
    let went = at - *pos;
    *pos = at;
    step.left -= went.dot(step.dir);
    step.ticks_left = step.ticks_left.saturating_sub(1);
    if went != d && step.depth < YIELD_CHAIN_DEPTH {
        let owner = world.body().owner;
        for &index in touched.iter() {
            let other = &world.bodies[index];
            if yields_to(other, owner, step.for_group) {
                requests.push(Request {
                    yielder: index,
                    dir: step.dir,
                    dist: step.left.clamp(YIELD_MIN_SIDESTEP_SU, longest(other.half)),
                    depth: step.depth + 1,
                    for_group: step.for_group,
                });
            }
        }
    }
    // Less than a hundredth of a space unit left is the rounding of the steps, not a distance to go.
    if step.left < 0.01 || step.ticks_left == 0 {
        // Like an arrival: the unit stands from this tick on.
        mover.sidestep = None;
        mover.velocity = Vec2::ZERO;
        mover.locomotion = Locomotion::Stationary;
    } else {
        mover.sidestep = Some(step);
        mover.velocity = went;
        mover.locomotion = if went == Vec2::ZERO {
            Locomotion::Stationary
        } else {
            Locomotion::Moving
        };
    }
    mover.orientation = Orientation::Maintaining;
    mover.blocked_ticks = 0;
}

/// Applies the tick's requests once every unit has moved, in the order they were made: movers ask in id order, so
/// of several requests to one unit the lowest id's wins. The unit must still be free to yield.
pub(super) fn apply(objects: &mut BTreeMap<ObjectId, Object>, bodies: &[Body], requests: &mut Vec<Request>) {
    for request in requests.drain(..) {
        let Some(o) = objects.get_mut(&bodies[request.yielder].id) else {
            continue;
        };
        let Some(unit) = &o.kind.def().unit else {
            continue;
        };
        let free = o.mover.as_ref().is_some_and(|mover| can_yield(o, mover, unit));
        if let Some(mover) = o.mover.as_mut().filter(|_| free) {
            let speed = unit.movement.max_speed() * mover.speed_factor;
            mover.sidestep = Some(Sidestep::new(
                request.dir,
                request.dist,
                speed,
                request.depth,
                request.for_group,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ObjectKind;
    use crate::orders::{Order, OrderGroup, OrderKind};
    use crate::scenario::SpawnAt;
    use crate::sim::Sim;
    use crate::testing::ScenarioBuilder;

    fn peacekeeper(owner: Option<PlayerId>) -> Object {
        let builder = ScenarioBuilder::new("....\n....").spawn(
            ObjectKind::Peacekeeper,
            owner,
            SpawnAt::Point(Vec2::new(100.0, 64.0)),
        );
        Sim::new(&builder.build()).objects().next().unwrap().clone()
    }

    fn yields(o: &Object) -> bool {
        can_yield(o, o.mover.as_ref().unwrap(), o.kind.def().unit.as_ref().unwrap())
    }

    /// 0022 and 0023, Who yields: idle units with an owner; not on Hold or under any order, not while moving, not in
    /// an attack sequence, not if they can't move. A slowing mode (Riot Shield) doesn't matter.
    #[test]
    fn only_idle_units_that_can_move_yield() {
        let idle = peacekeeper(Some(PlayerId(1)));
        assert!(yields(&idle));
        assert!(!yields(&peacekeeper(None)), "nobody's unit");

        let order = |kind| Order {
            kind,
            group: OrderGroup(1),
            slot: None,
        };
        let mut hold = idle.clone();
        hold.orders.as_mut().unwrap().current = Some(crate::orders::ActiveOrder {
            order: order(OrderKind::Hold),
            since: 1,
            state: crate::orders::Behaviour::Hold { target: None },
        });
        assert!(!yields(&hold), "Hold Position is a command");
        let mut queued = idle.clone();
        queued.orders.as_mut().unwrap().give(order(OrderKind::Stop), true);
        assert!(!yields(&queued));

        let mut moving = idle.clone();
        moving
            .mover
            .as_mut()
            .unwrap()
            .go_to(MoveGoal::Point(Vec2::ONE), None, 1);
        assert!(!yields(&moving));

        for (phase, busy) in [
            (AttackPhase::Aiming { target: ObjectId(9) }, true),
            (AttackPhase::Firing { target: None }, true),
            (AttackPhase::Cooldown, true),
            (AttackPhase::Reloading, false),
        ] {
            let mut attacking = idle.clone();
            attacking.attack.as_mut().unwrap().phase = Some(phase);
            assert_eq!(yields(&attacking), !busy, "{phase:?}");
        }

        // 0023: modes don't matter, unless the unit can't move in them (the hook for a deployed Artillery Rig).
        let mut shielded = idle.clone();
        shielded.mover.as_mut().unwrap().speed_factor = 0.5;
        assert!(yields(&shielded), "a raised Riot Shield isn't a task");
        let mut deployed = idle.clone();
        deployed.mover.as_mut().unwrap().speed_factor = 0.0;
        assert!(!yields(&deployed), "a unit that can't move is a fixed obstacle");

        let mut stepping = idle.clone();
        stepping.mover.as_mut().unwrap().sidestep = Some(Sidestep::new(Vec2::X, 24.0, 4.0, 0, None));
        assert!(!yields(&stepping), "one sidestep per request");
    }

    /// 0022: only to the owner's units. 0027: not to the group it arrived with.
    #[test]
    fn a_unit_yields_to_its_owners_units_but_not_to_the_group_it_arrived_with() {
        let body = |owner, arrived_with| Body {
            id: ObjectId(1),
            owner,
            pos: Vec2::ZERO,
            half: Vec2::splat(12.0),
            velocity: Vec2::ZERO,
            heading: Vec2::ZERO,
            stationary: true,
            progress: None,
            arrived_with,
            yields: true,
            gathers: false,
            leads: true,
        };
        let (p1, p2) = (Some(PlayerId(1)), Some(PlayerId(2)));
        assert!(yields_to(&body(p1, None), p1, Some(3)));
        assert!(!yields_to(&body(p1, None), p2, Some(3)), "enemies never yield");
        assert!(!yields_to(&body(None, None), None, None));
        assert!(!yields_to(&body(p1, Some(3)), p1, Some(3)), "its own group order");
        assert!(yields_to(&body(p1, Some(3)), p1, Some(4)), "another order");
        assert!(yields_to(&body(p1, Some(3)), p1, None));
        let busy = Body {
            yields: false,
            gathers: false,
            leads: true,
            ..body(p1, None)
        };
        assert!(!yields_to(&busy, p1, None));
    }
}
