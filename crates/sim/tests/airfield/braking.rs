//! Braking by drag alone (`systems/units.md > DragMovement`: "Air units [...] relying on drag alone to stop";
//! `systems/units.md > HoverCraft`: "Relies on high drag to decelerate passively"): an air HoverCraft such as the
//! Hornet thrusts nothing against its motion, and drifts to rest as its DragRatio slows it, where a ground hover unit
//! still brakes with its thrust (`sc_sim::defs::UnitBaseDef::brakes_by_drag`). The map's edge still stops it
//! (`Map::clamp_box`, design change 0045).

use super::*;

/// Unit `n` sent east toward a point far off, then stopped once it has flown `ticks` ticks: its speed as it is
/// stopped, and how far it then drifts until it is at rest, and in how many ticks; each tick's speed is checked to be
/// what the drag keeps of the last at least (`keep_at_least`).
fn stop_after(t: &mut TestSim, n: u32, ticks: u32, keep_at_least: f32) -> (f32, f32, u32) {
    let from = t.pos(n);
    next(
        t,
        P1,
        order(&[n], OrderKind::Move(Target::Point(from + Vec2::new(2400.0, 0.0)))),
    );
    t.run(ticks);
    let (stopped_at, speed) = (t.pos(n), t.mover(n).velocity.length());
    assert!(t.mover(n).velocity.y.abs() < 1e-3, "east: {}", t.mover(n).velocity);
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, order(&[n], OrderKind::Stop));
    let mut last = speed;
    let mut drift_ticks = 0;
    while drift_ticks == 0 || t.mover(n).velocity != Vec2::ZERO {
        assert!(drift_ticks < 400, "#{n} comes to rest");
        t.step();
        let now = t.mover(n).velocity.length();
        if now > 0.0 {
            assert!(now >= last * keep_at_least - 1e-3, "#{n}: {last} to {now}");
        }
        last = now;
        drift_ticks += 1;
    }
    (speed, t.pos(n).x - stopped_at.x, drift_ticks)
}

/// A Hornet near its top speed of 9 su a frame, ordered to Stop, drifts to rest by drag alone (`v ← 0.9 v` a frame),
/// about the `v × keep / (1 − keep)` its DragRatio gives: 80 su in 43 frames, of 81; braking with its thrust it would
/// slow to 8 su in the first. A Colonists Drone, a ground hover unit (HoverVehicle), still brakes with its thrust and
/// stops well short of what its drag alone would give.
#[test]
fn a_hornet_drifts_to_rest_by_drag_alone_and_a_ground_hover_unit_brakes() {
    let mut t = field(|builder| {
        builder
            .spawn(HORNET, Some(P1), at(400.0, 1600.0)) // 7
            .spawn(ObjectKind::Drone, Some(P1), at(400.0, 2200.0)) // 8
    });
    let (speed, drift, ticks) = stop_after(&mut t, 7, 60, 0.9);
    assert!(speed > 8.9, "near its top speed: {speed}");
    let by_drag = speed * 0.9 / (1.0 - 0.9);
    assert!(
        drift > by_drag - 2.0 && drift <= by_drag + 1e-3,
        "{drift} su from {speed} ({by_drag})"
    );
    assert!((40..=44).contains(&ticks), "{ticks} ticks");

    let (speed, drift, _) = stop_after(&mut t, 8, 60, 0.0);
    let keep = 1.0 - drone_drag_ratio();
    let by_drag = speed * keep / (1.0 - keep);
    assert!(speed > 2.0, "{speed}");
    assert!(
        drift < 0.8 * by_drag,
        "braking with its thrust: {drift} su from {speed} ({by_drag})"
    );
}

/// The Drone's DragRatio.
fn drone_drag_ratio() -> f32 {
    match ObjectKind::Drone.def().unit.as_ref().expect("a unit").movement {
        sc_sim::defs::MovementAttrs::Drag { drag_ratio, .. } => drag_ratio,
        other => panic!("{other:?}"),
    }
}

/// A Hornet stopped short of the map's east edge drifts on to it and no further: its box stays on the map.
#[test]
fn the_map_edge_stops_a_drifting_hornet() {
    let edge = 60.0 * 64.0;
    let mut t = field(|builder| builder.spawn(HORNET, Some(P1), at(edge - 900.0, 1600.0))); // 7
    let goal = Vec2::new(edge + 2000.0, 1600.0);
    let clamp = |sim: &Sim| sim.object(id(7)).map(|o| o.pos.x);
    next(&mut t, P1, order(&[7], OrderKind::Move(Target::Point(goal))));
    step_until(&mut t, 400, "near the edge", |sim| {
        clamp(sim).is_some_and(|x| x > edge - 90.0)
    });
    next(&mut t, P1, order(&[7], OrderKind::Stop));
    t.run(100);
    let x = t.pos(7).x;
    assert!(x <= edge - 24.0 + 1e-3, "its box on the map: {x}");
    assert!(x > edge - 40.0, "it drifted on to the edge: {x}");
    assert_eq!(t.mover(7).velocity, Vec2::ZERO);
}
