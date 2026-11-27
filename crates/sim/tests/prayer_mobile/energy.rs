//! Its energy (`factions/cults/units/prayer_mobile.md > Placeholder stats`: "Energy: MaxEnergy 200; starts at 50;
//! regenerates 0.5 per second"; "**Inspire** [...] 75 energy"; "**Suppress** [...] 100 energy").

use super::*;

/// The plan's acceptance (slice M5-N11): "it starts at 50, neither affordable at first, Inspire affordable after
/// 50 s". A Prayer-mobile in the game from tick 0 has 50, and the gate refuses both R and F
/// (`Reject::NotEnoughEnergy`); it gains 1/32 of a point a tick, so it has Inspire's 75 at the end of tick 800, 50 s
/// in, when the gate takes R and still refuses F, and Suppress's 100 at the end of tick 1600, 100 s in. It stops at
/// its MaxEnergy of 200, 300 s in.
#[test]
fn it_starts_at_50_inspires_after_50_s_and_suppresses_after_100_s() {
    let mut t = versus(Faction::Cults, 16, 10)
        .spawn(PRAYER_MOBILE, Some(P1), sc_sim::SpawnAt::Point(Vec2::new(400.0, 320.0)))
        .start();
    let pm = id(1);
    assert_eq!(energy(&t, pm), 50 * ONE, "it starts at 50");
    for ability in [INSPIRE, SUPPRESS] {
        assert_eq!(
            validate(&t, cast(&t, pm, ability)),
            Err(Reject::NotEnoughEnergy),
            "{ability:?}"
        );
    }
    t.run_until(799);
    assert_eq!(energy(&t, pm), 50 * ONE + 799 * A_TICK);
    assert_eq!(
        validate(&t, cast(&t, pm, INSPIRE)),
        Err(Reject::NotEnoughEnergy),
        "a tick short"
    );
    t.run_until(800);
    assert_eq!(energy(&t, pm), 75 * ONE, "50 s in");
    assert_eq!(validate(&t, cast(&t, pm, INSPIRE)), Ok(()));
    assert_eq!(validate(&t, cast(&t, pm, SUPPRESS)), Err(Reject::NotEnoughEnergy));
    t.run_until(1599);
    assert_eq!(
        validate(&t, cast(&t, pm, SUPPRESS)),
        Err(Reject::NotEnoughEnergy),
        "a tick short"
    );
    t.run_until(1600);
    assert_eq!(energy(&t, pm), 100 * ONE, "100 s in");
    assert_eq!(validate(&t, cast(&t, pm, SUPPRESS)), Ok(()));
    t.run_until(4800);
    assert_eq!(energy(&t, pm), 200 * ONE, "full 300 s in");
    t.run(16);
    assert_eq!(energy(&t, pm), 200 * ONE, "and no more");
}
