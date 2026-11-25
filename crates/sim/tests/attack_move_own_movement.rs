//! AttackMove's reach rule, re-checked by the unit's own movement (`systems/controls/behaviors.md >
//! AttackMovingToLocation > Algorithm` 2, "Re-checking a rejected enemy"; design change 0243, answering question
//! Q0074): a rejection by the reach rule is judged again once "the unit itself has moved a meaningful distance since
//! it judged that enemy (placeholder 1 GU; Q0074, so the same situation always gets the same response)"
//! (`tuning::orders::REJUDGE_SELF_SU`). "Failure-rule and fog rejections keep 0230's triggers": the corridor tests of
//! `attack_move_reach.rs` and `attack_behind_friends/no_room.rs` hold as they were. The rest of the reach rule is
//! `attack_move_reach.rs`'s.

use sc_sim::orders::state::{Judged, Leg};
use sc_sim::orders::{Behaviour, JUDGE_PATHS_PER_TICK};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tuning::orders::REJUDGE_SELF_SU;
use sc_sim::{DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerId, Shape, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick, Range 3, Sight 5
const DRONE: ObjectKind = ObjectKind::Drone; // 36 × 36 su, unarmed
/// The Guard's SightRange and Range, as its scan measures them on flat ground: edge to edge.
const SIGHT_SU: f32 = 5.0 * 64.0;
const RANGE_SU: f32 = 3.0 * 64.0;

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

/// The leg of unit `n`'s AttackMove, while it has one.
fn leg(t: &TestSim, n: u32) -> Option<&Leg> {
    let orders = t.sim().object(id(n))?.orders.as_ref()?;
    match orders.current.as_ref().map(|active| &active.state) {
        Some(Behaviour::AttackMove(leg)) => Some(leg),
        _ => None,
    }
}

/// The target unit `n`'s AttackMove is engaged with, if any.
fn engaged(t: &TestSim, n: u32) -> Option<ObjectId> {
    leg(t, n)?.engaged.map(|pursuit| pursuit.target)
}

/// The rejection unit `n`'s leg keeps of enemy `enemy`, if any.
fn rejection(t: &TestSim, n: u32, enemy: u32) -> Option<Judged> {
    leg(t, n)?
        .rejected
        .iter()
        .copied()
        .find(|judged| judged.target == id(enemy))
}

fn attack_ticks(t: &TestSim, attacker: u32, target: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| {
            matches!(e, SimEvent::Attacked { attacker: a, target: b, .. } if *a == id(attacker) && *b == id(target))
        })
        .map(|&(tick, _)| tick)
        .collect()
}

/// Whether unit 1's scan finds enemy `enemy` this tick: its owner sees it, within its SightRange (`World::pick`, by
/// the snapshot the last tick left: read it before the step).
fn in_its_sight(t: &TestSim, enemy: u32) -> bool {
    let object = t.sim().object(id(enemy)).expect("exists");
    t.sim().is_visible(P1, object) && gap(t, 1, enemy) <= SIGHT_SU
}

/// The rejections of unit 1's leg that its scan finds this tick and its own movement makes it judge again: it stands
/// [`REJUDGE_SELF_SU`] or more from where it judged them (read before the step, as [`in_its_sight`]).
fn due(t: &TestSim) -> Vec<ObjectId> {
    let at = pos(t, 1);
    let rejected = leg(t, 1).map_or(&[][..], |leg| &leg.rejected);
    rejected
        .iter()
        .filter(|judged| in_its_sight(t, judged.target.0))
        .filter(|judged| judged.from.is_some_and(|from| from.distance(at) >= REJUDGE_SELF_SU))
        .map(|judged| judged.target)
        .collect()
}

/// An L-shaped wall: along row 3 from the west edge to column 23, and down column 23 to row 10. From the north of
/// it, the way to its inside is round its end (tile (24, 11)), and back west along row 11.
const L_WALL: &str = "
    ..............................
    ..............................
    ..............................
    ~~~~~~~~~~~~~~~~~~~~~~~~......
    .......................~......
    .......................~......
    .......................~......
    .......................~......
    .......................~......
    .......................~......
    .......................~......
    ..............................
    ..............................
    ..............................
