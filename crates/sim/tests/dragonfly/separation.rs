//! The Dragonfly's soft separation: `factions/gdo/units/dragonfly.md > Placeholder stats`: "SeparationRadius **48** su
//! (raised from 32, Q0148; must exceed the 40 su Silhouette)" (design change 0405); and `systems/units.md > Air
//! Collision`: "**Gliders push apart slowly** [...] **a third** of the HoverCraft strength" (design change 0412). It is
//! every Glider's (`UnitBaseDef::separation_strength`, `movement::air`), as the Sparrow's (`tests/air`'s `separation`).

use super::*;

/// The Dragonfly's SeparationRadius, su.
const SEPARATION: f32 = 48.0;
/// Every Glider's idle circle: "1 grid unit" (`tuning::air`).
const IDLE_RADIUS: f32 = 64.0;

/// The plan's acceptance, "two Dragonflies end 48 su apart, pushed at the gliders' third of the HoverCraft strength".
/// Two Dragonflies side by side 30 su apart, 400 su west of a point, each sent to it on its own (the same point for
/// both), fly there together and circle it, 21 su apart at closest. Their weak push parts them gradually as they
/// circle: as the game played when this test was made, the pair was last under 90 % of the SeparationRadius at 8.1 s,
/// under 99 % at 10.3 s and more than 0.01 su short of it at 15.8 s. They end on the point's idle circle, idle, 48 su
/// apart, where the push stops, and stay so.
#[test]
fn two_dragonflies_sent_to_one_point_end_48_su_apart() {
    let goal = Vec2::new(2400.0, 1600.0);
    let mut t = field(|builder| {
        builder
            .spawn(DRAGONFLY, Some(P1), at(2000.0, 1600.0))
            .spawn(DRAGONFLY, Some(P1), at(2000.0, 1630.0))
    });
    let pair = [7, 8];
    for n in pair {
        next(&mut t, P1, move_to(&[n], goal));
    }
    let apart = |t: &TestSim| t.pos(pair[0]).distance(t.pos(pair[1]));
    let mut closest = f32::INFINITY;
    // The last tick the pair was under 90 % of the radius, under 99 %, and more than 0.01 su short of it.
    let mut under = [0; 3];
    for _ in 0..16 * 60 {
        t.step();
        for n in pair {
            flies_or_is_held(frame(t.sim(), n));
        }
        let now = apart(&t);
        closest = closest.min(now);
        let least = [SEPARATION * 0.9, SEPARATION * 0.99, SEPARATION - 0.01];
        for (last, least) in under.iter_mut().zip(least) {
            if now < least {
                *last = t.sim().tick();
            }
        }
    }
    assert!(closest < SEPARATION / 2.0, "they fly in close: {closest}");
    let [ninety, ninety_nine, about] = under;
    assert!(
        (16 * 4..16 * 12).contains(&ninety),
        "they part gradually, under 90 % until tick {ninety}"
    );
    assert!(ninety_nine <= 16 * 16, "under 99 % until tick {ninety_nine}");
    assert!(about <= 16 * 24, "more than 0.01 su short until tick {about}");
    let end = apart(&t);
    assert!(
        (SEPARATION - 0.01..SEPARATION + 0.05).contains(&end),
        "they end {end} su apart"
    );
    for n in pair {
        let off = t.pos(n).distance(goal);
        assert!((off - IDLE_RADIUS).abs() < 0.05, "#{n}: {off} su off the point");
        assert_eq!((doing(t.sim(), n), mover(t.sim(), n).orbit), (None, Some(goal)), "#{n}");
    }
    t.run(16 * 5);
    let after = apart(&t);
    assert!((end - after).abs() < 0.01, "and stay so: {end}, {after}");
}
