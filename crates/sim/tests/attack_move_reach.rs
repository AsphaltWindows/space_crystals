//! AttackMove's reach rule (`systems/controls/command_behaviors.md > AttackMovingToLocation > Algorithm` 2, design
//! change 0230, answering question Q0058): "an attack-moving unit fights what it can reach without leaving its route or
//! losing sight of it." An enemy within weapon range is always taken; one out of range only if the path to a firing
//! position stays within the leash of the PathReference and keeps the target within the unit's SightRange. There is
//! no memory of the enemies passed over: a rejection only saves judging again while nothing it depends on has
//! changed (`orders::reach`), the unit's own movement included (design change 0243: its tests are
//! `attack_move_own_movement.rs`'s). The leash itself is `attack_move_leash.rs`'s.

use sc_sim::orders::state::{Judged, Leg};
use sc_sim::orders::{Behaviour, JUDGE_PATHS_PER_TICK};
use sc_sim::testing::{ScenarioBuilder, TestSim};
const REJUDGE_MOVE_SU: f32 = sc_sim::tuning::orders::BUILTIN.rejudge_move_su;
const REJUDGE_SELF_SU: f32 = sc_sim::tuning::orders::BUILTIN.rejudge_self_su;
use sc_sim::{DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerId, Shape, SimEvent, SpawnAt, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
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

fn attack_ticks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == id(attacker)))
        .map(|&(tick, _)| tick)
        .collect()
}

fn teleport(t: &mut TestSim, n: u32, to: Vec2) {
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::SetPosition { id: id(n), pos: to })
        .run_until(tick);
}

/// What a walk of unit 1 came to ([`walk`]).
#[derive(Debug, Default)]
struct Walk {
    /// How often it took each enemy, by id: the times it went from not engaged with it to engaged.
    taken: Vec<(ObjectId, u32)>,
    /// Paths planned to judge enemies, in all, and the most in one tick.
    paths: u32,
    most_paths: u32,
    /// Rejections judged again for the unit's own movement (design change 0243): replaced by one of the same enemy,
    /// standing where it stood, judged a grid unit or more from where the unit judged the one before.
    again: u32,
    /// It never stepped back west.
    never_back: bool,
}

impl Walk {
    fn times_taken(&self, enemy: u32) -> u32 {
        (self.taken.iter().find(|&&(taken, _)| taken == id(enemy))).map_or(0, |&(_, times)| times)
    }
}

/// Steps until `done` (at most `limit` ticks from now), watching unit 1's AttackMove.
fn walk(t: &mut TestSim, limit: Tick, done: impl Fn(&TestSim) -> bool) -> Walk {
    let mut seen = Walk {
        never_back: true,
        ..Walk::default()
    };
    let until = t.sim().tick() + limit;
    let mut was = engaged(t, 1);
    while !done(t) {
        assert!(
            t.sim().tick() < until,
            "not within {limit} ticks: {seen:?}\n{}",
            t.ascii(None)
        );
        let before = pos(t, 1).x;
        let rejected = leg(t, 1).map_or_else(Vec::new, |leg| leg.rejected.clone());
        t.step();
        let again = |now: &Judged| {
            (rejected.iter()).any(|was| {
                (was.target, was.at) == (now.target, now.at)
                    && (was.from.zip(now.from)).is_some_and(|(was, now)| was.distance(now) >= REJUDGE_SELF_SU)
            })
        };
        seen.again += leg(t, 1).map_or(0, |leg| leg.rejected.iter().filter(|now| again(now)).count() as u32);
        let counters = t.sim().reach_counters();
        assert!(
            counters.most_per_unit <= JUDGE_PATHS_PER_TICK,
            "tick {}: {counters:?}",
            t.sim().tick()
        );
        seen.paths += counters.paths;
        seen.most_paths = seen.most_paths.max(counters.paths);
        seen.never_back &= pos(t, 1).x >= before;
        let now = engaged(t, 1);
        if now != was
            && let Some(target) = now
        {
            match seen.taken.iter_mut().find(|(taken, _)| *taken == target) {
                Some((_, times)) => *times += 1,
                None => seen.taken.push((target, 1)),
            }
        }
        was = now;
    }
    seen
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

/// A Guard of player 1 on the route of [`WALL`] and an unarmed Drone of player 2 across the wall, 4 rows south: in
/// the Guard's sight as it passes, out of its Range. The Guard attack-moves east along row 1, from tick 2.
fn past_the_drone() -> (TestSim, Vec2) {
    let mut t = ScenarioBuilder::new(WALL)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 1))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 5))) // 2
        .start();
    let to = centre(29, 1);
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .run_until(2);
    (t, to)
}

