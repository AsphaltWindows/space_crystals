//! The energy's integers (plan decision T75) and its regeneration where a test needs the sim's inside: a pool that
//! regenerates by the units inside it (the Shrine's rule, before the Shrine has its pool: slice M5-N5), on an Armory a
//! test def gives one.

use glam::Vec2;

use super::*;
use crate::command::{Command, PlayerCommand, TickInputs};
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectDef, ObjectKind};
use crate::ids::PlayerId;
use crate::orders::OrderKind;
use crate::player::Faction;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;
use crate::testing::invariants::Invariants;
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);

/// Plan decision T75: "Every 0319 rate is exact: 0.25/s → 1024 a tick, 0.5/s → 2048, 1/s → 4096, 5/s → 20480", and
/// "The thresholds (10, 50, 75) compare exactly"; a point is `RATIO_ONE`.
#[test]
fn the_design_s_rates_and_amounts_are_exact() {
    assert_eq!(ONE, 65_536);
    assert_eq!([0.25, 0.5, 1.0, 5.0].map(per_tick), [1024, 2048, 4096, 20_480]);
    assert_eq!(
        [10.0, 50.0, 75.0, 200.0].map(units),
        [655_360, 3_276_800, 4_915_200, 13_107_200]
    );
    assert_eq!(points(units(50.0)), 50.0);
    assert_eq!(points(3_276_800 + 2048), 50.031_25, "a tick at 0.5 a second");
    assert_eq!(units(-1.0), 0, "none below 0");
}

/// Every type's pool converts to the integers exactly, its start within its maximum and its rate a whole number of
/// units a tick, so that no rounding builds up (T75's alternative: "a tuned rate [...] such as 0.3/s would drift").
#[test]
fn every_energy_pool_converts_exactly() {
    let pools: Vec<_> = (ObjectKind::ALL.iter())
        .filter_map(|&kind| Some((kind, kind.def().energy?)))
        .collect();
    assert!(pools.iter().any(|&(kind, _)| kind == ObjectKind::Disciple), "{pools:?}");
    for (kind, EnergyDef { max, start, regen }) in pools {
        assert!(max > 0.0 && (0.0..=max).contains(&start), "{kind:?}: {start} of {max}");
        assert_eq!((points(units(max)), points(units(start))), (max, start), "{kind:?}");
        let (EnergyRegen::PerSecond(rate) | EnergyRegen::PerUnitInside(rate)) = regen;
        assert!(rate > 0.0, "{kind:?}: {rate}");
        assert_eq!(
            per_tick(rate) * TICKS_PER_SECOND,
            units(rate),
            "{kind:?}: {rate} a second, exact a tick"
        );
    }
}

/// Up to the maximum, never above it, and a maximum that fell takes the energy down to it.
#[test]
fn energy_regenerates_up_to_the_maximum() {
    let max = units(200.0);
    assert_eq!(regenerate(units(50.0), 2048, max), units(50.0) + 2048);
    assert_eq!(regenerate(max - 1000, 2048, max), max);
    assert_eq!(regenerate(max, 2048, max), max);
    assert_eq!(regenerate(max + 5, 0, max), max);
    assert_eq!(regenerate(u32::MAX - 1, 2048, u32::MAX), u32::MAX, "no overflow");
}

/// (M5-N4b) A spell spends all of a cost or nothing, never below 0, and needs what it needs exactly: 10 points is
/// 655,360 units, and one unit less isn't enough.
#[test]
fn spending_is_all_or_nothing_and_the_need_compares_exactly() {
    let mut energy = units(10.0);
    assert!(spend(&mut energy, per_tick(5.0)));
    assert_eq!(energy, units(10.0) - 20_480);
    let mut short = 20_479;
    assert!(!spend(&mut short, 20_480), "a tick's drain it can't pay");
    assert_eq!(short, 20_479, "nothing spent");
    let mut exact = 20_480;
    assert!(spend(&mut exact, 20_480) && exact == 0);
    let builder = ScenarioBuilder::new(&"..........\n".repeat(4)).spawn(
        ObjectKind::Disciple,
        Some(P1),
        SpawnAt::Point(Vec2::new(100.0, 100.0)),
    );
    let mut disciple = Sim::new(&builder.build()).objects().next().unwrap().clone();
    for (energy, has_ten) in [(units(10.0), true), (units(10.0) - 1, false), (units(200.0), true)] {
        disciple.energy = Some(energy);
        assert_eq!(has(&disciple, 10.0), has_ten, "{energy}");
    }
    disciple.energy = None;
    assert!(!has(&disciple, 0.0), "no pool, no energy");
}

