//! DragMovement in a game (slice M3-G, plan decision T25; `systems/units.md > DragMovement`, `> HoverVehicle`,
//! `> UnitCollision > Collision Response (Un-sticking)`; design changes 0035 and 0039): the Drone's speed curve and
//! its equilibrium, braking and exact arrival, a Stop in full flight, Rugged terrain at half speed, a wall met at
//! speed, arrival within reach of a patch, a save in full flight, and the amended invariants I5 and I7 from their
//! wrong side. The model's numbers alone are `movement/drag/tests.rs`. `TestSim` checks the invariants after every
//! tick.

use sc_sim::mover::Locomotion;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    MoveGoal, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Sim, SpawnAt, Target, TickInputs, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const DRONE: ObjectKind = ObjectKind::Drone;
/// Half the Drone's Silhouette.
const HALF: f32 = 18.0;
/// `factions/colonists/units.md > Drone`: the equilibrium of its thrust and drag, space units per tick.
const TOP: f32 = 8.0;
/// The documented bounds of `movement::drag`: from top speed the Drone stops within this many space units and
/// ticks when it is stopped…
const STOPS_WITHIN: f32 = 42.0;
const STOPS_IN: u32 = 16;
/// … and on its way to a goal, where it plans with a reserve, from the step in which it begins to brake: that
/// step, of at most 8, and the 43 su it stops in.
const ARRIVES_WITHIN: f32 = 51.0;
const ARRIVES_IN: u32 = 17;

fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn move_to(t: &mut TestSim, tick: u32, to: Vec2) {
    t.order_at(tick, P1, &[ObjectId(1)], OrderKind::Move(Target::Point(to)), false);
}

/// Steps once and returns the Drone's displacement.
fn step(t: &mut TestSim) -> Vec2 {
    let from = t.pos(1);
    t.step().pos(1) - from
}

/// Acceptance, **Speed** and **Stopping and arrival**: from rest and facing its way, the Drone's speed follows the
/// scheme `v ← v + 0.8 − 0.1 v` (0.8, 1.52, 2.168, …, `8 × (1 − 0.9ⁿ)`), comes within 1 % of 8 space units per tick
/// and never passes it. It begins to brake when its stopping distance reaches the path that is left, loses speed in
/// every tick from there, and lands exactly on its goal: no overshoot, no step back.
#[test]
fn a_drone_accelerates_to_eight_brakes_and_lands_on_its_goal() {
    let mut t = flat(24, 5).spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2))).start();
    let goal = centre(20, 2); // 1216 su east; the Drone faces east
    move_to(&mut t, 1, goal);
    let mut speeds = Vec::new();
    while t.status(1) != MoveStatus::Arrived {
        let d = step(&mut t);
        assert!(d.x >= 0.0 && d.y == 0.0, "tick {}: straight on, never back: {d}", t.sim().tick());
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
        power *= 0.9;
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
    // Slower than the stand-in was: 152 ticks at a constant 8.
    assert!((165..=175).contains(&speeds.len()), "{} ticks", speeds.len());
}

/// A Drone doesn't stand to turn (`systems/combat.md > LocomotionOrientationConstraints[DragMovement]`: it turns at
/// its TurnRate, 45° a tick, whatever it does): ordered behind itself, it thrusts from the first tick, turns as it
/// goes, and arrives exactly.
#[test]
fn a_drone_turns_as_it_moves() {
    let mut t = flat(16, 5).spawn(DRONE, Some(P1), SpawnAt::Point(centre(12, 2))).start();
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
        assert!(d.x > 0.0 && d.x < last, "tick {}: slower every tick: {d}", t.sim().tick());
        assert_eq!((mover.velocity, mover.momentum), (d, d));
        last = d.x;
    }
    let stopped_in = t.pos(1).x - from.x;
    assert!(
        (40.0..=STOPS_WITHIN).contains(&stopped_in) && ticks <= STOPS_IN,
        "{stopped_in} su in {ticks} ticks"
    );
    assert_eq!(t.mover(1).locomotion, Locomotion::Stationary);
    assert_eq!((t.mover(1).velocity, t.mover(1).momentum), (Vec2::ZERO, Vec2::ZERO));
    for _ in 0..20 {
        assert_eq!(step(&mut t), Vec2::ZERO, "and stays");
    }
}

