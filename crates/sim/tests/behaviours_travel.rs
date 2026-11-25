//! Travelling behaviours (`systems/controls/behaviors.md`): `> MovingToObject` with Follow (design changes 0032 and
//! 0033) and `> AttackMovingToLocation`; and a fight under AttackMove replayed, and saved and loaded in its middle.
//! Patrols are in `behaviours_patrol.rs`.

use sc_sim::mover::{MAX_REPATHS, OCCUPIED_ARRIVAL_GU};
use sc_sim::orders::{ATTACK_MOVE_LEASH_GU, Behaviour, PROXIMITY_SU};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Shape, Sim, SimEvent, SpawnAt, Target, Tick,
    TickInputs, TilePos, Vec2, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick, Range 4, Sight 5
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5

/// 32 × 24 open tiles: 2048 × 1536 su.
fn open() -> String {
    "................................\n".repeat(24)
}

/// A corridor one tile (64 su) wide and 26 long, y from 64 to 128.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~~~~~~~~~~~
    ..........................
    ~~~~~~~~~~~~~~~~~~~~~~~~~~
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn ids(range: std::ops::RangeInclusive<u32>) -> Vec<ObjectId> {
    range.map(ObjectId).collect()
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

fn follow(n: u32) -> OrderKind {
    OrderKind::Move(Target::Object(id(n)))
}

fn exists(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).is_some()
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

fn behaviour(t: &TestSim, n: u32) -> Option<Behaviour> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    orders.current.as_ref().map(|active| active.state.clone())
}

fn gap(t: &TestSim, a: u32, b: u32) -> f32 {
    let shape = |n| Shape::of(t.sim().object(id(n)).expect("exists"));
    shape(a).gap(&shape(b))
}

fn attack_ticks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == id(attacker)))
        .map(|&(tick, _)| tick)
        .collect()
}

/// The ticks in which a move of `unit` failed.
fn failed_moves(t: &TestSim, unit: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::MoveFailed { unit: u } if *u == id(unit)))
        .map(|&(tick, _)| tick)
        .collect()
}

fn spawn_tiles(t: &mut TestSim, tick: Tick, kind: ObjectKind, x: i32, y: i32) {
    t.debug_at(
        tick,
        DebugCommand::Spawn {
            kind,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(x, y)),
            orientation: Default::default(),
        },
    );
}

/// Design change 0032's Verification: "Right-click own Peacekeeper B with A selected: A walks to B and waits within
/// ⅛ tile. Move B: A follows and keeps following until given another order." And the follow ends when B dies.
#[test]
fn a_move_to_a_unit_follows_it() {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0)) // A
        .spawn_unit_at(1, PK, P1, v(600.0, 300.0)) // B
        .order_at(2, P1, &[id(1)], follow(2), false);
    t.run_until(150);
    let waiting = gap(&t, 1, 2);
    assert!(waiting <= PROXIMITY_SU && waiting > PROXIMITY_SU - 4.0, "{waiting}");
    assert_eq!(t.status(1), MoveStatus::Idle, "it stopped and waits");
    assert_eq!(
        current(&t, 1),
        Some(follow(2)),
        "the order doesn't complete on reaching B"
    );
    assert_eq!(pos(&t, 2), v(600.0, 300.0), "B wasn't asked to make way");

    // B walks off, around a corner; A keeps up, a few steps behind, and closes up when B stops.
    t.order_at(151, P1, &[id(2)], move_to(v(1200.0, 300.0)), false)
        .order_at(151, P1, &[id(2)], move_to(v(1200.0, 900.0)), true);
    let mut farthest = 0.0_f32;
    t.run_until(151);
    while !is_idle(&t, 2) {
        assert!(t.sim().tick() < 800);
        t.step();
        farthest = farthest.max(gap(&t, 1, 2));
        assert_eq!(current(&t, 1), Some(follow(2)));
    }
    assert!(farthest <= PROXIMITY_SU + 8.0, "A fell {farthest} su behind");
    assert_eq!(pos(&t, 2), v(1200.0, 900.0));
    t.run(10);
    assert!(gap(&t, 1, 2) <= PROXIMITY_SU);
    assert!(pos(&t, 1).distance(v(1200.0, 900.0)) < 60.0, "{}", pos(&t, 1));
    t.run(200);
    assert_eq!(
        current(&t, 1),
        Some(follow(2)),
        "still following, for as long as B lives"
    );
    assert_eq!(failed_moves(&t, 1), [] as [Tick; 0]);

    // Orders queued behind a follow wait (like a Patrol) until the follow ends: B dies.
    let next = t.sim().tick() + 1;
    t.order_at(next, P1, &[id(1)], move_to(v(1000.0, 900.0)), true)
        .debug_at(next + 5, DebugCommand::Remove { id: id(2) });
    t.run_until(next + 4);
    assert_eq!(current(&t, 1), Some(follow(2)));
    t.run_until(next + 6);
    assert_eq!(
        current(&t, 1),
        Some(move_to(v(1000.0, 900.0))),
        "the follow ended, the next order runs"
    );
}

