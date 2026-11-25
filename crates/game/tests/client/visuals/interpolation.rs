//! Interpolation (`ARCHITECTURE.md > Client architecture > Visuals > Interpolation`): an object drawn between
//! its tick positions, except after a teleport.

use super::*;

/// Advances real time by `fraction` of a tick in one frame. Returns the frame's interpolation factor and where
/// object 1 is drawn: on flat ground, the ground position it is drawn at.
fn advance(app: &mut App, fraction: f64) -> (f32, Vec2) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(timestep.mul_f64(fraction)));
    app.update();
    let alpha = app.world().resource::<RenderAlpha>().0;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let drawn = app.world().get::<Transform>(entity).unwrap().translation;
    (alpha, Vec2::new(drawn.x, -drawn.y))
}

/// `ARCHITECTURE.md > Client architecture > Visuals > Interpolation`: between ticks, an object is drawn between its
/// `prev_pos` and `pos`, by the part of the next tick that has passed (`Time<Fixed>::overstep_fraction`); a teleport
/// (`DebugCommand::SetPosition`) is drawn at its target at once, never on the way there.
#[test]
fn objects_are_drawn_between_their_tick_positions_except_after_a_teleport() {
    let mut scenario = scenarios::empty(16, 12);
    scenario.map.spawns.push(patch_at(2, 2));
    let mut app = headless_app(scenario);
    updates(&mut app, 2);
    // Nothing in the sim moves yet: make object 1 a free (not tile-based) object that moved in the last tick.
    let (from, to) = (Vec2::new(200.0, 300.0), Vec2::new(520.0, 140.0));
    let mut state = serde_json::to_value(app.world().resource::<SimRes>().sim()).unwrap();
    let object = &mut state["objects"][0];
    object["prev_pos"] = serde_json::to_value(from).unwrap();
    object["pos"] = serde_json::to_value(to).unwrap();
    object["placement"] = serde_json::Value::Null;
    let moved: Sim = serde_json::from_value(state).unwrap();
    app.world_mut().resource_mut::<SimRes>().replace(moved);
    let start = tick(&app);

    let close = |a: Vec2, b: Vec2| (a - b).length() < 1e-3;
    let (alpha, drawn) = advance(&mut app, 0.0);
    assert_eq!(alpha, 0.0);
    assert!(close(drawn, from), "{drawn} should be {from}");
    for (fraction, expected_alpha) in [(0.25, 0.25), (0.5, 0.75)] {
        let (alpha, drawn) = advance(&mut app, fraction);
        assert!((alpha - expected_alpha).abs() < 1e-6, "{alpha}");
        let expected = from.lerp(to, expected_alpha);
        assert!(close(drawn, expected), "{drawn} should be {expected}");
    }
    assert_eq!(tick(&app), start, "no tick ran yet");

    // The next tick teleports the object: drawn at the target in the frame of the tick, wherever the clock is.
    let target = Vec2::new(700.0, 600.0);
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::SetPosition {
            id: ObjectId(1),
            pos: target,
        }));
    let (alpha, drawn) = advance(&mut app, 0.5);
    assert_eq!(tick(&app), start + 1);
    assert!((alpha - 0.25).abs() < 1e-6, "{alpha}");
    assert_eq!(drawn, target);
    let object = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().clone();
    assert_eq!((object.prev_pos, object.pos), (target, target));
    let (_, drawn) = advance(&mut app, 0.5);
    assert_eq!(drawn, target);
}