/// Design change 0230's Verification, and question Q0058's loop: "A Guard attack-moves past an unarmed Drone behind
/// a wall. If the only way to the Drone leaves the leash or loses sight of it, the Guard walks on without turning
/// back, and arrives." The way to the Drone leads back west round the wall, out of the Guard's sight of it at once.
/// Taken whenever seen (question Q0058: 78 times in 1500 ticks), the Guard would never arrive. It never takes it,
/// judges it by one path, again by one for each grid unit it walks with the Drone in its sight (design change 0243),
/// and walks on.
#[test]
fn a_guard_walks_on_past_a_drone_it_would_lose_sight_of_on_the_way() {
    let (mut t, to) = past_the_drone();
    let seen = walk(&mut t, 1000, |t| rejection(t, 1, 2).is_some());
    assert_eq!(seen.paths, 1, "judged by one path");
    let judged = rejection(&t, 1, 2).unwrap();
    assert_eq!(judged.at, centre(15, 5));
    assert!(
        gap(&t, 1, 2) > 3.0 * 64.0 && gap(&t, 1, 2) <= 5.0 * 64.0,
        "in sight, out of range"
    );
    let rest = walk(&mut t, 1000, |t| is_idle(t, 1));
    assert_eq!(pos(&t, 1), to, "arrived");
    assert_eq!((seen.times_taken(2), rest.times_taken(2)), (0, 0), "never taken");
    assert!(seen.never_back && rest.never_back, "without turning back");
    // "Otherwise a rejected enemy costs nothing more": while the Drone stood still and the obstacles stayed the same,
    // paths were planned for it only for the Guard's own movement.
    assert!(rest.again > 0 && rest.paths == rest.again, "{rest:?}");
    assert!(pos(&t, 2) == centre(15, 5) && attack_ticks(&t, 1).is_empty());
}

/// Two walls (rows 2 and 6, from column 3 to the east edge) on either side of a route along row 4; the ways round
/// them are at their west ends.
const WALLS: &str = "
    ..............................
    ..............................
    ...~~~~~~~~~~~~~~~~~~~~~~~~~~~
    ..............................
    ..............................
    ..............................
    ...~~~~~~~~~~~~~~~~~~~~~~~~~~~
    ..............................
    ..............................
";

/// The take-and-drop loop with two targets (the soak's seed 29): a Guard attack-moves east along row 4, between two
/// walls, and two unarmed enemy Drones stand across them, one north and one south, as far off. The way to either
/// leads back west, out of sight of both. The Guard takes neither, judges each by one path, and again only for its
/// own movement, and walks on.
#[test]
fn two_drones_across_two_walls_are_judged_and_passed() {
    let mut t = ScenarioBuilder::new(WALLS)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 4))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 0))) // 2
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 8))) // 3
        .start();
    let to = centre(29, 4);
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .run_until(2);
    let seen = walk(&mut t, 1500, |t| is_idle(t, 1));
    assert_eq!(pos(&t, 1), to, "arrived");
    assert!(seen.taken.is_empty(), "{seen:?}");
    assert_eq!(
        seen.paths - seen.again,
        2,
        "one path each, and again only for its own movement: {seen:?}"
    );
    assert!(seen.never_back && attack_ticks(&t, 1).is_empty());
}

