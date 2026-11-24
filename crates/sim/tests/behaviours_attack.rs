//! The Attack order (`systems/controls/behaviors.md > AttackingObject`): the approach until the target is in range
//! (measured edge to edge, design change 0031), the chase when it moves, the walk to its last seen position when it
//! goes into the fog (design change 0034), and giving up a target out of reach.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Shape, SimEvent, Target, Tick, Vec2, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick, Range 4, Sight 5
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5

/// 40 × 12 open tiles: 2560 × 768 su.
const OPEN: &str = "
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
    ........................................
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn pos(t: &TestSim, n: u32) -> Vec2 {
    t.sim().object(id(n)).expect("exists").pos
}

fn is_idle(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).unwrap().orders.as_ref().unwrap().is_idle()
}

fn current(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The gap between the shapes of two objects (`systems/combat.md > Range Measurement`).
fn gap(t: &TestSim, a: u32, b: u32) -> f32 {
    let shape = |n| Shape::of(t.sim().object(id(n)).expect("exists"));
    shape(a).gap(&shape(b))
}

/// The ticks in which `attacker`'s attacks took effect.
fn attack_ticks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == id(attacker)))
        .map(|&(tick, _)| tick)
        .collect()
}

/// An attacker of player 1 (id 1) at (200, 300), a Guard of player 2 (id 2) holding its position at (900, 300), and
/// a scout of player 1 (id 3), also on Hold, that sees the Guard from beyond everyone's Range. The attacker is
/// ordered to attack the Guard in tick 2.
fn approach(attacker: ObjectKind) -> TestSim {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, attacker, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(900.0, 300.0))
        .spawn_unit_at(1, PK, P1, v(900.0, 600.0))
        .order_at(2, P2, &[id(2)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(3)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    t
}

/// Design change 0031's Verification, with today's units: an attacker walks up until the **gap between the edges**
/// is its Range and stops there, not when its centre is that far from the target's centre or its edge; and a wider
/// attacker's own body doesn't eat into its range. The Peacekeeper (Range 4) stops with a gap of 4 grid units to
/// the Guard, the Guard (Range 3) with a gap of 3 to its target.
#[test]
fn an_attacker_stops_when_the_gap_between_the_edges_is_its_range() {
    for (attacker, range, step) in [(PK, 4.0, 4.0), (GUARD, 3.0, 5.0)] {
        let mut t = approach(attacker);
        t.run_until(200);
        let stopped = gap(&t, 1, 2);
        assert!(
            stopped <= gu(range) && stopped > gu(range) - step,
            "{attacker:?} stopped at a gap of {stopped}"
        );
        assert_eq!(pos(&t, 1).y, 300.0, "straight at it");
        assert_eq!(t.status(1), MoveStatus::Idle, "it stands while it attacks");
        assert_eq!(current(&t, 1), Some(OrderKind::Attack(id(2))));
        let shots = attack_ticks(&t, 1);
        assert!(!shots.is_empty(), "and attacks from there");
        // It aims in the tick it comes into range: the first hit lands two ticks after its last step.
        let arrival = ((gap_at_start(attacker) - gu(range)) / step).ceil() as Tick + 1;
        assert_eq!(shots[0], arrival + 2, "{attacker:?}");
        assert_eq!(pos(&t, 2), v(900.0, 300.0), "the Guard on Hold never moved");
    }
}

/// The gap between the attacker of [`approach`] and the Guard when they are spawned.
fn gap_at_start(attacker: ObjectKind) -> f32 {
    let half = attacker.def().half_extents().unwrap().x;
    900.0 - 18.0 - (200.0 + half)
}

/// `> AttackingObject` 4 and 5: the target moves out of range, and the unit goes after it, re-aiming its path as
/// the target moves; it attacks whenever it is in range, and the order completes when the target is destroyed. A
/// Guard (5 su per tick, Range 3) hunts a Peacekeeper (4 su per tick) that walks away and then turns south.
#[test]
fn an_attacker_chases_a_moving_target_until_it_is_destroyed() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, GUARD, P1, v(200.0, 300.0))
        .spawn_unit_at(1, PK, P2, v(450.0, 300.0))
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false)
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(1400.0, 300.0))), false)
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(1400.0, 700.0))), true);
    let mut farthest = 0.0_f32;
    t.run_until(1);
    loop {
        assert!(t.sim().tick() < 1500, "the hunt never ends\n{}", t.ascii(None));
        t.step();
        if t.sim().object(id(2)).is_none() {
            break;
        }
        if t.sim().tick() > 60 {
            farthest = farthest.max(gap(&t, 1, 2));
        }
        assert_eq!(current(&t, 1), Some(OrderKind::Attack(id(2))), "the order runs on");
    }
    // The Guard closes in at 1 su per tick; once it has caught up, the target is never far out of its Range.
    assert!(farthest <= gu(3.0) + 16.0, "the target got {farthest} su away");
    let hunter = pos(&t, 1);
    assert!(
        hunter.x > 1000.0 && hunter.y > 320.0,
        "it followed around the corner: {hunter}"
    );
    assert!(is_idle(&t, 1), "the target is destroyed: the behaviour is complete");
    assert!(attack_ticks(&t, 1).len() >= 6, "{:?}", attack_ticks(&t, 1));
}

