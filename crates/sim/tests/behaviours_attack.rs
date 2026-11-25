//! The Attack order (`systems/controls/behaviors.md > AttackingObject`): the approach until the target is in range
//! (measured edge to edge, design change 0031), the chase when it moves, the stop where the unit stands when the
//! target goes into the fog (design change 0200; more in `target_lost.rs`), and giving up a target out of reach.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tuning::movement::{GIVE_UP_TICKS, REPATH_WAITS, STUCK_GRACE_TICKS};
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
/// the Guard, the Guard (Range 3) with a gap of 3 to its target. (Two Guards in range of each other trade hits of 17
/// until both fall, shortly before tick 200: tick 150 sees them fighting.)
#[test]
fn an_attacker_stops_when_the_gap_between_the_edges_is_its_range() {
    for (attacker, range, step) in [(PK, 4.0, 4.0), (GUARD, 3.0, 5.0)] {
        let mut t = approach(attacker);
        t.run_until(150);
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
        // It aims in the tick it comes into range: the first hit lands its AimDuration after its last step (2 ticks
        // for the Peacekeeper, 4 for the Guard).
        let arrival = ((gap_at_start(attacker) - gu(range)) / step).ceil() as Tick + 1;
        let aim = attacker.def().unit.as_ref().unwrap().attack.unwrap().aim_ticks;
        assert_eq!(shots[0], arrival + Tick::from(aim), "{attacker:?}");
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
    // Three hits of 17 (design change 0186) take the Peacekeeper's 50 HP.
    assert_eq!(attack_ticks(&t, 1).len(), 3, "{:?}", attack_ticks(&t, 1));
}

/// Design change 0200's Verification (answering question Q0051; it replaces design change 0034's walk to the last
/// seen position): "A Peacekeeper attacks a fleeing enemy that walks out of sight. It stops right there, with no walk
/// into the fog." The Guard runs east, faster than the Peacekeeper, which goes after it. In the tick after the Guard
/// leaves player 1's sight (behaviours read the vision of the tick before), the order completes, "exactly as when the
/// target is destroyed": the Peacekeeper stands where it was as the Guard vanished, idle, engaging nothing.
///
/// Slice M3-G2's case, folded in (`behaviours_soak`'s seed 17): the Guard then leaves the game, and the Peacekeeper
/// stands on. Under 0034 it was on its walk to the last seen position then, which the removal path left running:
/// it went on there without an order.
#[test]
fn a_target_lost_in_the_fog_ends_the_attack_where_the_unit_stands() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(500.0, 300.0))
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false)
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(2400.0, 300.0))), false);
    let mut lost = None;
    t.run_until(2);
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 600, "the order never completes\n{}", t.ascii(None));
        t.step();
        if lost.is_none() && !t.sim().is_visible(P1, t.sim().object(id(2)).unwrap()) {
            lost = Some((t.sim().tick(), pos(&t, 1)));
        }
    }
    let (lost_at, there) = lost.expect("the Guard got away");
    assert_eq!(t.sim().tick(), lost_at + 1, "it completes at once");
    assert!(there.x > 200.0, "it went after the Guard while it saw it: {there}");
    assert_eq!(pos(&t, 1), there, "it stops right there");
    assert_eq!(t.status(1), MoveStatus::Idle);
    assert_eq!(t.sim().object(id(1)).unwrap().attack.as_ref().unwrap().target, None);
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0], "the Guard was never in range");
    // The Guard runs on through the fog; the unit stays, idle.
    t.run(100);
    assert_eq!(pos(&t, 1), there, "no walk into the fog");
    assert!(pos(&t, 2).x > there.x + gu(10.0), "{}", pos(&t, 2));
    let gone = t.sim().tick() + 1;
    t.debug_at(gone, sc_sim::DebugCommand::Remove { id: id(2) });
    t.run(60);
    assert!(t.sim().object(id(2)).is_none(), "the Guard left the game");
    assert!(is_idle(&t, 1));
    assert_eq!((pos(&t, 1), t.status(1)), (there, MoveStatus::Idle), "it stands on");
}

/// Design change 0200's Verification: "When the enemy steps back into view near the idle Peacekeeper, it is engaged"
/// ("Once idle, normal idle auto-targeting (Idle behavior, with IdleLeashDistance) applies if the enemy is visible
/// again"). The Guard runs out of sight, and the Peacekeeper's Attack ends where it stands. The Guard stops beyond
/// its sight, and the Peacekeeper doesn't go looking for it; once the Guard walks back into its sight, the idle
/// Peacekeeper engages it on its own, and fires.
#[test]
fn an_enemy_back_in_view_is_engaged_by_the_idle_unit() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(500.0, 300.0))
        .order_at(2, P1, &[id(1)], OrderKind::Attack(id(2)), false)
        .order_at(2, P2, &[id(2)], OrderKind::Move(Target::Point(v(1400.0, 300.0))), false);
    t.run_until(2);
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 300, "the order never completes\n{}", t.ascii(None));
        t.step();
    }
    let (completed, there) = (t.sim().tick(), pos(&t, 1));
    t.run_until(completed + 200);
    assert_eq!(pos(&t, 1), there, "the Guard stopped out of its sight");
    assert!(!t.sim().is_visible(P1, t.sim().object(id(2)).unwrap()));
    // The Guard walks back, into the Peacekeeper's sight and Range, and holds there.
    let (back, at) = (Target::Point(there + v(12.0 + 220.0 + 18.0, 0.0)), completed + 201);
    t.order_at(at, P2, &[id(2)], OrderKind::Move(back), false)
        .order_at(at, P2, &[id(2)], OrderKind::Hold, true);
    let engaged = loop {
        assert!(t.sim().tick() < completed + 600, "it never engaged the Guard");
        t.step();
        if t.sim().object(id(1)).unwrap().attack.as_ref().unwrap().target == Some(id(2)) {
            break t.sim().tick();
        }
    };
    assert!(is_idle(&t, 1), "on its own");
    assert!(t.sim().is_visible(P1, t.sim().object(id(2)).unwrap()));
    t.run(60);
    let shots = attack_ticks(&t, 1);
    assert!(
        shots.first().is_some_and(|&first| first > engaged),
        "it fired once the Guard was back: {shots:?} after {engaged}"
    );
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

