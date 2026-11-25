//! The momentum as state (`Mover::momentum`): a save in full flight, the amended invariants I5 and I7 from
//! their wrong side (negative tests), and a debug teleport, which takes the momentum.

use super::*;

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
    t.order_at(
        1,
        P1,
        &[ObjectId(2)],
        OrderKind::Move(Target::Point(centre(22, 5))),
        false,
    );
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
    edit(
        objects
            .iter_mut()
            .find(|o| o["id"] == n)
            .expect("the unit is in the state"),
    );
    serde_json::from_value(json).expect("the edited state loads")
}

/// The invariant the checker, which last saw `before`, finds broken in `after`, if any of I1 to I7.
fn broken(before: &Sim, after: &Sim) -> Option<&'static str> {
    let result = Invariants::new(before).check(after, &TickInputs::default());
    result
        .err()
        .map(|violation| violation.invariant)
        .filter(|i| i.starts_with('I'))
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
    t.order_at(
        1,
        P1,
        &[ObjectId(3)],
        OrderKind::Move(Target::Point(centre(22, 5))),
        false,
    );
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

/// A debug teleport takes the unit's momentum: it doesn't arrive with the speed it had, and starts again from rest
/// towards its goal.
#[test]
fn a_teleported_drone_starts_again_from_rest() {
    let mut t = flat(24, 8).spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2))).start();
    move_to(&mut t, 1, centre(22, 2));
    t.run_until(40);
    assert!(t.mover(1).momentum.x > 7.0);
    let to = centre(3, 5);
    t.debug_at(
        41,
        sc_sim::DebugCommand::SetPosition {
            id: ObjectId(1),
            pos: to,
        },
    );
    t.step();
    let step = t.pos(1) - to;
    assert!(step.length() <= 0.8 + 1e-3, "from rest: {step}");
    assert_eq!(t.mover(1).momentum, step);
    for _ in 0..300 {
        t.step();
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, centre(22, 2)));
}