/// A new order ends a follow.
#[test]
fn a_new_order_ends_a_follow() {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, GUARD, P1, v(200.0, 300.0))
        .spawn_unit_at(1, PK, P1, v(260.0, 300.0))
        .order_at(2, P1, &[id(1)], follow(2), false)
        .order_at(2, P1, &[id(2)], move_to(v(1800.0, 300.0)), false)
        .order_at(100, P1, &[id(1)], OrderKind::Stop, false);
    t.run_until(99);
    // The faster Guard caught up and keeps within a step or two of proximity.
    assert!(gap(&t, 1, 2) <= PROXIMITY_SU + 10.0, "{}", gap(&t, 1, 2));
    t.run_until(100);
    let stopped = pos(&t, 1);
    assert!(is_idle(&t, 1));
    t.run(100);
    assert_eq!(pos(&t, 1), stopped);
}

/// Design change 0032's Verification: "Right-click a crystal patch: the unit walks to it and its order completes
/// there": within proximity, a gap of 8 su to the patch's tile. The same for a Supply Delivery Station, whose own
/// tiles block the way to its centre.
#[test]
fn a_move_to_a_static_object_completes_within_proximity() {
    let mut t = ScenarioBuilder::new(&open()).start();
    spawn_tiles(&mut t, 1, ObjectKind::CrystalPatch, 6, 4); // id 1: x 384..448, y 256..320
    spawn_tiles(&mut t, 1, ObjectKind::SupplyDeliveryStation, 6, 12); // id 2: x 384..512, y 768..896
    t.spawn_unit_at(1, PK, P1, v(200.0, 200.0))
        .spawn_unit_at(1, GUARD, P1, v(200.0, 700.0))
        .order_at(2, P1, &[id(3)], follow(1), false)
        .order_at(2, P1, &[id(4)], follow(2), false);
    t.run_until(2);
    assert_eq!((current(&t, 3), current(&t, 4)), (Some(follow(1)), Some(follow(2))));
    t.run_until(120);
    for (unit, object) in [(3, 1), (4, 2)] {
        assert!(is_idle(&t, unit), "unit {unit}'s order completed");
        let reached = gap(&t, unit, object);
        assert!(reached <= PROXIMITY_SU, "unit {unit} stopped {reached} su from it");
        assert_eq!(failed_moves(&t, unit), [] as [Tick; 0]);
    }
    let stands = (pos(&t, 3), pos(&t, 4));
    t.run(50);
    assert_eq!((pos(&t, 3), pos(&t, 4)), stands);
}

/// Design change 0033's Verification: "Right-click a crystal patch surrounded by idle enemy units: the unit stops
/// within 2 tiles and its order completes immediately", with no re-route. (Auto-targeting is off: this is about the
/// move.)
#[test]
fn a_static_object_behind_a_crowd_counts_as_reached_within_two_grid_units() {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting();
    spawn_tiles(&mut t, 1, ObjectKind::CrystalPatch, 10, 4); // id 1: x 640..704, y 256..320
    let centre = v(672.0, 288.0);
    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        t.spawn_unit_at(1, GUARD, P2, centre + v(50.0 * dx as f32, 50.0 * dy as f32)); // ids 2..=9
    }
    t.spawn_unit_at(1, PK, P1, v(400.0, 288.0)) // id 10
        .order_at(2, P1, &[id(10)], follow(1), false);
    t.run_until(2);
    assert_eq!(current(&t, 10), Some(follow(1)));
    let mut completed = None;
    while completed.is_none() {
        assert!(t.sim().tick() < 300, "the order never completes\n{}", t.ascii(None));
        t.step();
        if is_idle(&t, 10) {
            completed = Some(t.sim().tick());
        }
    }
    // 47 steps to the ring, then stuck there for 24 ticks at most.
    let completed = completed.unwrap();
    assert!(completed <= 2 + 47 + 24 + 2, "completed in tick {completed}");
    let reached = gap(&t, 10, 1);
    assert!(
        reached > PROXIMITY_SU && reached <= gu(2.0),
        "{reached} su from the patch"
    );
    assert_eq!(t.status(10), MoveStatus::Arrived);
    assert_eq!(t.mover(10).repaths_left, MAX_REPATHS, "no re-route attempt");
    assert_eq!(failed_moves(&t, 10), [] as [Tick; 0]);
    for guard in 2..=9 {
        assert_eq!(t.status(guard), MoveStatus::Idle, "nobody moved for it");
    }
}

