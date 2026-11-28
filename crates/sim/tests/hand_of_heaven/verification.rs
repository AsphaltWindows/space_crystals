//! Design change 0363's Verification: "Fire at a building 12 tiles away: it aims for 3 s, launches, and can drive off
//! at once. The missile lands about 6 s later. A unit inside 4 tiles can't be targeted." And (M6-S6a) design change
//! 0454's first: "Order a lone Hand of Heaven to attack an enemy 2 GU away. It backs off to about 5 GU, keeps the
//! target in sight, and fires."

use sc_sim::gu;

use super::*;

/// Player 1's Hand of Heaven (1) at the centre of tile (6, 12), facing east, and player 2's Chop Shop (2) on tiles
/// (17, 11) to (19, 13), whose centre is 12 tiles due east of the Hand of Heaven's: the gap between them, edge to edge,
/// is 768 − 96 − 32 = 640 su, 10 grid units, within its Range of 14. Player 1's Recruit (3) north of the Chop Shop is
/// its spotter, as the Hand of Heaven sees 7 grid units ("it needs spotters for its full range"); player 2's Storage
/// (4) in the far corner keeps it in the game.
pub(super) fn shooting_range() -> TestSim {
    let mut t = cults(40, 24)
        .spawn(HAND, Some(P1), point(centre(6, 12)))
        .spawn(ObjectKind::ChopShop, Some(P2), tile(17, 11))
        .spawn(ObjectKind::Recruit, Some(P1), point(centre(18, 8)))
        .spawn(ObjectKind::Storage, Some(P2), tile(36, 1))
        .start();
    t.without_auto_targeting();
    t
}

/// "Fire at a building 12 tiles away: it aims for 3 s, launches, and can drive off at once. The missile lands about 6 s
/// later." Facing it, the Hand of Heaven aims at the Chop Shop from the order's tick for its AimDuration, 48 ticks (3
/// s), and fires for its FiringDuration, 2: the missile leaves at the end of tick 1 + 48 + 2 − 1, from its centre to
/// the Chop Shop's, where it lands 768 / 8 = 96 ticks (6 s) later. A Move given in the next tick turns it at once, its
/// attack's Cooldown of 4 and Reload of 96 running on to their ends; it is driving well before the Reload is over,
/// and then aims at nothing.
#[test]
fn fired_at_a_building_12_tiles_away_it_aims_3_s_launches_and_drives_off_at_once() {
    let mut t = shooting_range();
    let (from, at) = (t.pos(1), t.pos(2));
    assert_eq!(at - from, Vec2::new(TWELVE_TILES, 0.0));
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    t.run_until(1);
    assert_eq!(
        phase(&t, 1),
        Some(AttackPhase::Aiming { target: id(2) }),
        "aiming at once"
    );
    let launch_tick = 1 + AIM + FIRING - 1;
    until(&mut t, 80, "the launch", |t| !launches(t).is_empty());
    let [launch] = launches(&t)[..] else {
        panic!("one missile: {:?}", launches(&t));
    };
    assert_eq!(
        launch,
        Launch {
            tick: launch_tick,
            shooter: id(1),
            target: Some(id(2)),
            from,
            to: at,
            lands_at: launch_tick + 96,
        },
        "48 ticks aimed and 2 fired; 768 su at 8 su a tick"
    );
    assert_eq!((TWELVE_TILES / SPEED) as Tick, 96);
    assert_eq!(phase(&t, 1), Some(AttackPhase::Cooldown));
    assert_eq!(t.pos(1), from, "it stood to fire");

    // Drive off at once: a Move north, a quarter turn from its heading, in the first tick of its Cooldown.
    let off = centre(6, 3);
    let was = facing(&t, 1);
    order(
        &mut t,
        launch_tick + 1,
        P1,
        &[1],
        OrderKind::Move(sc_sim::Target::Point(off)),
    );
    t.step();
    assert!(facing(&t, 1) != was, "it turns in the Move's first tick");
    assert_eq!(phase(&t, 1), Some(AttackPhase::Cooldown), "in its Cooldown");
    let mut moving = None;
    let reload_ends = launch_tick + COOLDOWN + RELOAD;
    while t.sim().tick() < reload_ends - 1 {
        t.step();
        let tick = t.sim().tick();
        if moving.is_none() && t.pos(1) != from {
            moving = Some(tick);
        }
        // Each phase lasts its duration: the Cooldown takes ticks 51 to 54, the Reload 55 to 150.
        let expected = if tick < launch_tick + COOLDOWN {
            AttackPhase::Cooldown
        } else {
            AttackPhase::Reloading
        };
        assert_eq!(phase(&t, 1), Some(expected), "tick {tick}");
    }
    let moving = moving.expect("it drove off");
    assert!(
        moving <= launch_tick + 8,
        "driving from tick {moving}, a spin of 5 first"
    );
    assert!(t.pos(1).y < from.y - 128.0, "well on its way north: {}", t.pos(1));
    t.step();
    assert_eq!(
        phase(&t, 1),
        None,
        "tick {}: the Reload over, nothing to aim at",
        t.sim().tick()
    );

    until(&mut t, 50, "the landing", |t| !blasts(t).is_empty());
    assert_eq!(blasts(&t), [(launch_tick + 96, Some(id(1)), at)]);
    let [(tick, attacker, _)] = hits_on(&t, 2)[..] else {
        panic!("one hit: {:?}", hits_on(&t, 2));
    };
    assert_eq!((tick, attacker), (launch_tick + 96, id(1)), "6 s after the launch");
}

