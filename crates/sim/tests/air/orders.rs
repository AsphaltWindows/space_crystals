//! Stop and Hold (`super`): `factions/cults/units/sparrow.md > Flight rules`: "**Stop**: it circles where it is.
//! **Hold Position**: it circles there and fires only at what comes into range, without chasing" (design change 0354,
//! point 7).

use sc_sim::DebugCommand;

use super::*;

/// Stopped at full speed on its way east, a Sparrow circles the point where it was stopped: it slows by its
/// Deceleration to IdleSpeed, and within 15 s it is on its idle circle round that point, for good. The Stop is done at
/// once (`systems/controls/command_behaviors.md > StoppingBehavior`), and it is idle.
#[test]
fn stopped_it_circles_where_it_is() {
    let mut t = quiet(flat(64, 20).spawn(SPARROW, Some(P1), point(300.0, 640.0)).start());
    order(&mut t, 1, P1, &[1], move_to(3800.0, 640.0));
    t.run(40);
    let at = t.pos(1);
    order(&mut t, 41, P1, &[1], OrderKind::Stop);
    t.step();
    assert_eq!((current(&t, 1), t.mover(1).orbit), (None, Some(at)));
    t.run(16 * 15);
    for _ in 0..16 * 10 {
        t.step();
        assert!(
            near(t.pos(1).distance(at), IDLE_RADIUS, 1.0),
            "{}",
            t.pos(1).distance(at)
        );
        assert!(near(speed(&t, 1), IDLE_SPEED, 1e-3));
    }
}

/// Put on Hold at full speed, a Sparrow circles the point where it was. Once it is on its idle circle, player 2's
/// Drones come into the game and auto-targeting is turned on: it never leaves its circle for them, though it sees them.
/// The one 320 su north, out of its Range wherever it is on the circle, it never shoots nor chases; the one 100 su
/// south, which the circle brings into range and its turret's cone, it shoots (`systems/controls/command_behaviors.md >
/// HoldingPosition`: "turret continues autonomous scanning and engaging targets within range and arc").
#[test]
fn on_hold_it_circles_there_and_fires_only_at_what_comes_into_range() {
    let mut t = quiet(
        flat(64, 30)
            .faction(P2, Faction::Colonists)
            .spawn(SPARROW, Some(P1), point(300.0, 960.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1], move_to(3800.0, 960.0));
    t.run(60);
    let at = t.pos(1);
    order(&mut t, 61, P1, &[1], OrderKind::Hold);
    t.run(16 * 15);
    assert_eq!(t.mover(1).orbit, Some(at));
    assert!(
        near(t.pos(1).distance(at), IDLE_RADIUS, 1.0),
        "{}",
        t.pos(1).distance(at)
    );
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::SetAutoTargeting { on: true })
        .spawn_unit_at(tick, ObjectKind::Drone, P2, at + Vec2::new(0.0, -320.0))
        .spawn_unit_at(tick, ObjectKind::Drone, P2, at + Vec2::new(0.0, 100.0));
    for _ in 0..16 * 30 {
        t.step();
        assert!(
            near(t.pos(1).distance(at), IDLE_RADIUS, 1.0),
            "{}",
            t.pos(1).distance(at)
        );
    }
    assert_eq!(current(&t, 1), Some(OrderKind::Hold));
    let shots: Vec<u32> = (hits(&t).into_iter())
        .filter(|&(_, attacker, ..)| attacker == 1)
        .map(|(_, _, target, _)| target)
        .collect();
    assert!(shots.contains(&3), "it shoots the Drone in range: {shots:?}");
    assert!(!shots.contains(&2), "never the one out of range: {shots:?}");
}
