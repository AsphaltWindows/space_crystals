//! A target lost from vision (design change 0200, answering question Q0051; it replaces design change 0034's walk to
//! the last seen position): `systems/controls/behaviors.md > AttackingObject > Algorithm` 6, "if TargetObject is no
//! longer visible to the unit's owner (it went into the fog of war), the behavior completes at once, exactly as when
//! the target is destroyed. The unit stops where it is and is idle, or goes on to its next queued order if it has
//! one"; and `> MovingToObject`, a Follow "same as AttackingObject > Target lost from vision".
//!
//! Here: the queued order after an Attack and after a Follow; a structure the fog covers while an Attack on it runs,
//! which the player may order again; AttackMove, which resumes its leg; an idle unit whose own target leaves its
//! sight; and a unit on its way round its owner's attackers (slice M3-G2), which stops there. Elsewhere: the stop
//! itself (`behaviours_attack.rs`, `behaviours_travel.rs`), the target kept through the fog delay and lost after it
//! (`fog_delay.rs`), an Attack on a structure remembered when it is given, which walks to the record (design change
//! 0139, `remembered_targets.rs`), and rally points on enemies, unchanged (`production/rally.rs`,
//! `production/drone_rally.rs`).
//!
//! Every `TestSim` checks the invariants I1–I7 and V1 after each tick.

use sc_sim::orders::{Auto, Behaviour};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tuning::orders::IDLE_LEASH_GU;
use sc_sim::{
    Command, DebugCommand, MoveGoal, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Shape,
    SimEvent, SpawnAt, Target, TargetView, Tick, TilePos, Vec2, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick, Range 4, Sight 5
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5
const DRONE: ObjectKind = ObjectKind::Drone; // 36 × 36 su, unarmed

/// 48 × 24 open tiles: 3072 × 1536 su.
fn open() -> String {
    format!("{}\n", ".".repeat(48)).repeat(24)
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

fn pos(t: &TestSim, n: u32) -> Vec2 {
    t.sim().object(id(n)).expect("exists").pos
}

fn is_idle(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).unwrap().orders.as_ref().unwrap().is_idle()
}

/// Unit `n`'s current order, and the tick it started.
fn current(t: &TestSim, n: u32) -> Option<(OrderKind, Tick)> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| (active.order.kind, active.since))
}

/// What unit `n`'s attack engages.
fn target(t: &TestSim, n: u32) -> Option<ObjectId> {
    t.sim().object(id(n)).unwrap().attack.as_ref().unwrap().target
}

fn auto(t: &TestSim, n: u32) -> Auto {
    t.sim().object(id(n)).unwrap().orders.as_ref().unwrap().auto
}

/// The target unit `n`'s AttackMove is engaged with, if any.
fn engaged(t: &TestSim, n: u32) -> Option<ObjectId> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    match orders.current.as_ref().map(|active| &active.state) {
        Some(Behaviour::AttackMove(leg)) => leg.engaged.map(|pursuit| pursuit.target),
        _ => None,
    }
}

fn gap(t: &TestSim, a: u32, b: u32) -> f32 {
    let shape = |n| Shape::of(t.sim().object(id(n)).expect("exists"));
    shape(a).gap(&shape(b))
}

fn seen(t: &TestSim, n: u32) -> bool {
    t.sim().is_visible(P1, t.sim().object(id(n)).unwrap())
}

fn attack_ticks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == id(attacker)))
        .map(|&(tick, _)| tick)
        .collect()
}

fn teleport(t: &mut TestSim, tick: Tick, n: u32, to: Vec2) {
    t.debug_at(tick, DebugCommand::SetPosition { id: id(n), pos: to });
}

/// Steps until player 1 no longer sees object `n`: the tick it left sight, and where unit 1 stood then.
fn until_lost(t: &mut TestSim, n: u32) -> (Tick, Vec2) {
    while seen(t, n) {
        assert!(t.sim().tick() < 600, "#{n} never left sight\n{}", t.ascii(None));
        t.step();
    }
    (t.sim().tick(), pos(t, 1))
}