/// Design change 0034's Verification: "Attack a fast enemy that runs out of sight: your unit runs to where it
/// vanished and stops." A Peacekeeper attacks a Guard that runs east, faster than it. The Guard leaves player 1's
/// sight; the Peacekeeper walks to the Guard's last seen position, and its order completes there. It doesn't
/// follow the Guard through the fog.
#[test]
fn a_target_lost_in_the_fog_is_followed_to_its_last_seen_position() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(500.0, 300.0))
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false)
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(2400.0, 300.0))), false);
    let mut last_seen = None;
    let mut lost_at = None;
    t.run_until(2);
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 600, "the order never completes\n{}", t.ascii(None));
        t.step();
        let guard = t.sim().object(id(2)).unwrap();
        if lost_at.is_none() {
            if t.sim().is_visible(P1, guard) {
                last_seen = Some(guard.pos);
            } else {
                lost_at = Some(t.sim().tick());
            }
        }
    }
    let (last_seen, lost_at) = (last_seen.unwrap(), lost_at.expect("the Guard got away"));
    // Sight goes by tiles: the Guard is lost as soon as its box has left the last tile whose centre is in sight.
    assert!(last_seen.x > 500.0 && last_seen.x < 600.0, "{last_seen}");
    assert_eq!(pos(&t, 1), last_seen, "it stands where the Guard vanished");
    assert_eq!(t.status(1), MoveStatus::Arrived);
    assert!(t.sim().tick() > lost_at + 20, "it walked there after losing it");
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0], "the Guard was never in range");
    // The Guard is long gone: the unit stays, idle.
    let guard = pos(&t, 2);
    assert!(guard.x > last_seen.x + gu(5.0), "{guard}");
    let stands = pos(&t, 1);
    t.run(100);
    assert_eq!(pos(&t, 1), stands);
    assert_eq!(t.sim().object(id(1)).unwrap().attack.as_ref().unwrap().target, None);
}

/// Design change 0034: "Once stopped it is idle, and normal idle auto-targeting applies if the enemy is visible
/// again." The Guard stops not far beyond where it vanished. The Peacekeeper sees it again on its way, but walks
/// on to the last seen position without a shot; there its order is over and, idle, it engages the Guard on its own.
#[test]
fn from_the_last_seen_position_idle_auto_targeting_takes_over() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(500.0, 300.0))
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false)
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(760.0, 300.0))), false)
        // It holds there, so that it doesn't come back for the Peacekeeper by itself.
        .order_at(2, P2, &[id(2)], OrderKind::Hold, true);
    let mut completed = None;
    let mut lost = false;
    t.run_until(2);
    while t.sim().tick() < 200 {
        t.step();
        lost |= !t.sim().is_visible(P1, t.sim().object(id(2)).unwrap());
        if completed.is_none() && is_idle(&t, 1) {
            completed = Some(t.sim().tick());
            assert!(lost, "the Guard went out of sight first");
            assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0], "no shot under the order");
        }
    }
    let completed = completed.expect("the Attack order completed");
    let shots = attack_ticks(&t, 1);
    assert!(
        shots.first().is_some_and(|&first| first > completed),
        "it attacked only after its order was over, on its own: {shots:?} after {completed}"
    );
    assert!(is_idle(&t, 1));
    assert!(gap(&t, 1, 2) <= gu(4.0));
}

/// A moat four tiles wide with an island beyond it.
const MOAT: &str = "
    ..........~~~~......
    ..........~~~~......
    ..........~~~~......
    ..........~~~~......
    ..........~~~~......
    ..........~~~~......
    ..........~~~~......
    ..........~~~~......
";

