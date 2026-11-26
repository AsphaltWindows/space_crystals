//! AttackMove (`super`): `systems/controls/command_behaviors.md > AttackMovingToLocation`, GliderMovement exception:
//! "Glider follows its path, performing strafing runs on enemies within SightRange. Leash is measured the same way. On
//! arrival at TargetLocation, transitions to idle circling."

use super::*;

/// Player 1's Sparrow (1) attack-moves 1700 su east past player 2's idle Drone (2), 100 su south of its way: it takes
/// the Drone, which it sees, and makes a strafing pass over it, firing; it flies on to its destination, where its
/// AttackMove is over, and from then on it is idle and circles the destination: within 15 s at IdleSpeed on its idle
/// circle, for good.
///
/// With the Sparrow's numbers it makes one pass only: its loop back, at full speed, swings it 400 su to the side of
/// its course (twice its 200-su turn radius), past the AttackMove's leash of 6 grid units from its way, so it gives the
/// Drone up there (`> AttackMovingToLocation` 3: "Leash is measured the same way") and resumes its way.
#[test]
fn on_an_attack_move_it_strafes_what_it_meets_and_circles_its_destination() {
    let destination = Vec2::new(2000.0, 640.0);
    let mut t = flat(40, 20)
        .faction(P2, Faction::Colonists)
        .spawn(SPARROW, Some(P1), point(300.0, 640.0))
        .spawn(ObjectKind::Drone, Some(P2), point(1100.0, 740.0))
        .start();
    order(&mut t, 1, P1, &[1], OrderKind::AttackMove(destination));
    let mut strafed = 0;
    while t.sim().tick() < 2 || current(&t, 1).is_some() {
        assert!(t.sim().tick() < 16 * 30, "still on its way at {}", t.pos(1));
        let before = hits(&t).len();
        t.step();
        let shots = &hits(&t)[before..];
        assert!(
            shots
                .iter()
                .all(|&(_, attacker, target, _)| (attacker, target) == (1, 2)),
            "{shots:?}"
        );
        if !shots.is_empty() {
            assert!(t.mover(1).strafe.is_some(), "it fires on a strafing pass");
            strafed += shots.len();
        }
    }
    assert!(strafed >= 3, "{strafed} hits");
    assert!(t.sim().object(ObjectId(2)).is_some(), "the Drone lives on");
    assert_eq!(t.mover(1).orbit, Some(destination));
    t.run(16 * 15);
    for _ in 0..16 * 10 {
        t.step();
        assert!(
            near(t.pos(1).distance(destination), IDLE_RADIUS, 1.0),
            "{}",
            t.pos(1).distance(destination)
        );
        assert!(near(speed(&t, 1), IDLE_SPEED, 1e-3), "{}", speed(&t, 1));
        assert_eq!(current(&t, 1), None);
    }
}