/// Acceptance, **Rugged**, and 0039's Verification: "A Drone crosses a Rugged patch at roughly half speed and
/// speeds back up on leaving it." Its path leads straight through the strip (Rugged is no obstacle for a hover
/// unit); the drag is doubled while the tile under its centre is Rugged, so it settles at 4 there.
#[test]
fn a_drone_crosses_rugged_terrain_at_half_speed() {
    // Tiles 10 to 21 of the row are Rugged: 768 su.
    let row = format!("{}{}{}", ".".repeat(10), "r".repeat(12), ".".repeat(12));
    let mut t = ScenarioBuilder::new(&vec![row; 5].join("\n"))
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2)))
        .start();
    let goal = centre(32, 2);
    move_to(&mut t, 1, goal);
    let (mut before, mut inside, mut after) = (Vec::new(), Vec::new(), Vec::new());
    while t.status(1) != MoveStatus::Arrived {
        let d = step(&mut t);
        assert_eq!(d.y, 0.0, "straight through");
        let tile = TilePos::containing(t.pos(1)).x;
        match tile {
            ..10 => before.push(d.x),
            10..22 => inside.push(d.x),
            _ => after.push(d.x),
        }
        assert!(t.sim().tick() < 500, "{}", t.ascii(None));
    }
    assert_eq!(t.pos(1), goal);
    let fastest = |speeds: &[f32]| speeds.iter().copied().fold(0.0, f32::max);
    assert!(fastest(&before) > 7.9, "up to speed before the strip: {}", fastest(&before));
    // It comes in at 8 and is braked by the drag: within 1 % of 4 from a third of the strip on.
    let settled = &inside[inside.len() / 3..];
    assert!(settled.iter().all(|&s| (3.96..=4.04).contains(&s)), "{inside:?}");
    assert!(inside.windows(2).all(|pair| pair[1] <= pair[0] + 1e-3), "it only slows in there");
    // And back up to speed beyond it.
    assert!(fastest(&after) > 7.9, "{}", fastest(&after));
    assert!(after[..20].windows(2).all(|pair| pair[1] > pair[0]), "{after:?}");
}

/// Acceptance, **Collision** (design change 0035, the points that apply to a hover unit: "the blocked portion of
/// the movement is dropped … and its velocity is set to 0"): a Drone flying east at speed is sent round a corner
/// to the north. Its momentum carries it into the wall beyond the corner. It never overlaps the wall (I2, every
/// tick); against it, it has no velocity into it and keeps what it has along it, and it creeps on round the corner
/// and arrives.
#[test]
fn a_drone_that_meets_a_wall_at_speed_slides_along_it() {
    // An L-shaped passage: east along row 6, then north along column 14. The wall east of column 14 is Mountain.
    let map = [
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "...............M",
        "MMMMMMMMMMMMMMMM",
    ];
    let mut t = ScenarioBuilder::new(&map.join("\n"))
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 6)))
        .start();
    let goal = centre(14, 1);
    move_to(&mut t, 1, goal);
    // The east wall begins at x = 15 × 64: the Drone's centre rests 18 su before it.
    let against = 15.0 * 64.0 - HALF;
    let (mut came_at, mut slid) = (Vec2::ZERO, None);
    while t.status(1) != MoveStatus::Arrived {
        let before = t.mover(1).momentum;
        let d = step(&mut t);
        assert!(t.pos(1).x <= against, "tick {}: in the wall", t.sim().tick());
        if t.status(1) != MoveStatus::Arrived {
            assert_eq!(t.mover(1).momentum, d, "its momentum is the move it made");
        }
        if t.pos(1).x == against && slid.is_none() && before.x > 0.0 {
            // The tick it met the wall: it went as far as the wall, and on along it.
            came_at = before;
            slid = Some(d);
        } else if t.pos(1).x == against {
            assert_eq!(d.x, 0.0, "tick {}: no velocity into the wall", t.sim().tick());
        }
        assert!(t.sim().tick() < 400, "{}", t.ascii(None));
    }
    let slid = slid.expect("its momentum carried it into the wall");
    assert!(came_at.x > 6.0, "it came at speed: {came_at}");
    assert!(slid.x < came_at.x * 0.9 && slid.y < came_at.y, "{came_at} then {slid}");
    assert_eq!(t.pos(1), goal);
}

