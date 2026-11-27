//! AttackMove (`super`): `systems/controls/command_behaviors.md > AttackMovingToLocation`, GliderMovement exception:
//! "Glider follows its path, performing strafing runs on enemies within SightRange. Leash is measured the same way. On
//! arrival at TargetLocation, transitions to idle circling."

use super::*;

/// `systems/controls/command_behaviors.md > AttackMovingToLocation > Global Constant: AttackMoveLeashDistance`: 6 grid
/// units from the PathReference, here the straight way from the Sparrow's start to its destination, along y = 640.
const LEASH: f32 = 6.0 * 64.0;

/// Player 1's Sparrow (1), alone, attack-moves 1700 su east past player 2's idle Drone (2), 100 su south of its way:
/// the Sparrow takes the Drone and makes strafing passes over it, firing; it flies on to its destination, where its
/// AttackMove is over, and from then on it is idle and circles the destination: within 15 s at IdleSpeed on its idle
/// circle, for good.
///
/// The leash lets the Drone go (design change 0403: "**Attack Move is unchanged**: its 6 GU leash may still drop a
/// looping Sparrow's target"). Flying on out of range and looping back at full speed on its 200-su turn radius, the
/// Sparrow swings more than the leash off its way after its second pass, and it lets the Drone go as it crosses the
/// leash ("Leash is measured the same way"; `systems/controls/command_behaviors.md > AttackMovingToLocation` 3) and
/// resumes its way. (M5-N7) It crosses the leash on its loop back toward the Drone, heading away from its way, so its
/// turn for its destination swings it some 700 su off its way, and the Drone is farther than its SightRange of 8 grid
/// units (0403) by the time it is back within the leash: the reach rule, which judges the Drone again like any other
/// enemy (design change 0230: "There is no memory of the targets it let go"), doesn't take it again ("without leaving
/// its route or losing sight of it"). Before design change 0408, each pass flew on until the Drone's far edge was out
/// of range, not its centre, some 18 su farther (its half width); the loop crossed the leash elsewhere, and the Sparrow
/// took the Drone once more, for two more passes. The Drone's Shields, back up after each pass, keep it alive. (With
/// the SightRange of 6 this test was written for, a spotter kept the Drone in sight, and the Sparrow took it once, for
/// one pass.)
#[test]
fn on_an_attack_move_it_strafes_what_it_meets_and_circles_its_destination() {
    let destination = Vec2::new(2000.0, 640.0);
    let mut t = flat(40, 20)
        .faction(P2, Faction::Colonists)
        .spawn(SPARROW, Some(P1), point(300.0, 640.0))
        .spawn(ObjectKind::Drone, Some(P2), point(1100.0, 740.0))
        .start();
    order(&mut t, 1, P1, &[1], OrderKind::AttackMove(destination));
    let off_way = |p: Vec2| (p.y - destination.y).abs();
    let (mut strafed, mut let_go) = (0, Vec::new());
    while t.sim().tick() < 2 || current(&t, 1).is_some() {
        assert!(t.sim().tick() < 16 * 60, "still on its way at {}", t.pos(1));
        let (before, judged_at, strafing) = (hits(&t).len(), t.pos(1), t.mover(1).strafe.is_some());
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
        if strafing && t.mover(1).strafe.is_none() {
            let_go.push(off_way(judged_at));
        }
    }
    assert!(strafed >= 12, "{strafed} hits");
    assert!(t.sim().object(ObjectId(2)).is_some(), "the Drone lives on");
    // Where it stood as its AttackMove judged the chase, before its flight in the tick: just past the leash, by less
    // than a tick's flight at MaxSpeed (10 su), each time.
    assert_eq!(let_go.len(), 1, "it takes the Drone once: {let_go:?}");
    for &off in &let_go {
        assert!(off > LEASH - 2.0 && off < LEASH + 12.0, "let go {off} su off its way");
    }
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