/// Player 1's Peacekeeper (1) at (200, 300) is ordered `order` on player 2's Guard (2), in its sight and out of its
/// Range, with a Move to `to` Shift-queued after it, in tick 2, as the Guard sets off east, faster than it. Run to
/// the tick after the Guard left player 1's sight; returns where the Peacekeeper stood as it did.
fn chase_then(order: OrderKind, to: Vec2) -> (TestSim, Vec2) {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(500.0, 300.0))
        .order_at(2, P1, &[id(1)], order, false)
        .order_at(2, P1, &[id(1)], move_to(to), true)
        .order_at(2, P2, &[id(2)], move_to(v(2900.0, 300.0)), false)
        .run_until(2);
    assert_eq!(current(&t, 1), Some((order, 2)));
    let (lost, there) = until_lost(&mut t, 2);
    assert_eq!(
        current(&t, 1),
        Some((order, 2)),
        "not before the behaviour sees it gone"
    );
    t.step();
    // The next order starts in the tick the first ends, from where the unit stands.
    assert_eq!(current(&t, 1), Some((move_to(to), lost + 1)));
    assert_eq!(t.mover(1).goal, MoveGoal::Point(to));
    assert_eq!(target(&t, 1), None);
    (t, there)
}

/// Design change 0200's Verification: "With a Shift-queued Move after the Attack, it goes on to that Move." The
/// Guard outruns the Peacekeeper's chase into the fog; in the next tick the Attack completes and the queued Move
/// starts, and the Peacekeeper walks from there to its destination, south-west, away from the fog the Guard went
/// into.
#[test]
fn after_an_attack_lost_in_the_fog_the_unit_goes_on_to_its_queued_move() {
    let to = v(300.0, 900.0);
    let (mut t, there) = chase_then(OrderKind::Attack(id(2)), to);
    assert!(there.x > 200.0, "it went after the Guard: {there}");
    t.run(300);
    assert_eq!((pos(&t, 1), t.status(1)), (to, MoveStatus::Arrived));
    assert!(is_idle(&t, 1));
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0], "the Guard was never in range");
}

/// `> MovingToObject` (Follow): "Following continues until ... the target ... goes out of the owner's vision ...;
/// then the behavior completes, and the unit stops where it is (or goes on to its next queued order)." The Follow
/// never ends by itself, so its queued Move waits for it: the followed Guard runs out of sight, the Follow ends, and
/// the Move starts in the same tick.
#[test]
fn after_a_follow_lost_in_the_fog_the_unit_goes_on_to_its_queued_order() {
    let to = v(300.0, 900.0);
    let (mut t, _) = chase_then(OrderKind::Move(Target::Object(id(2))), to);
    t.run(300);
    assert_eq!((pos(&t, 1), t.status(1)), (to, MoveStatus::Arrived));
    assert!(is_idle(&t, 1));
}

/// The brief's reading of 0200 for a structure: "'No longer visible' means a change during the behaviour. An order
/// given on a remembered object (0139) still walks to its record, and attacks once the target is seen. A target that
/// leaves vision after that completes the behaviour." Player 1's Peacekeeper (1) attacks player 2's Generator (3),
/// far east, which only its scout (2) sees; the scout is removed on the way. Once the fog delay is over the owner
/// remembers the Generator, out of its sight, and the Attack ends at once: the Peacekeeper stops where it is. It
/// doesn't walk on to the record. Ordered again, on the remembered Generator, it does (0139), and attacks it once
/// it sees it standing.
#[test]
fn an_attack_on_a_structure_the_fog_covers_ends_and_may_be_ordered_again() {
    let mut t = ScenarioBuilder::new(&open())
        .spawn(PK, Some(P1), SpawnAt::Point(v(200.0, 320.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(v(1850.0, 320.0)))
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(30, 4)))
        .start();
    let generator = id(3);
    t.without_auto_targeting()
        .order_at(2, P1, &[id(1)], OrderKind::Attack(generator), false)
        .debug_at(10, DebugCommand::Remove { id: id(2) })
        .run_until(10);
    assert_eq!(t.status(1), MoveStatus::Moving, "on its way");
    let (lost, there) = until_lost(&mut t, 3);
    assert!(matches!(
        t.sim().target_view(P1, generator),
        TargetView::Remembered { .. }
    ));
    assert_eq!(current(&t, 1).map(|(kind, _)| kind), Some(OrderKind::Attack(generator)));
    t.step();
    assert!(is_idle(&t, 1), "the Attack ended in tick {}", lost + 1);
    assert_eq!(pos(&t, 1), there, "where it stood");
    t.run(50);
    assert_eq!(
        (pos(&t, 1), t.status(1)),
        (there, MoveStatus::Idle),
        "no walk on to the record"
    );
    // Ordered again: the gate takes the remembered Generator, and the unit walks to it and attacks it.
    let again = PlayerCommand {
        player: P1,
        command: Command::Order {
            units: vec![id(1)],
            kind: OrderKind::Attack(generator),
            queue: false,
        },
    };
    assert_eq!(t.sim().validate(&again), Ok(()));
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, again.command).run(600);
    assert!(
        attack_ticks(&t, 1).first().is_some_and(|&first| first > tick),
        "it attacked the Generator: {:?}",
        attack_ticks(&t, 1)
    );
}