";

/// The building of [`L_WALL`]'s scene: a Generator of player 2 inside the wall's corner, on tiles (10, 6) to (11, 7),
/// object 2. Unit 1 is a Guard of player 1 at `at`, ordered in tick 2 to attack-move to [`INSIDE`].
fn l_wall(at: Vec2) -> TestSim {
    let mut t = ScenarioBuilder::new(L_WALL)
        .spawn(GUARD, Some(P1), SpawnAt::Point(at)) // 1
        .spawn(ObjectKind::Generator, Some(P2), SpawnAt::Tile(TilePos::new(10, 6))) // 2
        .start();
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(INSIDE), false)
        .run_until(2);
    t
}

/// The destination of [`l_wall`]'s order: inside the wall's corner, at its west end, so that a route from the north
/// of the wall goes round its end and back west along row 11, 3.4 grid units south of the Generator: in a Guard's
/// sight and out of its Range.
const INSIDE: Vec2 = Vec2::new(96.0, 760.0);

/// **Design change 0243's Verification**: "A Guard attack-moves along a wall past a building it rejected. After
/// rounding the wall into the open, with the building reachable within leash and sight, it attacks." And question
/// Q0074's comparison: "A second Guard that arrives at the same spot with no history attacks the building."
///
/// The Guard attack-moves from the north of [`L_WALL`] to its inside, round its end. Walking east along the wall, it
/// passes the Generator across it, in its sight and out of its Range: the way to the Generator leads round the
/// wall's end, out of its sight of it, so the Guard rejects it, and again for each grid unit it walks with the
/// Generator in its sight. Round the end and walking back west in the open, it has the Generator in its sight again,
/// out of its Range. It stands far from where it judged, so it judges it again, takes it and attacks it. Before 0243
/// it walked on to its destination: the building hadn't moved, its owner knew the same obstacles and its route was the
/// same, and the route never brings it within the Guard's Range (as measured, 218 su at the nearest).
///
/// The fresh Guard, placed where the first one took the Generator and given the same order, takes it in the tick of
/// its order, out of its Range, and attacks it: the same response.
#[test]
fn a_guard_that_rounds_the_wall_attacks_the_building_as_a_fresh_one_would() {
    let mut t = l_wall(centre(2, 1));
    let mut judged_north = Vec::new();
    while engaged(&t, 1).is_none() {
        assert!(t.sim().tick() < 1000, "never took it\n{}", t.ascii(None));
        t.step();
        if let Some(judged) = rejection(&t, 1, 2)
            && judged_north.last() != Some(&judged)
        {
            assert!(pos(&t, 1).y < 3.0 * 64.0, "rejected north of the wall: {judged:?}");
            judged_north.push(judged);
        }
    }
    let (took, at) = (t.sim().tick(), pos(&t, 1));
    assert!(
        judged_north.len() >= 3,
        "rejected, and again as it walked: {judged_north:?}"
    );
    assert!(
        at.y > 11.0 * 64.0 && at.x > 12.0 * 64.0,
        "round the wall, in the open: {at}"
    );
    let gap_then = gap(&t, 1, 2);
    assert!(
        gap_then > RANGE_SU && gap_then <= SIGHT_SU,
        "in its sight, out of its Range: {gap_then}"
    );
    assert!(t.sim().reach_counters().paths >= 1, "by a judgement");
    let last = judged_north.last().unwrap().from.unwrap();
    assert!(last.distance(at) >= REJUDGE_SELF_SU, "far from where it last judged it");
    while attack_ticks(&t, 1, 2).is_empty() {
        assert!(t.sim().tick() < took + 100, "never attacked\n{}", t.ascii(None));
        t.step();
    }

    // A fresh Guard on the same spot, with the same order: it takes the building in the tick of its order.
    let mut fresh = l_wall(at);
    assert_eq!(fresh.sim().tick(), 2);
    assert_eq!(engaged(&fresh, 1), Some(id(2)), "taken at once");
    assert!(rejection(&fresh, 1, 2).is_none());
    assert!(gap(&fresh, 1, 2) > RANGE_SU, "out of its Range");
    while attack_ticks(&fresh, 1, 2).is_empty() {
        assert!(fresh.sim().tick() < 100, "never attacked\n{}", fresh.ascii(None));
        fresh.step();
    }
}