/// "An enemy within weapon range is always taken", a rejected one too. The Drone of
/// [`a_guard_walks_on_past_a_drone_it_would_lose_sight_of_on_the_way`], rejected, steps up to the wall as the Guard
/// passes it: 40 su, less than the distance that has it judged again, so the rejection stands. Now within the
/// Guard's Range across the wall, it is taken at once and shot.
#[test]
fn a_rejected_enemy_that_comes_within_weapon_range_is_taken() {
    let (mut t, to) = past_the_drone();
    walk(&mut t, 1000, |t| (pos(t, 1).x - centre(15, 5).x).abs() <= 5.0);
    assert!(rejection(&t, 1, 2).is_some() && engaged(&t, 1).is_none());
    assert!(gap(&t, 1, 2) > 3.0 * 64.0, "out of range: {}", gap(&t, 1, 2));
    let nearer = centre(15, 5) - v(0.0, 40.0);
    assert!(nearer.distance(centre(15, 5)) < REJUDGE_MOVE_SU);
    teleport(&mut t, 2, nearer);
    let seen = walk(&mut t, 3, |t| engaged(t, 1) == Some(id(2)));
    assert!(gap(&t, 1, 2) <= 3.0 * 64.0, "within range: {}", gap(&t, 1, 2));
    assert_eq!(
        rejection(&t, 1, 2).map(|judged| judged.at),
        Some(centre(15, 5)),
        "its rejection stands"
    );
    assert_eq!(seen.paths, 0, "taken without a judgement");
    walk(&mut t, 300, |t| t.sim().object(id(2)).is_none());
    assert!(!attack_ticks(&t, 1).is_empty());
    walk(&mut t, 1000, |t| is_idle(t, 1));
    assert_eq!(pos(&t, 1), to, "and it went on");
}

/// "There is no memory of dropped targets (the same enemy, or an identical one swapped in unseen, must get the same
/// response)." The Drone the Guard passed over is taken out of the game and an identical one put in its place: the
/// Guard judges the new one as it judged the first, by a path of its own, and passes it too. Then the same in the
/// open: a Drone the Guard can reach is taken, and so is the identical one that takes its place.
#[test]
fn an_identical_enemy_swapped_in_gets_the_same_response() {
    let (mut t, to) = past_the_drone();
    let first = walk(&mut t, 1000, |t| rejection(t, 1, 2).is_some());
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::Remove { id: id(2) })
        .spawn_unit_at(tick, DRONE, P2, centre(15, 5)) // 3
        .run_until(tick);
    assert!(rejection(&t, 1, 2).is_none(), "nothing is kept of the one that is gone");
    // The new one is judged in the tick it appears in.
    assert_eq!(
        (first.paths, t.sim().reach_counters().paths),
        (1, 1),
        "each judged by a path"
    );
    assert_eq!(rejection(&t, 1, 3).map(|judged| judged.at), Some(centre(15, 5)));
    let rest = walk(&mut t, 1000, |t| is_idle(t, 1));
    assert_eq!(pos(&t, 1), to, "arrived");
    assert!(first.taken.is_empty() && rest.taken.is_empty() && engaged(&t, 1).is_none());

    // In the open, 4 rows south of the route: within reach.
    let open = "..............................\n".repeat(9);
    let mut t = ScenarioBuilder::new(&open)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 1))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 5))) // 2
        .start();
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(centre(29, 1)), false)
        .run_until(2);
    walk(&mut t, 1000, |t| engaged(t, 1) == Some(id(2)));
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::Remove { id: id(2) })
        .spawn_unit_at(tick, DRONE, P2, centre(15, 5)) // 3
        .run_until(tick);
    walk(&mut t, 5, |t| engaged(t, 1) == Some(id(3)));
    walk(&mut t, 300, |t| t.sim().object(id(3)).is_none());
    walk(&mut t, 1000, |t| is_idle(t, 1));
    assert_eq!(pos(&t, 1), centre(29, 1));
}

/// "Re-checking a rejected enemy: only when something relevant changes, i.e. the enemy moves a meaningful distance".
/// The rejected Drone moves 30 su along the wall: nothing is planned for it. It moves on, to 100 su from where it was
/// judged: it is judged again, by one path, and rejected where it stands now. Then design change 0230's
/// Verification: "If the Drone then moves into the open within reach, the Guard engages it."
#[test]
fn an_enemy_is_judged_again_once_it_has_moved_a_grid_unit() {
    let (mut t, _) = past_the_drone();
    let first = walk(&mut t, 1000, |t| rejection(t, 1, 2).is_some());
    assert_eq!(first.paths, 1);
    let at = centre(15, 5);
    teleport(&mut t, 2, at - v(30.0, 0.0));
    let until = t.sim().tick() + 6;
    let near = walk(&mut t, 7, |t| t.sim().tick() >= until);
    assert_eq!(near.paths, 0, "less than a grid unit: its rejection stands");
    assert_eq!(rejection(&t, 1, 2).map(|judged| judged.at), Some(at));
    teleport(&mut t, 2, at - v(100.0, 0.0));
    // Judged again in the tick it is found to have moved, by one path.
    assert_eq!(t.sim().reach_counters().paths, 1);
    assert_eq!(rejection(&t, 1, 2).map(|judged| judged.at), Some(at - v(100.0, 0.0)));
    assert_eq!(engaged(&t, 1), None);
    // Into the open: on the Guard's side of the wall, ahead of it, in its sight and out of its Range.
    let ahead = pos(&t, 1) + v(18.0 + 230.0 + 18.0, 0.0);
    teleport(&mut t, 2, ahead);
    assert_eq!(engaged(&t, 1), Some(id(2)), "engaged");
    walk(&mut t, 300, |t| t.sim().object(id(2)).is_none());
    assert!(!attack_ticks(&t, 1).is_empty());
}

