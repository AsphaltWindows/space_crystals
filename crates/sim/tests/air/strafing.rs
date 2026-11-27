//! Strafing (`super`): `systems/controls/command_behaviors.md > AttackingObject`, GliderMovement exception: "Gliders
//! cannot stop. Instead of stopping when in range, the Glider performs strafing runs: 1. Fly toward TargetObject 2.
//! When in range and turret arc: fire while passing through 3. Continue past the target 4. Loop around for another
//! pass 5. Repeat until target is destroyed or a new command is issued"; `factions/cults/units/sparrow.md > Flight
//! rules`: "Strafing: after a pass it flies on until the target's **centre** is out of range [...], then loops back at
//! full speed" (design change 0408).

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
/// The spotter keeps the target in player 1's sight, so that the passes alone are tested: from the start, 880 su off,
/// and through the loops, as an Attack whose target its owner no longer sees ends
/// (`systems/controls/command_behaviors.md > AttackingObject` 6, design change 0200). A loop back at full speed, 200 su
/// to the side, takes the Sparrow some 500 su from its target, and more from one this long: about its SightRange of 8
/// grid units (512 su, design change 0403; 6 when this test was written). The next test has no spotter.
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

/// The Sparrow's SightRange, 8 grid units (design change 0403).
const SIGHT: f32 = 8.0 * 64.0;

/// Design change 0403's Verification: "A lone Sparrow ordered to Attack a stationary target, with no spotter, makes
/// repeated passes until the target dies" (`factions/cults/units/sparrow.md > Placeholder stats`: "SightRange **8 grid
/// units** [...] its full-speed loop carries it up to ~500 su from its target, so 8 GU = 512 su keeps the target in
/// sight through every pass and a lone Sparrow keeps strafing on an Attack"). Player 1's Sparrow (1), alone, attacks
/// player 2's Peacekeeper (2), 412 su east of it. The Peacekeeper stands still (auto-targeting off) at the centre of a
/// tile, the one tile it covers, so that player 1 sees it only while that tile's centre, its own, is within the
/// Sparrow's sight (or for the fog delay after). The Attack is the Sparrow's order until the Peacekeeper (MaxHP 50,
/// PointArmor 1: 2 HP a hit) dies, on its fifth pass; each loop between two passes takes the Sparrow 488 to 496 su from
/// its target (497 to 502 before design change 0408 ended a pass where the target's centre is out of range, not its
/// far edge; `pass_end` has the other targets), within its sight; the Attack then completes.
#[test]
fn a_lone_sparrow_makes_pass_after_pass_until_its_target_dies() {
    let target = Vec2::new(1312.0, 1312.0);
    let mut t = quiet(
        flat(40, 40)
            .faction(P2, Faction::Gdo)
            .spawn(SPARROW, Some(P1), point(900.0, 1312.0))
            .spawn(ObjectKind::Peacekeeper, Some(P2), SpawnAt::Point(target))
            .start(),
    );
    order(&mut t, 1, P1, &[1], OrderKind::Attack(ObjectId(2)));
    // Each pass, its shots no more than 2 s apart: its last shot, and the farthest the Sparrow got from the target
    // between the pass before and this one.
    let mut passes: Vec<(Tick, f32)> = Vec::new();
    let (mut seen, mut farthest) = (0, 0.0_f32);
    loop {
        t.step();
        let tick = t.sim().tick();
        farthest = farthest.max(t.pos(1).distance(target));
        let shots = hits(&t);
        for &(at, attacker, target, _) in &shots[seen..] {
            assert_eq!((at, attacker, target), (tick, 1, 2));
            match passes.last_mut() {
                Some(pass) if tick - pass.0 <= 32 => pass.0 = tick,
                _ => passes.push((tick, std::mem::take(&mut farthest))),
            }
        }
        seen = shots.len();
        if t.sim().object(ObjectId(2)).is_none() {
            break;
        }
        assert!(tick < 16 * 60, "the target lives on: {passes:?}");
        assert_eq!(
            current(&t, 1),
            Some(OrderKind::Attack(ObjectId(2))),
            "tick {tick}: {passes:?}"
        );
    }
    assert!(passes.len() >= 4, "{passes:?}");
    for &(_, farthest) in &passes[1..] {
        assert!(farthest < SIGHT, "a loop took it {farthest} su off: {passes:?}");
    }
    assert_eq!(current(&t, 1), None, "its target died");
}