/// A queued Attack whose target, a unit, is out of its owner's sight when its turn comes completes at once, as one
/// lost from vision does (design change 0200; a placeholder reading, since the target left sight before the order
/// began, which the design doesn't cover). The gate took it while the target was in sight.
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

/// `docs/movement.md > 6.5`, rule 2 (queueing; slice M3-G3, design change 0223): a unit chasing the target of its
/// attack, blocked by a unit of its owner that stands attacking the same target, looks for a way round the unit
/// ahead, and a corridor has none. Those attempts are the leg's three, and then it gives up as any stuck unit does
/// (`systems/controls/behaviors.md > AttackingObject > Blocked by its own side`, `> MovingToLocation > Failure`): its
/// move fails about 7 s after it was blocked, and its Attack completes. Until 0223 it waited for as long as the
/// fight lasted. (`tests/attack_behind_friends` has the cases with room, and more without.)
///
/// Two Guards attack a Peacekeeper down a corridor; the second can't get into range past the first, which fights for
/// longer than the 104 ticks after which a stuck move gives up (on Rugged ground the Peacekeeper takes 8 a hit, 18 ×
/// 0.5 − 1: seven hits, one every 24 ticks). Auto-targeting is off, so the target doesn't fight back, the scout that
/// sees it doesn't join in, and the second Guard, idle, doesn't take the target up again: it stays where it gave up.
/// When the target is destroyed, the first Guard's order completes.
#[test]
fn an_attacker_blocked_behind_its_owners_attacker_of_the_same_target_gives_up() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GUARD, P1, v(300.0, 96.0))
        .spawn_unit_at(1, GUARD, P1, v(200.0, 96.0))
        .spawn_unit_at(1, PK, P2, v(928.0, 96.0))
        .spawn_unit_at(1, PK, P1, v(1150.0, 96.0)) // the scout
        .order_at(2, P1, &[id(1), id(2)], OrderKind::Attack(id(3)), false);
    t.run_until(2);
    assert_eq!(current(&t, 2), Some(OrderKind::Attack(id(3))));
    let (mut blocked, mut failed, mut stood) = (None, None, None);
    loop {
        assert!(t.sim().tick() < 600, "the fight never ends\n{}", t.ascii(None));
        t.step();
        let tick = t.sim().tick();
        // The order's own two paths are planned in tick 2, and none after it: the attempts at a way round ask for
        // none, and the second Guard, idle, sets out for nothing.
        let planned = t.sim().nav_counters().searches + t.sim().nav_counters().fields;
        assert!(tick <= 2 || planned == 0, "a path in tick {tick}");
        if t.sim().object(id(3)).is_none() {
            break;
        }
        assert_eq!(current(&t, 1), Some(OrderKind::Attack(id(3))), "tick {tick}");
        // Behind the first Guard, touching it, and out of range.
        if blocked.is_none() && gap(&t, 1, 2) == 0.0 && gap(&t, 2, 3) > gu(3.0) {
            blocked = Some(tick);
        }
        let events = t.sim().events();
        if events
            .iter()
            .any(|e| matches!(e, SimEvent::MoveFailed { unit } if *unit == id(2)))
        {
            assert_eq!(failed, None, "it gives up once");
            assert_eq!(
                t.mover(2).repaths_left,
                0,
                "it looked for a way round, and there is none"
            );
            failed = Some(tick);
        }
        match (failed, stood) {
            (Some(failed), None) if tick > failed => {
                assert!(is_idle(&t, 2), "tick {tick}: its Attack completes once its move failed");
                stood = Some(pos(&t, 2));
            }
            (_, Some(at)) => assert_eq!(pos(&t, 2), at, "tick {tick}: idle, it stays where it gave up"),
            _ => assert_eq!(current(&t, 2), Some(OrderKind::Attack(id(3))), "tick {tick}"),
        }
    }
    let (blocked, failed) = (blocked.expect("it was blocked"), failed.expect("it gives up"));
    // No sooner than the waits before the leg's three attempts, and no later than the Failure rule allows. Measured:
    // 84 ticks (blocked in tick 104; its attempts 12, 28 and 60 ticks after, as the unit ahead has no goal to get
    // on to, and the give-up 24 after the last).
    let after = failed - blocked;
    let rule = REPATH_WAITS.iter().sum::<Tick>()..=GIVE_UP_TICKS + STUCK_GRACE_TICKS;
    assert!(rule.contains(&after), "it gave up {after} ticks after it was blocked");
    assert_eq!(attack_ticks(&t, 2), [] as [Tick; 0], "it never got into range");
    t.step();
    assert!(
        is_idle(&t, 1) && is_idle(&t, 2),
        "the target is destroyed: both orders are complete"
    );
}