/// The wall of [`WALL`] with a gap of two tiles (columns 14 and 15).
const GAP: &str = "
    ..............................
    ..............................
    ..............................
    ...~~~~~~~~~~~..~~~~~~~~~~~~~~
    ..............................
    ..............................
    ..............................
";

/// "…or the obstacle knowledge the path used changes (PathKnowledge)." A Generator of the Guard's own side fills the
/// gap in the wall, so the way to the Drone across it leads round the wall's west end, out of sight: rejected. The
/// Generator leaves the game: its owner's path knowledge changes, the Drone is judged again in the next tick, and
/// the way through the gap is one the Guard can finish: it takes the Drone and destroys it.
#[test]
fn an_enemy_is_judged_again_when_the_known_obstacles_change() {
    let mut t = ScenarioBuilder::new(GAP)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(4, 1))) // 1
        .spawn(DRONE, Some(P2), SpawnAt::Point(centre(15, 5))) // 2
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(14, 3))) // 3
        // Another structure of player 1's, so that it stays in the game without the first.
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(0, 5))) // 4
        .start();
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(centre(29, 1)), false)
        .run_until(2);
    let first = walk(&mut t, 1000, |t| rejection(t, 1, 2).is_some());
    assert_eq!((first.paths, first.times_taken(2)), (1, 0));
    let until = t.sim().tick() + 10;
    let waited = walk(&mut t, 11, |t| t.sim().tick() >= until);
    assert_eq!(waited.paths, 0);
    let version = t.sim().path_knowledge().version(P1);
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::Remove { id: id(3) }).run_until(tick);
    assert_ne!(
        t.sim().path_knowledge().version(P1),
        version,
        "its owner knows the gap is open"
    );
    let through = walk(&mut t, 2, |t| engaged(t, 1) == Some(id(2)));
    assert_eq!(through.paths, 1, "judged again, by one path");
    walk(&mut t, 300, |t| t.sim().object(id(2)).is_none());
    assert!(!attack_ticks(&t, 1).is_empty());
}

/// The cost bound: "plan at most a fixed number of judgement paths per unit per tick; judge later candidates on later
/// ticks". Twelve Drones across the wall, all in the Guard's sight as its AttackMove begins: it judges
/// [`JUDGE_PATHS_PER_TICK`] of them per tick ([`walk`] checks every tick), each once, takes none and walks on,
/// judging them again only for its own movement, within the bound still.
#[test]
fn a_unit_judges_a_bounded_number_of_enemies_per_tick() {
    let mut builder = ScenarioBuilder::new(WALL).spawn(GUARD, Some(P1), SpawnAt::Point(centre(15, 1))); // 1
    for i in 0..12 {
        let at = centre(15, 5) + v(40.0 * (i as f32 - 5.5), 0.0);
        builder = builder.spawn(DRONE, Some(P2), SpawnAt::Point(at)); // 2 to 13
    }
    let mut t = builder.start();
    for drone in 2..=13 {
        assert!(gap(&t, 1, drone) <= 5.0 * 64.0 && gap(&t, 1, drone) > 3.0 * 64.0);
    }
    let to = centre(29, 1);
    t.order_at(2, P1, &[id(1)], OrderKind::AttackMove(to), false)
        .run_until(1);
    let all = |t: &TestSim| (2..=13).all(|drone| rejection(t, 1, drone).is_some());
    let judging = walk(&mut t, 20, all);
    assert_eq!(judging.paths, 12, "each judged once");
    assert_eq!(judging.most_paths, JUDGE_PATHS_PER_TICK, "at the bound");
    assert_eq!(
        t.sim().tick(),
        1 + 12_u32.div_ceil(JUDGE_PATHS_PER_TICK),
        "and in as few ticks as it allows"
    );
    let rest = walk(&mut t, 1000, |t| is_idle(t, 1));
    assert_eq!(pos(&t, 1), to, "arrived");
    assert!(judging.taken.is_empty() && rest.taken.is_empty());
    assert!(rest.again > 0 && rest.paths == rest.again, "{rest:?}");
}

