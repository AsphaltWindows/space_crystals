//! Strafing (`super`): `systems/controls/command_behaviors.md > AttackingObject`, GliderMovement exception: "Gliders
//! cannot stop. Instead of stopping when in range, the Glider performs strafing runs: 1. Fly toward TargetObject 2.
//! When in range and turret arc: fire while passing through 3. Continue past the target 4. Loop around for another
//! pass 5. Repeat until target is destroyed or a new command is issued"; `factions/cults/units/sparrow.md > Flight
//! rules`: "Strafing: after a pass it flies on until the target is out of range, then loops back at full speed".

use sc_sim::{Shape, math};

use super::*;

/// The Sparrow's turret: "TurnAngle 45° (total)", half of it each side of its facing (`systems/units.md >
/// TurretAttributes`).
const HALF_CONE: f32 = 22.5;

/// The gap, edge to edge, between objects `a` and `b`, and the angle of `b`'s centre off `a`'s facing, degrees.
fn geometry(t: &TestSim, a: u32, b: u32) -> (f32, f32) {
    let (a, b) = (
        t.sim().object(ObjectId(a)).unwrap(),
        t.sim().object(ObjectId(b)).unwrap(),
    );
    let off = math::wrap_angle(math::vector_angle(b.pos - a.pos) - a.facing);
    (Shape::of(a).gap(&Shape::of(b)), off.abs().to_degrees())
}

/// Player 1's Sparrow (1), 880 su west of player 2's Hand of Heaven (2), which stands still (auto-targeting off) and
/// shrugs off the machine gun (armour 2), and which player 1's Militant (3) keeps in sight from 150 su south of it:
/// the Attack goes on for 80 s. The Sparrow makes pass after pass over it, each within a few su of its centre, and
/// fires only while the target is within its turret's 45° cone and its Range of 3 grid units; between passes it flies
/// on until out of range, and it flies at MaxSpeed throughout, once it has reached it. Its Attack never ends. A shot
/// is fired in the tick before it hits: its Firing begins there, the arc and the Range checked as its Aiming ends, and
/// lasts a frame (`systems/combat.md > Firing`; the Sparrow's "Aim 1, Firing 1" frames), so it is there that the target
/// is within the cone, even when the Sparrow passes over it before the hit lands.
///
/// The spotter keeps the target in player 1's sight: a loop back at full speed, 200 su to the side, takes the Sparrow
/// some 500 su from its target, past its own SightRange of 6 grid units, and an Attack whose target its owner no longer
/// sees ends (`systems/controls/command_behaviors.md > AttackingObject` 6, design change 0200).
#[test]
fn an_attack_makes_pass_after_pass_over_a_stationary_target() {
    let mut t = quiet(
        flat(40, 40)
            .spawn(SPARROW, Some(P1), point(400.0, 1280.0))
            .spawn(ObjectKind::HandOfHeaven, Some(P2), point(1280.0, 1280.0))
            .spawn(ObjectKind::Militant, Some(P1), point(1280.0, 1430.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1], OrderKind::Attack(ObjectId(2)));
    let mut seen = 0;
    // Each pass, its shots no more than 2 s apart: its first and last shot, its closest approach (until 2 s after its
    // last shot), and the widest gap since the pass before began.
    let mut passes: Vec<(Tick, Tick, f32, f32)> = Vec::new();
    let mut widest = 0.0_f32;
    let mut before = geometry(&t, 1, 2);
    for _ in 0..16 * 80 {
        t.step();
        let tick = t.sim().tick();
        let (gap, off) = before;
        before = geometry(&t, 1, 2);
        let distance = t.pos(1).distance(t.pos(2));
        widest = widest.max(before.0);
        if tick > 13 {
            assert!(near(speed(&t, 1), MAX_SPEED, 1e-3), "tick {tick}: {}", speed(&t, 1));
        }
        let shots = hits(&t);
        for &(at, attacker, target, _) in &shots[seen..] {
            assert_eq!((at, attacker, target), (tick, 1, 2));
            assert!(gap <= RANGE + 1e-3, "tick {}: fired from {gap} su", tick - 1);
            assert!(
                off <= HALF_CONE + 1e-3,
                "tick {}: fired {off}° off its facing",
                tick - 1
            );
            match passes.last_mut() {
                Some(pass) if tick - pass.1 <= 32 => pass.1 = tick,
                _ => {
                    passes.push((tick, tick, distance, widest));
                    widest = 0.0;
                }
            }
        }
        seen = shots.len();
        if let Some(pass) = passes.last_mut()
            && tick - pass.1 <= 32
        {
            pass.2 = pass.2.min(distance);
        }
    }
    assert!(passes.len() >= 4, "{passes:?}");
    for (n, &(_, _, closest, widest)) in passes.iter().enumerate() {
        assert!(closest < 30.0, "pass {n} passed {closest} su off: {passes:?}");
        if n > 0 {
            assert!(
                widest > RANGE,
                "pass {n} came after a widest gap of {widest}: {passes:?}"
            );
        }
    }
    assert_eq!(current(&t, 1), Some(OrderKind::Attack(ObjectId(2))));
}
