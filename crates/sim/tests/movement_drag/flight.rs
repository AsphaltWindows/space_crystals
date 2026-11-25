//! The Drone in flight (`systems/units.md > DragMovement`): its speed curve and equilibrium, braking and exact
//! arrival, turning as it moves, a Stop in full flight, and coming to rest within reach of a patch and a node.

use super::*;

/// Acceptance, **Speed** and **Stopping and arrival**: from rest and facing its way, the Drone's speed follows the
/// scheme `v ← v + 1.2 − 0.15 v` (1.2, 2.22, 3.087, …, `8 × (1 − 0.85ⁿ)`, design change 0205), comes within 1 % of
/// 8 space units per tick and never passes it. It begins to brake when its stopping distance reaches the path that
/// is left, loses speed in every tick from there, and lands exactly on its goal: no overshoot, no step back.
#[test]
fn a_drone_accelerates_to_eight_brakes_and_lands_on_its_goal() {
    let mut t = flat(24, 5).spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2))).start();
    let goal = centre(20, 2); // 1216 su east; the Drone faces east
    move_to(&mut t, 1, goal);
    let mut speeds = Vec::new();
    while t.status(1) != MoveStatus::Arrived {
        let d = step(&mut t);
        assert!(
            d.x >= 0.0 && d.y == 0.0,
            "tick {}: straight on, never back: {d}",
            t.sim().tick()
        );
        assert!(t.pos(1).x <= goal.x, "tick {}: past its goal", t.sim().tick());
        speeds.push(d.x);
        assert!(speeds.len() < 400, "{}", t.ascii(None));
    }
    assert_eq!(t.pos(1), goal, "exactly on it");
    assert_eq!(t.mover(1).momentum, Vec2::ZERO, "at rest");
    assert_eq!(t.mover(1).locomotion, Locomotion::Stationary);
    // The scheme, to the precision of positions around x = 100.
    let mut power = 1.0;
    for (n, &speed) in speeds.iter().take(60).enumerate() {
        power *= KEEP;
        assert!((speed - TOP * (1.0 - power)).abs() < 2e-3, "tick {}: {speed}", n + 1);
    }
    let fastest = speeds.iter().copied().fold(0.0, f32::max);
    assert!((TOP * 0.99..=TOP).contains(&fastest), "{fastest}");
    // It brakes once, at the end, within the documented bound.
    let brakes_from = speeds.iter().rposition(|&s| s >= fastest - 1e-3).expect("it cruised") + 1;
    let braking = &speeds[brakes_from..];
    assert!(braking.windows(2).all(|pair| pair[1] < pair[0]), "{braking:?}");
    let (distance, ticks) = (braking.iter().sum::<f32>(), braking.len() as u32);
    assert!(
        distance <= ARRIVES_WITHIN && ticks <= ARRIVES_IN,
        "{distance} su in {ticks} ticks"
    );
    // And it stays there.
    for _ in 0..20 {
        assert_eq!(step(&mut t), Vec2::ZERO);
    }
    // Slower than the stand-in was, 152 ticks at a constant 8. Measured: 164 ticks (between 165 and 175 before design
    // change 0205).
    assert!((160..=168).contains(&speeds.len()), "{} ticks", speeds.len());
}

/// Design change 0205's Verification, "A Drone starting from rest visibly reaches full speed in about 1.25 s", and
/// its numbers: "The Drone gets to about 65% of it after ~7 frames instead of 10, and close to full speed after ~20
/// frames instead of ~30." In a game, from rest and facing its way, measured: 67.9 % of its top speed after 7 ticks
/// (52.2 % before 0205), 96.1 % after 20, 1.25 s (87.8 % before), and within 1 % of it from the 29th (the 44th
/// before; `movement/drag/tests.rs` works out both curves).
#[test]
fn a_drone_gets_close_to_its_top_speed_in_a_second_and_a_quarter() {
    let mut t = flat(24, 5).spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2))).start();
    move_to(&mut t, 1, centre(20, 2));
    let speeds: Vec<f32> = (0..30).map(|_| step(&mut t).x).collect();
    let share = |tick: usize| speeds[tick - 1] / TOP;
    assert!(
        (0.65..=0.70).contains(&share(7)),
        "about 65 % after 7 ticks: {speeds:?}"
    );
    assert!(share(20) >= 0.96, "close to its top speed after 20 ticks: {speeds:?}");
    assert!(
        share(28) < 0.99 && share(29) >= 0.99,
        "within 1 % from the 29th: {speeds:?}"
    );
}

/// A Drone doesn't stand to turn (`systems/combat.md > LocomotionOrientationConstraints[DragMovement]`: it turns at
/// its TurnRate, 45° a tick, whatever it does): ordered behind itself, it thrusts from the first tick, turns as it
/// goes, and arrives exactly.
#[test]
fn a_drone_turns_as_it_moves() {
    let mut t = flat(16, 5)
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(12, 2)))
        .start();
    let goal = centre(2, 2); // west; the Drone faces east
    move_to(&mut t, 1, goal);
    let mut facings = Vec::new();
    for _ in 0..5 {
        let d = step(&mut t);
        assert!(d.x < 0.0, "it moves in its first tick: {d}");
        facings.push(t.sim().object(ObjectId(1)).unwrap().facing);
    }
    let quarter = core::f32::consts::FRAC_PI_4;
    for (n, facing) in facings.iter().take(4).enumerate() {
        assert!((facing.abs() - quarter * (n + 1) as f32).abs() < 1e-4, "{facings:?}");
    }
    // West, give or take the sideways speed its thrust gave it while it turned, which it is still taking back.
    let west = |facing: f32| core::f32::consts::PI - facing.abs() < 0.01;
    assert!(west(facings[3]) && west(facings[4]), "{facings:?}");
    for _ in 0..200 {
        t.step();
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
}