/// The brief's reading of 0200 for AttackMove: "When the attack completes this way, the unit resumes its leg" (`>
/// AttackMovingToLocation` 3: the unit "resume[s] MovingToLocation from current position toward TargetLocation").
/// A Peacekeeper attack-moves east and goes after an unarmed Drone ahead of it, in its sight but out of its Range;
/// the Drone vanishes into the fog (set far off), and the Peacekeeper lets go of it at once and walks on along its
/// leg. Set back on its way, within its Range, the Drone is taken again and fought; the Peacekeeper then goes on to
/// its destination. (Out of its Range, it is judged like any other enemy by the reach rule once it has moved, design
/// change 0230: `orders::reach`, `attack_move_reach.rs`.)
#[test]
fn an_attack_move_resumes_its_leg_and_takes_the_target_again_in_range() {
    let to = v(2400.0, 300.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, DRONE, P2, v(700.0, 300.0))
        .order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .run_until(2);
    while engaged(&t, 1).is_none() {
        assert!(t.sim().tick() < 200, "it never went for the Drone");
        t.step();
    }
    assert_eq!(engaged(&t, 1), Some(id(2)));
    assert!(gap(&t, 1, 2) > gu(4.0), "out of Range: it goes after it");
    // Set off where the owner's last sight didn't reach, it is out of sight for the behaviour in the same tick.
    let tick = t.sim().tick();
    teleport(&mut t, tick + 1, 2, v(2900.0, 1400.0));
    t.run(1);
    assert!(!seen(&t, 2));
    assert_eq!(engaged(&t, 1), None, "let go of");
    assert_eq!(target(&t, 1), None);
    assert!(
        matches!(&current(&t, 1), Some((OrderKind::AttackMove(at), 2)) if *at == to),
        "the same leg"
    );
    assert_eq!(t.mover(1).goal, MoveGoal::Point(to), "it resumes its leg");
    let from = pos(&t, 1);
    t.run(20);
    assert!(pos(&t, 1).x > from.x + 60.0, "and walks on: {}", pos(&t, 1));
    // Back on its way, within its Range: taken again, and fought.
    let (tick, ahead) = (t.sim().tick(), pos(&t, 1) + v(12.0 + 200.0 + 18.0, 0.0));
    teleport(&mut t, tick + 1, 2, ahead);
    t.run(2);
    assert_eq!(engaged(&t, 1), Some(id(2)), "taken again");
    while t.sim().object(id(2)).is_some() {
        assert!(t.sim().tick() < tick + 300, "the Drone should be destroyed by now");
        t.step();
    }
    assert!(attack_ticks(&t, 1).first().is_some_and(|&first| first > tick));
    t.run(600);
    assert_eq!((pos(&t, 1), t.status(1)), (to, MoveStatus::Arrived));
    assert!(is_idle(&t, 1));
}

/// The brief's reading of 0200 for idle units: "If the auto-target leaves vision, the attack completes, and the Idle
/// behaviour's own rules (IdleLeashDistance) take over" (`> Active during Idle`: "If the target ... leaves
/// SightRange, the unit returns to idle at its current position (new IdleOrigin)"). An idle Peacekeeper goes after
/// an enemy Peacekeeper that walks away as fast as it follows, in its sight and out of its Range; the enemy vanishes
/// into the fog (set far off), and the unit stops where it is: it doesn't walk back to where it stood. Set back into
/// its sight, walking on, the enemy is engaged again, from there: the unit chases it up to IdleLeashDistance from
/// where it stopped, its new IdleOrigin, and walks back there.
#[test]
fn an_idle_unit_stops_where_its_target_left_sight_and_takes_it_again_within_its_leash() {
    let origin = v(300.0, 500.0);
    // A gap of 270 su: beyond Range (256), within SightRange (320).
    let ahead = |at: Vec2| at + v(12.0 + 270.0 + 12.0, 0.0);
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, PK, P1, origin)
        .spawn_unit_at(1, PK, P2, ahead(origin))
        .order_at(2, P2, &[id(2)], move_to(v(2900.0, 500.0)), false)
        .run_until(2);
    assert!(matches!(auto(&t, 1), Auto::Engaging { pursuit, origin: at } if pursuit.target == id(2) && at == origin));
    teleport(&mut t, 20, 2, v(2900.0, 1400.0));
    t.run_until(21);
    let stopped = pos(&t, 1);
    assert_eq!((target(&t, 1), auto(&t, 1)), (None, Auto::default()), "back to idle");
    assert!(stopped.x > origin.x + 40.0, "it had gone after the enemy: {stopped}");
    t.run(30);
    assert_eq!(
        (pos(&t, 1), t.status(1)),
        (stopped, MoveStatus::Idle),
        "it stays where it stopped"
    );
    // Back in its sight, walking on east.
    let tick = t.sim().tick();
    teleport(&mut t, tick + 1, 2, ahead(stopped));
    t.run(2);
    assert!(
        matches!(auto(&t, 1), Auto::Engaging { pursuit, origin: at } if pursuit.target == id(2) && at == stopped),
        "{:?}",
        auto(&t, 1)
    );
    let mut farthest = 0.0_f32;
    let mut returned = false;
    while !(returned && t.status(1) == MoveStatus::Arrived) {
        assert!(t.sim().tick() < tick + 400, "it never came back\n{}", t.ascii(None));
        t.step();
        farthest = farthest.max(pos(&t, 1).distance(stopped));
        returned |= auto(&t, 1) == Auto::Returning;
    }
    assert!(
        farthest > gu(IDLE_LEASH_GU) && farthest <= gu(IDLE_LEASH_GU) + 4.0,
        "up to the leash from where it stopped: {farthest}"
    );
    assert_eq!(pos(&t, 1), stopped, "back on its new IdleOrigin");
    assert!(is_idle(&t, 1), "it had no order all the while");
}