/// The 0035 creep itself: a Drone at rest against a wall, ordered to a point it can only reach along the wall,
/// starts "from 0", at single-tick acceleration, and gets away along it. Every step is at most what one tick's
/// thrust adds to what the drag left of the last.
#[test]
fn a_drone_pressed_against_a_wall_creeps_away_along_it() {
    let map = ["......", "......", "MMMMMM", "MMMMMM"];
    // Its box rests on the wall to the south.
    let start = Vec2::new(96.0, 2.0 * 64.0 - HALF);
    let mut t = ScenarioBuilder::new(&map.join("\n"))
        .spawn(DRONE, Some(P1), SpawnAt::Point(start))
        .start();
    let goal = Vec2::new(320.0, start.y);
    move_to(&mut t, 1, goal);
    let mut last = 0.0_f32;
    while t.status(1) != MoveStatus::Arrived {
        let d = step(&mut t);
        assert_eq!(d.y, 0.0, "along the wall");
        assert!(d.x <= last * 0.9 + 0.8 + 1e-3, "tick {}: {d} after {last}", t.sim().tick());
        last = d.x;
        assert!(t.sim().tick() < 200, "{}", t.ascii(None));
    }
    assert_eq!(t.pos(1), goal);
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
        assert!(y > patch_edge && y < node_edge, "tick {}: it touched: {y}", t.sim().tick());
        if before.as_ref().map(std::mem::discriminant) != now.as_ref().map(std::mem::discriminant) {
            under_way = false;
        }
        match now {
            Some(GatherPhase::ToPatch) => {
                assert!(!under_way || d.y <= 0.0, "tick {}: back from the patch: {d}", t.sim().tick());
                under_way |= d.y < 0.0;
            }
            Some(GatherPhase::ToDropOff) => {
                assert!(!under_way || d.y >= 0.0, "tick {}: back from the node: {d}", t.sim().tick());
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

/// The momentum is state (`Mover::momentum`, hashed and serialized): a game saved with a Drone in full flight and
/// one braking without a goal goes on exactly as the game that wasn't saved. A state without momentum doesn't name
/// the field, so states saved before it existed load, and hash as they did.
#[test]
fn a_save_in_full_flight_continues_identically() {
    let mut t = flat(24, 8)
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2)))
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 5)))
        .start();
    let at_rest = serde_json::to_string(t.sim()).unwrap();
    assert!(!at_rest.contains("momentum"), "left out while zero");
    move_to(&mut t, 1, centre(22, 2));
    t.order_at(1, P1, &[ObjectId(2)], OrderKind::Move(Target::Point(centre(22, 5))), false);
    t.order_at(30, P1, &[ObjectId(2)], OrderKind::Stop, false);
    t.run_until(33);
    assert!(t.mover(1).momentum.x > 7.0 && t.mover(2).momentum.x > 3.0);
    assert_eq!(t.mover(2).goal, MoveGoal::None);
    let json = serde_json::to_string(t.sim()).unwrap();
    assert!(json.contains("momentum"));
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    // The hash covers it: the same state without its momentum is another state.
    let without = json.replace("\"momentum\"", "\"momentum_\"");
    if let Ok(stripped) = serde_json::from_str::<Sim>(&without) {
        assert_ne!(stripped.state_hash(), t.sim().state_hash());
    }
    for _ in 0..200 {
        t.step();
        loaded.step(&TickInputs::default());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(t.status(1), MoveStatus::Arrived);
}