/// Invariants I5 and I7, amended: a Drone stopped in full flight has no goal and brakes to a stop, within the
/// documented bound. It is `Stopping` and slower every tick, then `Stationary` for good.
///
/// Design change 0205's Verification, "It stops with less drift than before": measured, it comes to rest 25.5 su
/// on, after 9 ticks of braking, where it took 41.5 su and 15 ticks before 0205.
#[test]
fn a_drone_stopped_in_full_flight_brakes_to_a_stop() {
    let mut t = flat(24, 5).spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2))).start();
    move_to(&mut t, 1, centre(22, 2));
    t.run_until(60);
    let fast = t.mover(1).momentum.length();
    assert!(fast > 7.9, "{fast}");
    t.order_at(61, P1, &[ObjectId(1)], OrderKind::Stop, false);
    let from = t.pos(1);
    let (mut last, mut ticks) = (fast, 0);
    loop {
        let d = step(&mut t);
        let mover = t.mover(1);
        assert_eq!(mover.goal, MoveGoal::None);
        if d == Vec2::ZERO {
            break;
        }
        ticks += 1;
        assert_eq!(mover.locomotion, Locomotion::Stopping, "tick {}", t.sim().tick());
        assert!(
            d.x > 0.0 && d.x < last,
            "tick {}: slower every tick: {d}",
            t.sim().tick()
        );
        assert_eq!((mover.velocity, mover.momentum), (d, d));
        last = d.x;
    }
    let stopped_in = t.pos(1).x - from.x;
    assert!(
        (25.0..=STOPS_WITHIN).contains(&stopped_in) && ticks <= STOPS_IN,
        "{stopped_in} su in {ticks} ticks"
    );
    let (drifted, took) = STOPPED_BEFORE_0205;
    assert!(
        stopped_in < drifted * 0.65 && ticks < took,
        "less drift than before 0205: {stopped_in} su in {ticks} ticks"
    );
    assert_eq!(t.mover(1).locomotion, Locomotion::Stationary);
    assert_eq!((t.mover(1).velocity, t.mover(1).momentum), (Vec2::ZERO, Vec2::ZERO));
    for _ in 0..20 {
        assert_eq!(step(&mut t), Vec2::ZERO, "and stays");
    }
}

/// Acceptance, **Stopping and arrival**: "A Drone arrives … within the 8-su reach of a patch without overshoot or
/// oscillation." On the reference trip (design change 0107) it never turns back on its way to the patch or to the
/// node, it is at rest when it begins to mine and while it mines, and it never touches either: it stops by braking,
/// not against them.
#[test]
fn a_drone_comes_to_rest_within_reach_of_its_patch_and_its_node() {
    // Patch 2 at tile (5, 2), node 3 two grid units south of it, the Drone (1) between them.
    let mut t = flat(16, 10)
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(5, 4)))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(5, 2)))
        .spawn(ObjectKind::EgressNode, Some(P1), SpawnAt::Tile(TilePos::new(5, 5)))
        .start();
    t.order_at(1, P1, &[ObjectId(1)], OrderKind::Gather(ObjectId(2)), false);
    let phase = |t: &TestSim| {
        let orders = t.sim().object(ObjectId(1)).unwrap().orders.as_ref().unwrap();
        match orders.current.as_ref().map(|active| &active.state) {
            Some(Behaviour::Gather(gather)) => Some(gather.phase),
            _ => None,
        }
    };
    // The patch's south edge and the node's north edge: the Drone's box is between them.
    let (patch_edge, node_edge) = (3.0 * 64.0 + HALF, 5.0 * 64.0 - HALF);
    let (mut mined, mut unloaded) = (0, 0);
    // Whether it has set off on the leg it is on: a leg begins with the last of the braking of the one before,
    // since unloading takes 2 ticks and stopping more.
    let mut under_way = false;
    for _ in 0..480 {
        let before = phase(&t);
        let d = step(&mut t);
        let (y, now) = (t.pos(1).y, phase(&t));
        assert!(
            y > patch_edge && y < node_edge,
            "tick {}: it touched: {y}",
            t.sim().tick()
        );
        if before.as_ref().map(std::mem::discriminant) != now.as_ref().map(std::mem::discriminant) {
            under_way = false;
        }
        match now {
            Some(GatherPhase::ToPatch) => {
                assert!(
                    !under_way || d.y <= 0.0,
                    "tick {}: back from the patch: {d}",
                    t.sim().tick()
                );
                under_way |= d.y < 0.0;
            }
            Some(GatherPhase::ToDropOff) => {
                assert!(
                    !under_way || d.y >= 0.0,
                    "tick {}: back from the node: {d}",
                    t.sim().tick()
                );
                under_way |= d.y > 0.0;
            }
            Some(GatherPhase::Mining { .. }) => {
                // It begins to mine as it comes within reach, with the last of its braking still to do.
                assert!(y - patch_edge <= 8.0, "within MiningReach: {}", y - patch_edge);
                if matches!(before, Some(GatherPhase::Mining { done }) if done >= 10) {
                    assert_eq!(d, Vec2::ZERO, "tick {}: at rest while it mines", t.sim().tick());
                }
                mined += u32::from(!matches!(before, Some(GatherPhase::Mining { .. })));
            }
            Some(GatherPhase::DroppingOff { .. }) => {
                assert!(node_edge - y <= 8.0, "within DropOffReach: {}", node_edge - y);
                unloaded += u32::from(!matches!(before, Some(GatherPhase::DroppingOff { .. })));
            }
            _ => {}
        }
    }
    assert!(mined >= 5 && unloaded >= 5, "{mined} and {unloaded}");
}