/// Design change 0032, "Blocked near a followed unit": stuck within `OCCUPIED_ARRIVAL_GU` (3 grid units, design
/// change 0169) behind units that aren't moving counts as having reached it: the follower waits there, with no
/// re-route, and follows again when the target moves on.
#[test]
fn a_follower_blocked_near_its_target_waits_without_re_routing() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    // In the corridor: the follower, a Guard on Hold that plugs it, and the target beyond.
    t.spawn_unit_at(1, PK, P1, v(200.0, 96.0))
        .spawn_unit_at(1, GUARD, P1, v(700.0, 96.0))
        .spawn_unit_at(1, PK, P1, v(760.0, 96.0))
        .order_at(2, P1, &[id(2)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(1)], follow(3), false);
    t.run_until(400);
    let stopped = pos(&t, 1);
    assert!(
        stopped.x > 640.0 && stopped.x <= 700.0 - 18.0 - 12.0,
        "against the Guard: {stopped}"
    );
    assert_eq!(current(&t, 1), Some(follow(3)), "the follow goes on");
    assert_eq!(t.status(1), MoveStatus::Arrived, "it counts as reached");
    assert_eq!(t.mover(1).repaths_left, MAX_REPATHS);
    assert_eq!(failed_moves(&t, 1), [] as [Tick; 0]);
    assert!(gap(&t, 1, 3) > PROXIMITY_SU && gap(&t, 1, 3) <= gu(OCCUPIED_ARRIVAL_GU));
    // The target walks on down the corridor: the follower sets off again (and is blocked again).
    t.order_at(401, P1, &[id(3)], move_to(v(1500.0, 96.0)), false)
        .run_until(402);
    assert_eq!(t.status(1), MoveStatus::Moving);
}

/// Design change 0200 (it replaces 0034's walk to the last seen position): following an enemy (a Move on it) that
/// "goes out of the owner's vision", "the behavior completes, and the unit stops where it is", in the next tick
/// (behaviours read the vision of the tick before). A Move never engages.
#[test]
fn a_followed_enemy_lost_in_the_fog_ends_the_follow_where_the_unit_stands() {
    let mut t = ScenarioBuilder::new(&open()).start();
    t.spawn_unit_at(1, PK, P1, v(200.0, 300.0))
        .spawn_unit_at(1, GUARD, P2, v(450.0, 300.0))
        .order_at(2, P1, &[id(1)], follow(2), false)
        .order_at(2, P2, &[id(2)], move_to(v(1900.0, 300.0)), false);
    let mut lost = None;
    t.run_until(2);
    assert_eq!(current(&t, 1), Some(follow(2)));
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 600, "the follow never ends\n{}", t.ascii(None));
        t.step();
        if lost.is_none() && !t.sim().is_visible(P1, t.sim().object(id(2)).unwrap()) {
            lost = Some((t.sim().tick() + 1, pos(&t, 1)));
        }
    }
    let (now, here) = (t.sim().tick(), pos(&t, 1));
    assert_eq!(lost, Some((now, here)), "it stops at once, right there");
    assert_eq!(t.status(1), MoveStatus::Idle);
    t.run(100);
    assert_eq!(pos(&t, 1), here, "no walk into the fog");
    assert_eq!(attack_ticks(&t, 1), [] as [Tick; 0]);
    assert!(pos(&t, 2).x > pos(&t, 1).x + gu(5.0));
}

