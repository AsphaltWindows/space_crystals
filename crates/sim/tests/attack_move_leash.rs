//! AttackMove's leash and its PathReference (`systems/controls/behaviors.md > AttackMovingToLocation`, design change
//! 0180), with today's units: the PathReference is the route planned at the start and stays that after fights, a
//! detour around a blocked route takes its place, and a target the unit let go of isn't taken again and again.
//! The unit tests in `orders::tests::leash` give units other Ranges for the rest of 0180's Verification.

use sc_sim::orders::{ATTACK_MOVE_LEASH_GU, Behaviour};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, Shape, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2, gu};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick, Range 4, Sight 5
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5
const DRONE: ObjectKind = ObjectKind::Drone; // 36 × 36 su, unarmed

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn pos(t: &TestSim, n: u32) -> Vec2 {
    t.sim().object(id(n)).expect("exists").pos
}

fn gap(t: &TestSim, a: u32, b: u32) -> f32 {
    let shape = |n| Shape::of(t.sim().object(id(n)).expect("exists"));
    shape(a).gap(&shape(b))
}

fn is_idle(t: &TestSim, n: u32) -> bool {
    t.sim().object(id(n)).unwrap().orders.as_ref().unwrap().is_idle()
}

/// The target unit `n`'s AttackMove is engaged with, if any.
fn engaged(t: &TestSim, n: u32) -> Option<ObjectId> {
    let orders = t.sim().object(id(n)).unwrap().orders.as_ref().unwrap();
    match orders.current.as_ref().map(|active| &active.state) {
        Some(Behaviour::AttackMove(leg)) => leg.engaged.map(|pursuit| pursuit.target),
        _ => None,
    }
}

fn attack_ticks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == id(attacker)))
        .map(|&(tick, _)| tick)
        .collect()
}

/// A wall (row 3, from column 3 to the east edge) between a route along row 1 and row 5; the way round it is at its
/// west end.
const WALL: &str = "
    ..............................
    ..............................
    ..............................
    ...~~~~~~~~~~~~~~~~~~~~~~~~~~~
    ..............................
    ..............................
    ..............................
";

/// The take-and-drop loop (the soak's seed 1): a Guard attack-moves east along row 1 and sees, across the wall, an
/// unarmed enemy Drone 4 rows south, in its sight but out of its Range. The way to it leads back west round the
/// wall, out of sight at once: once its owner has lost sight of it, the fog delay (8 ticks) over, the Guard lets
/// it go and walks on east, where it sees the Drone again. Taken again as soon as it is seen, or after a fixed
/// while (it sees it for 384 su of its walk, 77 ticks), it would be let go of again and again, for good. Left alone
/// until the Guard has passed it, out of its sight (`orders::travel::march`), it walks on to its destination.
#[test]
fn a_target_whose_approach_leads_out_of_sight_is_let_go_until_the_unit_has_passed_it() {
    let mut t = ScenarioBuilder::new(WALL)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 1))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 5))) // 2
        .start();
    let to = centre(29, 1);
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .run_until(2);
    let mut taken = 0;
    let mut was_engaged = false;
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 1000, "it never got past the Drone\n{}", t.ascii(None));
        t.step();
        let now = engaged(&t, 1) == Some(id(2));
        taken += u32::from(now && !was_engaged);
        was_engaged = now;
    }
    assert_eq!(pos(&t, 1), to, "arrived");
    assert_eq!(taken, 1, "it went for the Drone once");
    assert!(pos(&t, 2) == centre(15, 5) && attack_ticks(&t, 1).is_empty());
}