/// A wall (row 3, from column 3 to the east edge) between a route along row 1 and row 5, as in
/// `attack_move_reach.rs`; the way round it is at its west end.
const WALL: &str = "
    ..............................
    ..............................
    ..............................
    ...~~~~~~~~~~~~~~~~~~~~~~~~~~~
    ..............................
    ..............................
    ..............................
";

/// What [`count_judgements`] saw.
#[derive(Debug, Default)]
struct Counted {
    /// Ticks in which unit 1's scan found a rejection it stood less than a grid unit from.
    near: u32,
    /// Rejections judged again because it stood a grid unit or more from where it judged them.
    again: u32,
    /// The most paths it planned in a tick.
    most: u32,
    /// Where it stood in the last tick its scan found a rejection.
    last_found: Option<Vec2>,
}

/// Steps until unit 1's AttackMove ends, checking tick by tick that it plans a judgement path for exactly the
/// rejections its own movement makes it judge again ([`due`]), up to [`JUDGE_PATHS_PER_TICK`], and none else: no
/// other enemy is judged meanwhile (each stood still, out of weapon range, and nothing else changed). Each one judged
/// again is rejected again, from where the unit stood.
fn count_judgements(t: &mut TestSim) -> Counted {
    let mut counted = Counted::default();
    while !is_idle(t, 1) {
        assert!(t.sim().tick() < 2000, "never arrived\n{}", t.ascii(None));
        let (at, due) = (pos(t, 1), due(t));
        let found = leg(t, 1).map_or(0, |leg| {
            (leg.rejected.iter())
                .filter(|judged| in_its_sight(t, judged.target.0))
                .count()
        });
        t.step();
        let paths = t.sim().reach_counters().paths;
        let tick = t.sim().tick();
        assert_eq!(
            paths as usize,
            due.len().min(JUDGE_PATHS_PER_TICK as usize),
            "tick {tick}: the Guard at {at} judges {due:?} again"
        );
        let judged: Vec<ObjectId> = (due.iter().copied())
            .filter(|&enemy| rejection(t, 1, enemy.0).is_some_and(|judged| judged.from == Some(at)))
            .collect();
        assert_eq!(judged.len() as u32, paths, "tick {tick}: rejected again from {at}");
        assert_eq!(engaged(t, 1), None, "tick {tick}: never taken");
        counted.near += u32::from(due.is_empty() && found > 0);
        counted.again += paths;
        counted.most = counted.most.max(paths);
        if found > 0 {
            counted.last_found = Some(at);
        }
    }
    counted
}

/// "The unit itself has moved a meaningful distance since it judged that enemy (placeholder 1 GU)." The Guard of
/// `attack_move_reach.rs`'s 0230 Verification attack-moves east along a wall, past an unarmed Drone of player 2
/// across it, in its sight and out of its Range; the way round the wall leaves its sight of the Drone at once. It
/// rejects the Drone, then, tick by tick ([`count_judgements`]): while it stands less than a grid unit from where it
/// judged, it plans no path for the Drone; once it stands a grid unit or more from there, exactly one, and rejects
/// the Drone again from where it stands. So it judges the Drone again for each grid unit it walks with the Drone in
/// its sight, 6 times as measured, and arrives.
#[test]
fn a_rejection_is_judged_again_once_the_unit_has_moved_a_grid_unit() {
    let mut t = ScenarioBuilder::new(WALL)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 1))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 5))) // 2
        .start();
    let to = centre(29, 1);
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .run_until(2);
    while rejection(&t, 1, 2).is_none() {
        assert!(t.sim().tick() < 500, "never judged");
        t.step();
    }
    let first = rejection(&t, 1, 2).unwrap();
    assert!(first.from.is_some(), "a rejection by the reach rule: {first:?}");
    let counted = count_judgements(&mut t);
    assert_eq!(pos(&t, 1), to, "arrived");
    assert!(
        counted.near >= 6 * 12,
        "less than a grid unit from where it judged, in most ticks: {counted:?}"
    );
    // Each judgement a grid unit, and at most a tick's walk more, from the one before.
    let walked = counted.last_found.unwrap().distance(first.from.unwrap());
    let speed = 5.0;
    let fewest = (walked / (REJUDGE_SELF_SU + speed)).floor() as u32;
    let most = (walked / REJUDGE_SELF_SU).floor() as u32;
    assert!(
        (fewest..=most).contains(&counted.again) && counted.again >= 5,
        "{walked} su walked with the Drone in its sight: {counted:?}"
    );
    assert!(attack_ticks(&t, 1, 2).is_empty());
}