/// The brief's "an AttackMove group engaging and resuming" (`> AttackMovingToLocation`): six Peacekeepers
/// attack-move east. Three Guards hold their position on the way. The group stops for them as they come into
/// sight, closes in to its own Range (beyond the Guards'), destroys them, and then goes on to its destinations:
/// each unit's own, by the magic box, so the group arrives in the shape it had.
#[test]
fn an_attack_move_group_engages_on_the_way_and_resumes() {
    let mut t = ScenarioBuilder::new(&open()).start();
    let starts: Vec<Vec2> = (0..6)
        .map(|i| v(200.0 + 40.0 * (i % 3) as f32, 280.0 + 40.0 * (i / 3) as f32))
        .collect();
    for &at in &starts {
        t.spawn_unit_at(1, PK, P1, at); // ids 1..=6
    }
    for i in 0..3 {
        t.spawn_unit_at(1, GUARD, P2, v(1000.0, 250.0 + 50.0 * i as f32)); // ids 7..=9
    }
    let click = v(1700.0, 300.0);
    t.order_at(2, P2, &ids(7..=9), OrderKind::Hold, false).order_at(
        2,
        P1,
        &ids(1..=6),
        OrderKind::AttackMove(click),
        false,
    );
    t.run_until(2);
    assert!(matches!(behaviour(&t, 1), Some(Behaviour::AttackMove(leg)) if leg.engaged.is_none() && leg.marching));

    // They meet: everyone has a target, and nobody walks on past the Guards.
    let mut engaged_at = None;
    while (7..=9).any(|guard| exists(&t, guard)) {
        assert!(t.sim().tick() < 900, "the Guards outlast them\n{}", t.ascii(None));
        t.step();
        let engaged = |n| matches!(behaviour(&t, n), Some(Behaviour::AttackMove(leg)) if leg.engaged.is_some());
        if engaged_at.is_none() && (1..=6).all(engaged) {
            engaged_at = Some(t.sim().tick());
        }
        for unit in 1..=6 {
            assert!(
                pos(&t, unit).x < 1000.0 - 18.0 - 12.0 - gu(3.0),
                "unit {unit} walked into the Guards' range"
            );
        }
    }
    let fight_over = t.sim().tick();
    assert!(
        engaged_at.is_some_and(|at| at < fight_over),
        "the whole group engaged: {engaged_at:?}"
    );
    let fired = (1..=6).filter(|&unit| !attack_ticks(&t, unit).is_empty()).count();
    assert!(fired >= 4, "{fired} of them got a shot in");
    assert_eq!(
        (1..=6).filter(|&unit| exists(&t, unit)).count(),
        6,
        "out of the Guards' range, nobody was lost"
    );

    // They resume, and arrive as the group they were: every unit on the click plus its offset from the group's
    // centre, (240, 300).
    t.run_until(fight_over + 1);
    assert!(matches!(behaviour(&t, 1), Some(Behaviour::AttackMove(leg)) if leg.engaged.is_none() && leg.marching));
    while (1..=6).any(|unit| !is_idle(&t, unit)) {
        assert!(
            t.sim().tick() < fight_over + 400,
            "they never arrive\n{}",
            t.ascii(None)
        );
        t.step();
    }
    // The fight shuffled them, so a unit may find its place walled in by those that arrived before it: it has then
    // arrived behind them, within `OCCUPIED_ARRIVAL_GU` (`> MovingToLocation > Occupied target location`).
    let mut exact = 0;
    for (unit, &start) in (1..=6).zip(&starts) {
        let destination = start + (click - v(240.0, 300.0));
        let off = pos(&t, unit).distance(destination);
        assert!(
            off <= gu(OCCUPIED_ARRIVAL_GU),
            "unit {unit} stopped {off} su from its place"
        );
        assert_eq!(t.status(unit), MoveStatus::Arrived);
        exact += u32::from(off == 0.0);
    }
    assert!(exact >= 4, "{exact} of 6 stand on their own place");
}