/// `> AttackingObject > Failure`, `> MovingToLocation > Unreachable target location`: a target the unit can't get
/// within range of. A Guard (Range 3) is ordered to attack a Peacekeeper on the far side of a moat 4 tiles wide:
/// it walks to the point nearest to it, at the water's edge, can't get any closer, and its order completes. It
/// doesn't walk off again for the same target every tick.
#[test]
fn a_target_out_of_reach_ends_the_order_at_the_nearest_point() {
    let mut t = ScenarioBuilder::new(MOAT).start();
    t.spawn_unit_at(1, GUARD, P1, v(200.0, 200.0))
        .spawn_unit_at(1, PK, P2, v(940.0, 200.0))
        // A scout at the shore sees the target; both hold, out of each other's range.
        .spawn_unit_at(1, GUARD, P1, v(620.0, 260.0))
        .order_at(2, P1, &[id(3)], OrderKind::Hold, false)
        .order_at(2, P2, &[id(2)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false);
    t.run_until(2);
    assert_eq!(
        current(&t, 1),
        Some(OrderKind::Attack(id(2))),
        "the gate took it: the scout sees it"
    );
    let mut completed = None;
    while completed.is_none() {
        assert!(t.sim().tick() < 400, "the order never completes\n{}", t.ascii(None));
        t.step();
        if is_idle(&t, 1) {
            completed = Some(t.sim().tick());
        }
    }
    let shore = pos(&t, 1);
    assert_eq!(shore.x, 640.0 - 18.0, "at the water's edge: {shore}");
    assert!(gap(&t, 1, 2) > gu(3.0));
    // Idle, it sees the target and would acquire it; out of reach, it stays where it is.
    let requests_before = t.events().len();
    t.run(200);
    assert_eq!(pos(&t, 1), shore);
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0]);
    assert!(
        t.events()[requests_before..]
            .iter()
            .all(|(_, e)| !matches!(e, SimEvent::MoveFailed { .. })),
        "no move fails over it"
    );
}

/// A queued Attack whose target is out of sight when its turn comes has no last seen position to go to: the order
/// completes at once (a placeholder, the design doesn't cover it). The gate took it while the target was in sight.
#[test]
fn a_queued_attack_on_a_target_that_left_sight_before_its_turn_completes_at_once() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(480.0, 300.0))
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(2400.0, 300.0))), false)
        .order_at(2, P1, &[id(1)], OrderKind::Move(Target::Point(v(200.0, 700.0))), false)
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), true);
    t.run_until(2);
    let orders = t.sim().object(id(1)).unwrap().orders.clone().unwrap();
    assert_eq!(orders.queue.len(), 1, "queued while the Guard was in sight");
    t.run_until(300);
    assert!(is_idle(&t, 1));
    assert_eq!(pos(&t, 1), v(200.0, 700.0), "it stayed where its move ended");
}

/// A corridor one tile (64 su) wide and 26 long, y from 64 to 128: two Guards (36 su) can't pass each other in it.
/// Tile 14 (x 896 to 960) is Rugged.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~~~~~~~~~~~
    ..............r...........
    ~~~~~~~~~~~~~~~~~~~~~~~~~~
";

/// `docs/movement.md > 6.5`, rule 2 (queueing): a unit chasing the target of its attack, blocked by a unit of its
/// owner that stands attacking the same target, keeps waiting: no re-route, no failure, however long the fight
/// lasts. Two Guards attack a Peacekeeper down a corridor; the second can't get into range past the first, which
/// fights for well over the 104 ticks after which a stuck move gives up (on Rugged ground the Peacekeeper takes 2
/// a hit). When the target is destroyed, both orders complete. (Auto-targeting is off, so the target doesn't fight
/// back and the scout that sees it doesn't join in.)
#[test]
fn an_attacker_queues_behind_its_owners_attackers_of_the_same_target() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, v(300.0, 96.0))
        .spawn_unit_at(1, GUARD, P1, v(200.0, 96.0))
        .spawn_unit_at(1, PK, P2, v(928.0, 96.0))
        .spawn_unit_at(1, PK, P1, v(1150.0, 96.0)) // the scout
        .order_at(2, P1, &[id(1), id(2)], OrderKind::Attack(id(3)), false);
    t.run_until(2);
    assert_eq!(current(&t, 2), Some(OrderKind::Attack(id(3))));
    let mut waited = 0;
    loop {
        assert!(t.sim().tick() < 600, "the fight never ends\n{}", t.ascii(None));
        t.step();
        if t.sim().object(id(3)).is_none() {
            break;
        }
        assert_eq!(
            current(&t, 2),
            Some(OrderKind::Attack(id(3))),
            "tick {}",
            t.sim().tick()
        );
        // Behind the first Guard, touching it, and out of range.
        if gap(&t, 1, 2) == 0.0 && gap(&t, 2, 3) > gu(3.0) {
            waited += 1;
        }
    }
    assert!(waited > 150, "it queued for {waited} ticks");
    assert!(gap(&t, 1, 2) == 0.0, "still right behind it");
    assert_eq!(attack_ticks(&t, 2), [] as [Tick; 0], "it never got into range");
    assert_eq!(t.mover(2).repaths_left, sc_sim::mover::MAX_REPATHS, "no re-route");
    assert!(
        t.events()
            .iter()
            .all(|(_, e)| !matches!(e, SimEvent::MoveFailed { .. })),
        "and no failure"
    );
    t.step();
    assert!(
        is_idle(&t, 1) && is_idle(&t, 2),
        "the target is destroyed: both orders are complete"
    );
}