/// Design change 0180, PathReference: "**not** replaced after fights (the walking path is re-planned, the leash line
/// isn't), so fights can't drag the unit progressively off course". A Guard attack-moves east along y = 200 and goes
/// after a Peacekeeper that runs south, until it would have to leave the leash, 6 grid units off the route, to
/// keep after it: it lets go there and walks on from where it stands, by a new path back towards the route. An
/// unarmed enemy Drone stands a little farther south, in the Guard's sight there but out of its Range, and it could
/// get into range of it within 6 grid units of that new path, though not of the route: it leaves the Drone alone.
#[test]
fn after_a_fight_the_leash_still_measures_from_the_route_planned_at_the_start() {
    let open = "................................\n".repeat(24);
    let mut t = ScenarioBuilder::new(&open).start();
    let (from, to) = (v(200.0, 200.0), v(1800.0, 200.0));
    t.spawn_unit_at(1, GUARD, P1, from)
        .spawn_unit_at(1, PK, P2, v(700.0, 480.0))
        .spawn_unit_at(1, DRONE, P2, v(800.0, 820.0))
        .order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .order_at(
            70,
            P2,
            &[id(2)],
            OrderKind::Move(Target::Point(v(700.0, 1500.0))),
            false,
        );
    let leash = gu(ATTACK_MOVE_LEASH_GU);
    let mut chased = false;
    let mut let_go = None;
    t.run_until(2);
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 900, "it never arrives\n{}", t.ascii(None));
        t.step();
        assert_ne!(
            engaged(&t, 1),
            Some(id(3)),
            "tick {}: it went for the Drone",
            t.sim().tick()
        );
        if engaged(&t, 1) == Some(id(2)) {
            chased = true;
        } else if chased && let_go.is_none() {
            let_go = Some((pos(&t, 1), gap(&t, 1, 3)));
        }
    }
    assert!(chased, "it went for the Peacekeeper");
    let (at, drone_gap) = let_go.expect("and let go of it");
    assert!(at.y - from.y > leash - 5.0, "at the leash: {at}");
    assert!(
        drone_gap > gu(3.0) && drone_gap <= gu(5.0),
        "the Drone in sight and out of range there: {drone_gap}"
    );
    assert_eq!(pos(&t, 1), to, "and went on to its destination");
    assert!(t.sim().object(id(2)).is_some(), "the Peacekeeper got away");
    assert_eq!(
        t.sim().object(id(3)).and_then(|o| o.hp),
        Some(20.0),
        "the Drone wasn't shot"
    );
}

/// Two ways east, joined at both ends: the direct one along row 0, one tile wide, and a detour along row 10, 10 grid
/// units south of it, with open ground south of it (rows 11 to 15).
const DETOUR: &str = "
    ..............................
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..~~~~~~~~~~~~~~~~~~~~~~~~~~..
    ..............................
    ..............................
    ..............................
    ..............................
    ..............................
    ..............................
";

/// Design change 0180's Verification: "Block the route so the unit detours 10 GU sideways: it can still chase
/// enemies near the detour." A Guard attack-moves east along row 0; another Guard of its owner holds its position in
/// that way, which a Guard fills. Stuck behind it, the Guard re-routes by the detour along row 10, 10 grid units
/// south (`docs/movement.md > 5.6`), and that is its PathReference from then on ("replaced on a blockage
/// re-route"). An unarmed enemy Drone stands south of the detour, 4 grid units off it: out of the Guard's Range, but
/// it can get into range of it within the leash of the detour, so it goes after the Drone and kills it, far beyond 6
/// grid units of the route it first planned.
#[test]
fn a_detour_round_a_blocked_route_is_the_new_path_reference() {
    let mut t = ScenarioBuilder::new(DETOUR)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 0))) // 1
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(10, 0))) // 2, the plug
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 14))) // 3
        .start();
    t.order_at(2, P1, &[id(2)], OrderKind::Hold, false)
        .order_at(2, P1, &[id(1)], OrderKind::AttackMove(centre(27, 0)), false)
        .run_until(2);
    let mut detoured_at = None;
    while t.sim().object(id(3)).is_some() {
        assert!(t.sim().tick() < 1500, "it never got the Drone\n{}", t.ascii(None));
        t.step();
        if detoured_at.is_none() && pos(&t, 1).y > centre(0, 8).y {
            detoured_at = Some(t.sim().tick());
        }
    }
    assert!(detoured_at.is_some(), "it took the detour");
    assert!(!attack_ticks(&t, 1).is_empty());
    assert!(
        pos(&t, 1).y > centre(0, 10).y,
        "it left the detour for the Drone: {}",
        pos(&t, 1)
    );
    // And on to its destination, by the detour.
    while !is_idle(&t, 1) {
        assert!(t.sim().tick() < 2500, "it never arrives\n{}", t.ascii(None));
        t.step();
    }
    assert_eq!(pos(&t, 1), centre(27, 0));
}
