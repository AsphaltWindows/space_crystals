//! HP and Shields while warping in (`factions/colonists/economy.md > Warp In > HP/Shields Interpolation`):
//! the maxima interpolated by the formulas, and damage taken before or during the Warp In carried through it
//! (design change 0110).

use super::*;

/// The plan's acceptance, "Interpolation" (`> Warp In > HP/Shields Interpolation`): MaxHP and MaxShields at 0, 25,
/// 50 and 100% of the WarpInTime follow "Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress", and the HP of
/// an undamaged building stays at its maximum: a Robo Portal at 50% has 360 HP and 15 Shields. 0% is the Beacon
/// itself; the command's tick is the first frame.
#[test]
fn hp_and_shields_interpolate_by_the_formulas() {
    for Building { kind, frames, .. } in BUILDINGS {
        let (to_hp, to_shields) = (kind.def().max_hp.unwrap(), kind.def().max_shields.unwrap());
        let mut t = base().start();
        assert_eq!(
            (object(&t, 1).max_hp(), object(&t, 1).max_shields()),
            (Some(120.0), Some(10.0))
        );
        t.debug_at(1, fund(P1, 1000)).at(1, P1, construction(&[1], warp(kind)));
        t.step();
        let first = 120.0 + (to_hp - 120.0) / frames as f32;
        assert!(
            (object(&t, 1).hp.unwrap() - first).abs() < 1e-3,
            "{kind:?}: one frame done"
        );
        for quarter in [1, 2, 4] {
            t.run_until(frames / 4 * quarter);
            let p = quarter as f32 / 4.0;
            let (hp, shields) = (120.0 + (to_hp - 120.0) * p, 10.0 + (to_shields - 10.0) * p);
            let o = object(&t, 1);
            assert_eq!(
                (o.hp, o.max_hp(), o.max_shields()),
                (Some(hp), Some(hp), Some(shields)),
                "{kind:?} at {}%",
                quarter * 25
            );
        }
        assert!(!object(&t, 1).is_transforming());
    }
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], warp(RoboPortal)));
    t.run_until(320);
    let o = object(&t, 1);
    assert_eq!((o.hp, o.max_shields()), (Some(360.0), Some(15.0)), "the plan's example");
}

/// The plan's acceptance, "With damage taken in between, the current values follow the default" (design change
/// 0110: "current += (newMax − oldMax) each frame; never a fraction-of-max rescale"): the maximum still follows the
/// formula, and the HP rises by what it rises, so the damage taken stays taken, exactly, to the end: player 2's
/// Peacekeeper 6 shoots the warping Robo Portal from tick 100 to 130, and the HP its hits take, what gets through
/// the portal's Shields less its PointArmor 1 (slice M3-B, `systems/combat.md > Shields > Shield Damage
/// Resolution`), is what it stays short of its maximum. Every value is a whole number of 64ths (HP 0.75 and Shields
/// 1/64 a frame, regeneration 0.25), so the f32 arithmetic is exact.
#[test]
fn damage_taken_while_warping_in_stays_taken() {
    let mut t = shot_at();
    t.run_until(135);
    let o = object(&t, 1);
    let deficit = o.max_hp().unwrap() - o.hp.unwrap();
    assert!(deficit > 0.0, "the Peacekeeper broke through the Shields: {deficit}");
    assert_eq!(deficit, hp_lost(&t, 1), "what its hits took");
    for tick in [200, 320, 639] {
        t.run_until(tick);
        let o = object(&t, 1);
        let max = 120.0 + 480.0 * tick as f32 / 640.0;
        assert_eq!((o.max_hp(), o.hp), (Some(max), Some(max - deficit)), "tick {tick}");
    }
    t.step();
    assert!(!object(&t, 1).is_transforming());
    assert_eq!(object(&t, 1).hp, Some(600.0 - deficit));
}

/// Design change 0110, for damage "taken before the transformation started too" (its Verification: "Same
/// arithmetic for a Warp In (Beacon 120 → target MaxHP)"): player 2's Peacekeeper 6 shoots Beacon 1 until it is 40 or
/// more short of its 120 HP, what its hits get through the Beacon's Shields (slice M3-B); the Robo Portal it then
/// warps in is exactly as short of each maximum on the way, and completes that short of its 600.
#[test]
fn a_damaged_beacon_keeps_its_deficit_through_the_warp_in() {
    let mut t = base().start();
    t.without_auto_targeting()
        .debug_at(1, fund(P1, 1000))
        .spawn_unit_at(1, Peacekeeper, P2, at(12, 7).center())
        .order_at(2, P2, &[id(6)], OrderKind::Attack(id(1)), false);
    while object(&t, 1).hp > Some(80.0) {
        assert!(t.sim().tick() < 300, "the Peacekeeper shoots the Beacon");
        t.step();
    }
    let tick = t.sim().tick();
    t.order_at(tick + 1, P2, &[id(6)], OrderKind::Stop, false).step();
    let deficit = 120.0 - object(&t, 1).hp.unwrap();
    assert!(deficit >= 40.0, "{deficit}");
    assert_eq!(deficit, hp_lost(&t, 1), "what its hits took");
    let start = tick + 2;
    t.at(start, P1, construction(&[1], warp(RoboPortal)));
    for frames in [1, 160, 320, 639, 640] {
        t.run_until(start + frames - 1);
        let o = object(&t, 1);
        let max = 120.0 + 480.0 * frames as f32 / 640.0;
        assert_eq!((o.max_hp(), o.hp), (Some(max), Some(max - deficit)), "{frames} frames");
    }
    assert!(!object(&t, 1).is_transforming());
    assert_eq!(object(&t, 1).hp, Some(600.0 - deficit));
}

/// The HP the hits on object `n` took so far (`SimEvent::Hit`).
fn hp_lost(t: &TestSim, n: u32) -> f32 {
    t.events()
        .iter()
        .filter_map(|(_, e)| match e {
            SimEvent::Hit { target, damage, .. } if *target == id(n) => Some(*damage),
            _ => None,
        })
        .sum()
}