/// "Exactly one per covered entry, within the budget": the scene of
/// [`a_rejection_is_judged_again_once_the_unit_has_moved_a_grid_unit`] with twelve Drones across the wall, all in
/// the Guard's sight as its AttackMove begins (`attack_move_reach.rs`'s cost bound). It judges
/// [`JUDGE_PATHS_PER_TICK`] of them a tick, rejects each, and then, as it walks on, judges each again once it stands
/// a grid unit from where it judged it, two a tick at most: each as it comes due, or in the next ticks while more
/// than two are due ([`count_judgements`]).
#[test]
fn several_rejections_are_judged_again_within_the_budget() {
    let mut builder = ScenarioBuilder::new(WALL).spawn(GUARD, Some(P1), SpawnAt::Point(centre(15, 1))); // 1
    for i in 0..12 {
        let at = centre(15, 5) + Vec2::new(40.0 * (i as f32 - 5.5), 0.0);
        builder = builder.spawn(DRONE, Some(P2), SpawnAt::Point(at)); // 2 to 13
    }
    let mut t = builder.start();
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(centre(29, 1)), false)
        .run_until(1);
    while !(2..=13).all(|drone| rejection(&t, 1, drone).is_some()) {
        assert!(t.sim().tick() < 20, "never judged them all");
        t.step();
    }
    let counted = count_judgements(&mut t);
    assert_eq!(counted.most, JUDGE_PATHS_PER_TICK, "at the bound: {counted:?}");
    assert!(
        counted.again >= 12 * 3,
        "each judged again as the Guard walks on: {counted:?}"
    );
}

/// A corridor one tile wide along row 1, between water, from column 8 east; the water of row 2 runs on to the east
/// edge, and row 5 is open ground.
const CORRIDOR: &str = "
    ........~~~~~~~~~~~~~~~~~~~~~~
    ..............................
    ........~~~~~~~~~~~~~~~~~~~~~~
    ..............................
    ..............................
    ..............................
    ..............................
";

/// "A unit standing still with a rejection plans no new judgement path." The Guard attack-moves east along the
/// corridor of [`CORRIDOR`], past an unarmed Drone of player 2 south of it, across the water, in its sight and out of
/// its Range: the way to it leads back west out of the corridor, out of the Guard's sight of it. A neutral Guard
/// blocks the corridor (it never yields, and nobody attacks it), so the Guard stops behind it, within a grid unit of
/// where it last judged the Drone, and stands there, but for its sideways tries along the blocker's face, until its
/// move gives up by the Failure rule, 90 ticks later. All that while it keeps its rejection of the Drone, which stays
/// in its sight, and plans no path for it. A neutral unit gives no vision and isn't a ValidTarget (design change
/// 0101), so nothing else is judged.
#[test]
fn a_unit_standing_still_judges_no_rejection_again() {
    let mut t = ScenarioBuilder::new(CORRIDOR)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 1))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(16, 5))) // 2
        .spawn(GUARD, None, SpawnAt::Point(centre(18, 1))) // 3
        .start();
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(centre(29, 1)), false)
        .run_until(2);
    // Up to the neutral Guard: it judges the Drone again for each grid unit it walks with the Drone in its sight.
    let mut still = 0;
    while still < 8 {
        assert!(t.sim().tick() < 500, "never stopped\n{}", t.ascii(None));
        let before = pos(&t, 1);
        t.step();
        still = if pos(&t, 1) == before { still + 1 } else { 0 };
    }
    let judged = rejection(&t, 1, 2).expect("rejected");
    let stopped = pos(&t, 1);
    assert!(judged.from.is_some_and(|from| from.distance(stopped) < REJUDGE_SELF_SU));
    assert!(
        stopped.x < centre(18, 1).x && gap(&t, 1, 3) <= 1.0,
        "behind the neutral Guard: {stopped}"
    );
    let mut standing = 0;
    loop {
        assert!(t.sim().tick() < 800, "never gave up\n{}", t.ascii(None));
        assert!(in_its_sight(&t, 2), "the Drone is in its sight");
        t.step();
        let tick = t.sim().tick();
        assert_eq!(t.sim().reach_counters().paths, 0, "tick {tick}");
        if is_idle(&t, 1) {
            break;
        }
        // Only its sideways tries along the blocker's face move it.
        let at = pos(&t, 1);
        assert!(
            at.x == stopped.x && (at.y - stopped.y).abs() <= 24.0,
            "tick {tick}: it stands, at {at}"
        );
        assert!(judged.from.is_some_and(|from| from.distance(at) < REJUDGE_SELF_SU));
        assert_eq!(rejection(&t, 1, 2), Some(judged), "tick {tick}: its rejection stands");
        standing += 1;
    }
    assert!(standing >= 80, "it stood {standing} ticks");
    assert!(attack_ticks(&t, 1, 2).is_empty() && attack_ticks(&t, 1, 3).is_empty());
}