/// `> AttackMovingToLocation` 3 (design change 0180): "The unit does not move to chase a target if that would take it
/// beyond AttackMoveLeashDistance (perpendicular distance from PathReference). If the target is out of weapon range
/// and can only be reached by going beyond the leash: disengage … resume MovingToLocation". A Guard attack-moves
/// east; a Peacekeeper it acquires on the way runs south, a little slower than the Guard follows. The Guard goes
/// after it up to its leash, 6 grid units off its route (a step past it at most: it closes in step by step), lets
/// go there and walks on to its destination. (`tests/attack_move_leash.rs` has the rest of 0180.)
#[test]
fn an_attack_move_breaks_off_at_its_leash_and_goes_on() {
    let mut t = ScenarioBuilder::new(&open()).start();
    let (from, to) = (v(200.0, 200.0), v(1800.0, 200.0));
    t.spawn_unit_at(1, GUARD, P1, from)
        .spawn_unit_at(1, PK, P2, v(700.0, 480.0))
        .order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .order_at(70, P2, &[id(2)], move_to(v(700.0, 1500.0)), false);
    let mut farthest = 0.0_f32;
    let mut chased = false;
    t.run_until(2);
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 900, "it never arrives\n{}", t.ascii(None));
        t.step();
        farthest = farthest.max(pos(&t, 1).y - from.y);
        chased |= matches!(behaviour(&t, 1), Some(Behaviour::AttackMove(leg)) if leg.engaged.is_some());
    }
    assert!(chased, "it went for the Peacekeeper");
    let leash = gu(ATTACK_MOVE_LEASH_GU);
    assert!(
        farthest > leash - 5.0 && farthest <= leash + 5.0,
        "it left its route by {farthest} su, within a step of the leash of {leash}"
    );
    assert_eq!(pos(&t, 1), to, "and went on to its destination");
    assert!(exists(&t, 2), "the Peacekeeper got away");
}

/// Two groups of six Peacekeepers attack-move into each other.
fn skirmish() -> TestSim {
    let mut t = ScenarioBuilder::new(&open()).seed(11).start();
    for i in 0..6 {
        let (col, row) = ((i % 2) as f32, (i / 2) as f32);
        t.spawn_unit_at(1, PK, P1, v(300.0 + 40.0 * col, 400.0 + 44.0 * row)); // ids 1, 3, .. 11
        t.spawn_unit_at(1, PK, P2, v(1300.0 - 40.0 * col, 410.0 + 44.0 * row)); // ids 2, 4, .. 12
    }
    let (west, east): (Vec<ObjectId>, Vec<ObjectId>) = (
        (1..=12).step_by(2).map(id).collect(),
        (2..=12).step_by(2).map(id).collect(),
    );
    t.order_at(2, P1, &west, OrderKind::AttackMove(v(1300.0, 444.0)), false)
        .order_at(2, P2, &east, OrderKind::AttackMove(v(300.0, 444.0)), false);
    t
}

/// The brief's "replay hash plus save/load in the middle of an AttackMove fight": the fight is the same every
/// time, a replay of it reproduces it, and a save taken while shots are in the air (attacks aiming, firing and
/// cooling down, units chasing) continues exactly as the game it was taken from. The invariants hold throughout.
#[test]
fn an_attack_move_fight_replays_and_survives_a_save_in_its_middle() {
    let mut t = skirmish();
    t.run_until(140);
    // In the middle of it: both sides have taken losses or damage, and units are engaged.
    let sim = t.sim();
    let hurt = sim.objects().filter(|o| o.hp.is_some_and(|hp| hp < 50.0)).count();
    let engaged = sim
        .objects()
        .filter(|o| {
            o.attack
                .as_ref()
                .is_some_and(|a| a.target.is_some() && a.phase.is_some())
        })
        .count();
    assert!(
        hurt >= 2 && engaged >= 4,
        "{hurt} hurt, {engaged} engaged\n{}",
        t.ascii(None)
    );
    assert!(sim.objects().count() >= 6, "the fight isn't over");

    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(&loaded, t.sim());
    let mut hashes = Vec::new();
    for _ in 0..400 {
        t.step();
        loaded.step(&TickInputs::default());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
        hashes.push(t.sim().state_hash());
    }
    assert_eq!(&loaded, t.sim());
    // One side won, and its survivors went on to their destination.
    let owners: Vec<Option<PlayerId>> = t.sim().objects().map(|o| o.owner).collect();
    assert!(
        !owners.is_empty() && owners.iter().all(|&owner| owner == owners[0]),
        "{owners:?}"
    );
    assert!(t.sim().objects().all(|o| o.orders.as_ref().unwrap().is_idle()));

    // The same again from the start, and from the recorded replay.
    let mut again = skirmish();
    again.run_until(140 + 400);
    assert_eq!(again.sim().state_hash(), t.sim().state_hash());
    let replay = t.replay().clone();
    let json = serde_json::to_string(&replay).unwrap();
    let replayed: sc_sim::Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(replayed.run().state_hash(), t.sim().state_hash());
    assert_eq!(hashes.len(), 400);
}