/// A corridor one tile wide leads east from player 1's Deployment Center (tiles (0, 1) to (3, 4)): tile row 3, y
/// from 192 to 256, x from 256 on, between water.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~~~~~~~~~
    ....~~~~~~~~~~~~~~~~~~~~
    ....~~~~~~~~~~~~~~~~~~~~
    ........................
    ....~~~~~~~~~~~~~~~~~~~~
    ~~~~~~~~~~~~~~~~~~~~~~~~
    ........................
    ........................
";

/// Design change 0235's Verification (answering question Q0073): "Four Peacekeepers attack-move through a 1-tile
/// corridor at a building. The rear ones give up once (about 7 s), walk on, and don't turn back while the front one
/// still fires." The first stops in range of the Deployment Center and fires; the three behind can't get past it,
/// and their chases fail by the Failure rule (design change 0223). "A chase given up by the Failure rule puts the
/// enemy in the reach rule's rejected set": each of them takes the building once, keeps a rejection of it from
/// then on, and never takes it again while the first stands (with the 32-tick retry of old, each went round a lap
/// of 116 ticks for good). Their AttackMoves end. When the first unit leaves, the next one walks up and fires:
/// "Within weapon range it is always fought."
#[test]
fn in_a_corridor_the_units_behind_give_up_their_chase_once() {
    let mut builder = ScenarioBuilder::new(CORRIDOR)
        .spawn(
            ObjectKind::DeploymentCenter,
            Some(P1),
            SpawnAt::Tile(TilePos::new(0, 1)),
        ) // 1
        .spawn(ObjectKind::Generator, Some(P1), SpawnAt::Tile(TilePos::new(1, 6))); // 2
    for i in 0..4 {
        let at = v(540.0 + 40.0 * i as f32, 224.0);
        builder = builder.spawn(ObjectKind::Peacekeeper, Some(P2), SpawnAt::Point(at)); // 3 to 6
    }
    let mut t = builder.start();
    let units = [id(3), id(4), id(5), id(6)];
    t.order_at(2, P2, &units, OrderKind::AttackMove(v(280.0, 224.0)), false)
        .run_until(2);
    let engaged_with = |t: &TestSim, n: u32| leg(t, n).and_then(|leg| leg.engaged).map(|pursuit| pursuit.target);
    let mut taken = [0_u32; 4];
    let mut was = [None; 4];
    let mut gave_up = [None; 4];
    while t.sim().tick() < 900 {
        t.step();
        for (i, unit) in (3..=6).enumerate() {
            let now = engaged_with(&t, unit);
            taken[i] += u32::from(now == Some(id(1)) && was[i] != now);
            if was[i] == Some(id(1)) && now.is_none() && gave_up[i].is_none() {
                gave_up[i] = Some(t.sim().tick());
                assert!(
                    is_idle(&t, unit) || rejection(&t, unit, 1).is_some(),
                    "#{unit} keeps a rejection of the building"
                );
            }
            was[i] = now;
        }
    }
    assert!(attack_ticks(&t, 3).len() > 10, "the first fires all along");
    assert_eq!((taken[0], gave_up[0]), (1, None), "and never lets go");
    for (i, unit) in (4..=6).enumerate() {
        assert!(attack_ticks(&t, unit).is_empty(), "#{unit} can't get into range");
        assert_eq!(taken[i + 1], 1, "#{unit} takes the building once: {gave_up:?}");
        assert!(
            gave_up[i + 1].is_some_and(|tick| tick < 300),
            "#{unit} gives up its chase: {gave_up:?}\n{}",
            t.ascii(None)
        );
        assert!(is_idle(&t, unit), "#{unit}'s AttackMove ends\n{}", t.ascii(None));
    }
    // The first leaves: the next is in range once it has walked up, and fights.
    let gone = t.sim().tick() + 1;
    t.debug_at(gone, DebugCommand::Remove { id: id(3) });
    t.run_until(gone + 100);
    assert!(
        attack_ticks(&t, 4).first().is_some_and(|&tick| tick > gone),
        "the next one walks up and fires"
    );
}