/// Slice M3-G2's detour (`movement::round`, `Mover::go_round`): a unit chasing its target, queueing behind a unit
/// of its owner that stands attacking the same target, goes round the row that unit stands in. Its target lost from
/// sight in the middle of the detour ends the Attack there too: the unit stops where it is, and the rest of the
/// detour goes with its path (`Mover::detour_end` back to 0). Player 1's Peacekeeper 2 stands in range of player 2's
/// Guard (1) and attacks it, in the middle of a row of six more on Hold (4 to 9, auto-targeting off: they don't fire,
/// and nor does the Guard); Peacekeeper 3, behind the row, is ordered to attack the Guard too. Once it is on its way
/// round the row, the Guard vanishes into the fog (set far off).
#[test]
fn a_target_lost_in_the_fog_ends_a_detour_round_friends_where_the_unit_is() {
    // The Guard's edge is at x 1138; Peacekeeper 2's at 1388, 250 su off, within its Range (256). Peacekeeper 3
    // can't get within it straight on: right behind Peacekeeper 2, its edge would be at 1412.
    let mut builder = ScenarioBuilder::new(&open())
        .spawn(GUARD, Some(P2), SpawnAt::Point(v(1120.0, 288.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(v(1400.0, 288.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(v(1480.0, 288.0)));
    // 2 su apart: a Peacekeeper doesn't fit between them.
    for k in [1.0, 2.0, 3.0, -1.0, -2.0, -3.0] {
        builder = builder.spawn(PK, Some(P1), SpawnAt::Point(v(1400.0, 288.0 + 26.0 * k)));
    }
    let mut t = builder.start();
    let row: Vec<ObjectId> = (4..10).map(id).collect();
    t.without_auto_targeting()
        .order_at(2, P1, &row, OrderKind::Hold, false)
        .order_at(2, P1, &[id(2), id(3)], OrderKind::Attack(id(1)), false)
        .run_until(2);
    while t.mover(3).detour_end == 0 {
        assert!(t.sim().tick() < 200, "it never went round\n{}", t.ascii(None));
        assert!(t.sim().object(id(1)).is_some(), "the Guard fell first");
        t.step();
    }
    assert_eq!(t.status(3), MoveStatus::Moving);
    assert!(!attack_ticks(&t, 2).is_empty(), "Peacekeeper 2 fires");
    let there = pos(&t, 3);
    let tick = t.sim().tick() + 1;
    teleport(&mut t, tick, 1, v(200.0, 1300.0));
    t.step();
    assert!(!seen(&t, 1), "out of sight at once");
    assert!(is_idle(&t, 2) && is_idle(&t, 3), "both Attacks ended");
    let mover = t.mover(3);
    assert_eq!(
        (mover.goal, mover.status, mover.detour_end),
        (MoveGoal::None, MoveStatus::Idle, 0),
        "it stopped, its detour gone"
    );
    assert_eq!(pos(&t, 3), there, "where it was");
    t.run(60);
    assert_eq!(pos(&t, 3), there, "and it stays");
    assert_eq!(attack_ticks(&t, 3), [] as [Tick; 0], "it never got into range");
}
