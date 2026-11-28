//! The trail's discs as plain data: one a sludge patch, as wide as it, at its centre; over the viewer's Visible tiles
//! only; shrinking over its last ticks.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Ability, Command, Faction, ObjectKind, OrderKind, PlayerId, SpawnAt, Target};

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const LOOK: crate::tuning::look::LookTuning = crate::tuning::look::BUILTIN;

/// Player 1's Shadebringer (1) laying its trail east from (300, 300) on a flat 64 × 20 map from tick 2 on, its
/// Sludge on and a move to (900, 300); player 2's Guard (2) at the map's far east end, (3900, 300), which sees none of
/// it. Tick 40 has run.
fn laying() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(64); 20].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(
            ObjectKind::Shadebringer,
            Some(P1),
            SpawnAt::Point(Vec2::new(300.0, 300.0)),
        )
        .spawn(ObjectKind::Guard, Some(P2), SpawnAt::Point(Vec2::new(3900.0, 300.0)))
        .start();
    let units = vec![sc_sim::ObjectId(1)];
    let on = Command::Ability {
        units: units.clone(),
        ability: Ability::Sludge { on: true },
    };
    t.at(2, P1, on);
    let to = OrderKind::Move(Target::Point(sc_sim::Vec2::new(900.0, 300.0)));
    t.order_at(2, P1, &units, to, false);
    t.run_until(40);
    t
}

/// A disc for each sludge patch, in the patches' order, centred on its centre (on flat ground, its view point turned
/// into Bevy's world space) and as wide as it, the newest whole: for the Shadebringer's owner, who sees its trail,
/// and for everyone while the fog doesn't apply; none for player 2, whose Guard sees none of it.
#[test]
fn each_sludge_patch_is_a_disc_over_the_viewer_s_visible_tiles() {
    let t = laying();
    let sim = t.sim();
    let patches: Vec<_> = sim.patches().iter().copied().collect();
    assert!(patches.len() > 5, "a trail: {patches:?}");
    let own = sludge_discs(sim, &Sight::new(sim, P1, true), 1.0, LOOK.sludge_fade_ticks);
    let expected: Vec<Disc> = (patches.iter())
        .map(|patch| Disc {
            center: Vec2::new(patch.centre.x, -patch.centre.y),
            radius: patch.radius,
        })
        .collect();
    assert_eq!(own, expected);
    assert_eq!(own[0].radius, 24.0, "the trail's 48 su");
    let theirs = sludge_discs(sim, &Sight::new(sim, P2, true), 1.0, LOOK.sludge_fade_ticks);
    assert_eq!(theirs, [], "out of their sight");
    let unfogged = sludge_discs(sim, &Sight::new(sim, P2, false), 1.0, LOOK.sludge_fade_ticks);
    assert_eq!(unfogged, expected, "the fog off");
}

/// A patch shows whole until its last `sludge_fade_ticks` (32), then shrinks, by the sim's ticks plus the render
/// alpha, to 0.4 of its radius at its last tick's end, the one before its `until`; with no fade, whole to its end.
#[test]
fn a_patch_shrinks_over_its_last_ticks() {
    let fade = LOOK.sludge_fade_ticks;
    assert_eq!(fade, 32);
    let until = 200;
    let share = |tick, alpha| shown_share(tick, alpha, until, fade);
    let near = |share: f32, expected: f32| (share - expected).abs() < 1e-6;
    assert_eq!(share(150, 1.0), 1.0);
    assert_eq!(share(until - 1 - 32, 1.0), 1.0, "32 ticks left");
    assert!(near(share(until - 1 - 16, 1.0), 0.7), "halfway");
    assert!(
        near(share(until - 1 - 16, 0.5), 0.4 + 0.6 * 16.5 / 32.0),
        "half a tick more left"
    );
    assert!(near(share(until - 1, 1.0), SHRUNK), "its last tick's end");
    assert_eq!(shown_share(until - 1, 1.0, until, 0), 1.0, "no fade");
}