/// `sim` as saved and loaded again, with `edit` applied to unit `n`'s saved object.
fn edited(sim: &Sim, n: u32, edit: impl FnOnce(&mut serde_json::Value)) -> Sim {
    let mut json = serde_json::to_value(sim).expect("a state serializes");
    let objects = json["objects"].as_array_mut().expect("the objects are a list");
    edit(objects.iter_mut().find(|o| o["id"] == n).expect("the unit is in the state"));
    serde_json::from_value(json).expect("the edited state loads")
}

/// The invariant the checker, which last saw `before`, finds broken in `after`, if any of I1 to I7.
fn broken(before: &Sim, after: &Sim) -> Option<&'static str> {
    let result = Invariants::new(before).check(after, &TickInputs::default());
    result.err().map(|violation| violation.invariant).filter(|i| i.starts_with('I'))
}

/// **Negative tests** of the amended I5 and I7 (`docs/movement.md > 8`): the exception is for a DragMovement unit
/// braking off the momentum it had, and for nothing else. The movement step never does otherwise, so these states
/// are made by editing a saved one, as `invariants_overlap.rs` does.
#[test]
fn only_a_braking_drag_unit_moves_without_a_goal() {
    // Drone 1 and Peacekeeper 2, both idle and at rest, and Drone 3 stopped in full flight: braking.
    let mut t = flat(24, 8)
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(2, 1)))
        .spawn(ObjectKind::Peacekeeper, Some(P1), SpawnAt::Point(centre(2, 3)))
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(2, 5)))
        .start();
    t.without_auto_targeting();
    t.order_at(1, P1, &[ObjectId(3)], OrderKind::Move(Target::Point(centre(22, 5))), false);
    t.order_at(40, P1, &[ObjectId(3)], OrderKind::Stop, false);
    t.run_until(42);
    let before = t.sim().clone();
    t.step();
    let after = t.sim();
    assert_eq!(broken(&before, after), None, "the game itself keeps them");
    let moved = |o: &mut serde_json::Value, by: f32| {
        let x = o["pos"][0].as_f64().unwrap() + f64::from(by);
        o["pos"][0] = x.into();
    };
    // An idle Drone at rest that moves: it had no momentum to brake off.
    let drifting = edited(after, 1, |o| moved(o, 2.0));
    assert_eq!(broken(&before, &drifting), Some("I5/I7"));
    // A unit of another model with momentum: it has none, ever.
    let pushed = edited(after, 2, |o| {
        o["mover"]["momentum"] = serde_json::json!([1.0, 0.0]);
    });
    assert_eq!(broken(&before, &pushed), Some("I5/I7"));
    // A Drone without a goal that speeds up: braking means slower every tick.
    let speed = before.object(ObjectId(3)).unwrap().mover.as_ref().unwrap().momentum.x;
    let faster = edited(after, 3, |o| {
        let v = serde_json::json!([speed + 0.5, 0.0]);
        o["mover"]["momentum"] = v.clone();
        o["mover"]["velocity"] = v;
    });
    assert_eq!(broken(&before, &faster), Some("I7"));
    // Or one that keeps its speed, or moves while it claims to stand.
    let coasting = edited(after, 3, |o| {
        let v = serde_json::json!([speed, 0.0]);
        o["mover"]["momentum"] = v.clone();
        o["mover"]["velocity"] = v;
    });
    assert_eq!(broken(&before, &coasting), Some("I7"));
    let standing = edited(after, 3, |o| {
        o["mover"]["locomotion"] = serde_json::json!("Moving");
    });
    assert_eq!(broken(&before, &standing), Some("I7"));
    // I3 holds a Drone to its model's maximum.
    let jumped = edited(after, 3, |o| moved(o, 6.0));
    assert_eq!(broken(&before, &jumped), Some("I3"));
}