/// Player 1's Hand of Heaven (1) at the centre of tile (10, 6), facing east, and player 2's Militant (2) `off` grid
/// units off its edge, inside its MinRange; with `spotter`, player 1's Recruit (3) 3 tiles north of the Militant, out
/// of the blast's circle, which keeps it in sight.
fn hugged(off: f32, spotter: bool) -> TestSim {
    let at = centre(10, 6);
    let militant = at + Vec2::new(32.0 + gu(off) + 12.0, 0.0);
    let mut scenario =
        cults(40, 12)
            .spawn(HAND, Some(P1), point(at))
            .spawn(ObjectKind::Militant, Some(P2), point(militant));
    if spotter {
        scenario = scenario.spawn(
            ObjectKind::Recruit,
            Some(P1),
            point(militant - Vec2::new(0.0, 3.0 * SU_PER_GU)),
        );
    }
    scenario.start()
}

/// The gap, edge to edge, between the Hand of Heaven whose centre is `hand`, facing along their row (its 64 su along
/// it), and the Militant at `target` (24 su).
fn gap(hand: Vec2, target: Vec2) -> f32 {
    hand.distance(target) - 32.0 - 12.0
}

/// "A unit inside 4 tiles can't be targeted" (`factions/cults/units/hand_of_heaven.md`: "enemies closer than MinRange
/// can't be targeted (systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange)"), with a
/// spotter: ordered to attack the Militant 3 grid units off, the Hand of Heaven backs off to MinRange + 1 grid unit (5
/// grid units; design changes 0182 and 0454) first, and every missile of its twenty seconds leaves from beyond
/// MinRange. On its own, it never picks the Militant: with auto-targeting on, the Militant on Hold
/// firing at it, it fires at nothing in ten seconds (`systems/controls/behaviors.md > BaseAutoTargeting > Targets
/// inside MinRange`: auto-targeting "skips enemies inside the weapon's MinRange").
#[test]
fn a_unit_inside_4_tiles_is_never_fired_at() {
    let mut t = hugged(3.0, true);
    t.without_auto_targeting();
    let target = t.pos(2);
    assert!(gap(t.pos(1), target) < gu(MIN_RANGE));
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    t.run_until(16 * 20);
    let fired = launches(&t);
    assert!(!fired.is_empty(), "it fires once out of MinRange");
    for launch in &fired {
        assert_eq!(launch.target, Some(id(2)));
        let off = gap(launch.from, target);
        assert!(off >= gu(MIN_RANGE), "fired from {off} su off");
    }

    let mut t = hugged(3.0, false);
    order(&mut t, 1, P2, &[2], OrderKind::Hold);
    t.run_until(16 * 10);
    assert!(!hits_on(&t, 1).is_empty(), "the Militant fires at it");
    assert_eq!(launches(&t), [], "it never fires at the Militant");
}

/// Design change 0454's Verification 1 (answering Q0179): "Order a lone Hand of Heaven to attack an enemy 2 GU away. It
/// backs off to about 5 GU, keeps the target in sight, and fires." `systems/controls/command_behaviors.md >
/// AttackingObject > Target inside MinRange`: "to **MinRange + 1 grid unit** (never beyond Range), every unit". With
/// no spotter, ordered to attack the Militant 2 grid units off, the Hand of Heaven backs off to 5 grid units, edge to
/// edge, within its own SightRange of 7, turns back to the Militant and fires at it, its Attack running all the while.
/// (Before 0454 it backed off to the middle of the band, 9 grid units, out of its own sight of the Militant, and its
/// Attack ended there unfired, as a target lost from vision does: "The band's middle took the Hand of Heaven, 9 GU,
/// beyond its own 7 GU sight, so alone it lost the target and never fired".)
#[test]
fn alone_it_backs_off_to_5_grid_units_keeps_the_target_in_sight_and_fires() {
    let mut t = hugged(2.0, false);
    t.without_auto_targeting();
    let target = t.pos(2);
    assert!((gap(t.pos(1), target) - gu(2.0)).abs() < 1e-3);
    order(&mut t, 1, P1, &[1], OrderKind::Attack(id(2)));
    while launches(&t).is_empty() {
        assert!(t.sim().tick() < 16 * 20, "it fires");
        t.step();
        let o = t.sim().object(id(1)).expect("the Hand of Heaven");
        let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
        assert!(current.is_some(), "tick {}: the Attack goes on", t.sim().tick());
    }
    let [launch] = launches(&t)[..] else {
        panic!("one missile: {:?}", launches(&t));
    };
    assert_eq!((launch.target, launch.to), (Some(id(2)), target));
    let off = gap(launch.from, target);
    assert!((off - gu(MIN_RANGE + 1.0)).abs() < 2.0, "fired from {off} su off");
    assert!(off < gu(7.0), "within its own sight");
}