/// A change of kind (`Sim::change_kind`): into a type with a pool, the energy kept at most its maximum, or its start;
/// into a type without one, none.
#[test]
fn a_change_of_kind_keeps_the_energy_within_the_new_pool() {
    let disciple = ObjectKind::Disciple;
    assert_eq!(
        after_change(None, disciple),
        Some(units(50.0)),
        "a Recruit trained: the start"
    );
    assert_eq!(after_change(Some(units(120.0)), disciple), Some(units(120.0)));
    assert_eq!(after_change(Some(units(250.0)), disciple), Some(units(200.0)));
    assert_eq!(after_change(Some(units(120.0)), ObjectKind::Recruit), None);
    assert_eq!(after_change(None, ObjectKind::Recruit), None);
    assert_eq!(start(disciple), Some(units(50.0)));
    assert_eq!((max(disciple), max(ObjectKind::Recruit)), (Some(units(200.0)), None));
}

/// An Armory a test def gives a pool that regenerates by the units inside it (`EnergyRegen::PerUnitInside`, the
/// Shrine's "0.25 per second per Recruit inside", `factions/cults/structures/shrine.md > Placeholder stats`), starting
/// at 0: nothing while its Recruits walk to it, 1024 a tick for each of them once inside, 2048 with both, and nothing
/// again once they are out (Eject All).
#[test]
fn a_pool_that_regenerates_by_the_units_inside_counts_them_each_tick() {
    let _armory = TestDef::set(ObjectDef {
        energy: Some(EnergyDef {
            max: 200.0,
            start: 0.0,
            regen: EnergyRegen::PerUnitInside(0.25),
        }),
        ..ObjectKind::Armory.def().clone()
    });
    let armory = ObjectId(1);
    let builder = ScenarioBuilder::new(&"..............................\n".repeat(22))
        .faction(P1, Faction::Cults)
        .spawn(ObjectKind::Armory, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(Vec2::new(1050.0, 704.0)))
        .spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(Vec2::new(1250.0, 704.0)));
    let mut sim = Sim::new(&builder.build());
    let mut invariants = Invariants::new(&sim);
    let energy = |sim: &Sim| sim.object(armory).and_then(|o| o.energy);
    assert_eq!(energy(&sim), Some(0), "its start");
    let mut step = |sim: &mut Sim, commands: Vec<PlayerCommand>| {
        let inputs = TickInputs { commands };
        sim.step(&inputs);
        invariants.check(sim, &inputs).unwrap();
    };
    let enter = Command::Order {
        units: vec![ObjectId(2), ObjectId(3)],
        kind: OrderKind::Enter(armory),
        queue: false,
    };
    step(
        &mut sim,
        vec![PlayerCommand {
            player: P1,
            command: enter,
        }],
    );
    let mut gains = Vec::new();
    for _ in 0..80 {
        let before = energy(&sim).unwrap();
        step(&mut sim, Vec::new());
        let inside = sim.held_in(armory).len() as u32;
        gains.push((inside, energy(&sim).unwrap() - before));
    }
    assert!(gains.iter().all(|&(inside, gain)| gain == 1024 * inside), "{gains:?}");
    assert_eq!(gains.first(), Some(&(0, 0)), "none inside yet");
    assert_eq!(gains.last(), Some(&(2, 2048)), "both inside");
    assert!(gains.iter().any(|&(inside, _)| inside == 1), "one in before the other");
    let eject = Command::Holding {
        objects: vec![armory],
        command: crate::command::HoldCommand::EjectAll,
    };
    step(
        &mut sim,
        vec![PlayerCommand {
            player: P1,
            command: eject,
        }],
    );
    for _ in 0..40 {
        step(&mut sim, Vec::new());
    }
    assert!(sim.held_in(armory).is_empty(), "both out");
    let before = energy(&sim);
    step(&mut sim, Vec::new());
    assert_eq!(energy(&sim), before, "none inside: no regeneration");
}