/// "Covered: reach-rule rejections, and chases that end beyond the leash", the leash being part of the reach rule
/// (design change 0230). In the open, the Guard attack-moving east along row 1 takes the Drone 4 rows south of its
/// route, within its reach, and goes after it. Pushed aside (a teleport) beyond the leash, 7.1 grid units from its
/// route, with the Drone in its sight and out of its Range, it lets go: a rejection by the reach rule, where it
/// stands. It walks back towards its route, judging nothing while beyond the leash; on the leash again, more than a
/// grid unit from where it let go, it judges the Drone again, takes it and shoots it, as a fresh Guard there would.
/// With 0230's triggers alone it walked on: the Drone stood, its owner knew the same obstacles, the route was the same.
#[test]
fn a_chase_that_ended_beyond_the_leash_is_judged_again_once_the_unit_has_moved() {
    let open = "..............................\n".repeat(12);
    let mut t = ScenarioBuilder::new(&open)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 1))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 5))) // 2
        .start();
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(centre(29, 1)), false)
        .run_until(2);
    while !(engaged(&t, 1) == Some(id(2)) && leg(&t, 1).is_some_and(|leg| leg.chase.is_some())) {
        assert!(t.sim().tick() < 300, "never went after the Drone\n{}", t.ascii(None));
        t.step();
    }
    let chase = leg(&t, 1).unwrap().chase.unwrap();
    assert_eq!(chase.from, None, "a chase's judgement keeps 0230's triggers");
    let beyond = Vec2::new(1142.0, 552.0);
    let pushed = t.sim().tick() + 1;
    t.debug_at(pushed, DebugCommand::SetPosition { id: id(1), pos: beyond })
        .run_until(pushed);
    assert!(
        gap(&t, 1, 2) > RANGE_SU && gap(&t, 1, 2) <= SIGHT_SU,
        "{}",
        gap(&t, 1, 2)
    );
    assert_eq!(engaged(&t, 1), None, "let go");
    let judged = rejection(&t, 1, 2).expect("rejected");
    assert_eq!(judged.from, Some(beyond), "by the reach rule, where it stood");
    while engaged(&t, 1).is_none() {
        assert!(!is_idle(&t, 1), "it walked on, and never took the Drone again");
        assert!(t.sim().tick() < pushed + 200, "never took it again\n{}", t.ascii(None));
        t.step();
        if leg(&t, 1).is_some_and(|leg| leg.engaged.is_none()) {
            assert_eq!(rejection(&t, 1, 2), Some(judged), "tick {}", t.sim().tick());
        }
    }
    assert!(pos(&t, 1).distance(beyond) >= REJUDGE_SELF_SU, "{}", pos(&t, 1));
    assert!(t.sim().reach_counters().paths >= 1, "by a judgement");
    while attack_ticks(&t, 1, 2).is_empty() {
        assert!(t.sim().tick() < pushed + 400, "never shot it\n{}", t.ascii(None));
        t.step();
    }
}
