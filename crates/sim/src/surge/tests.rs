//! The surges' numbers (`factions/syndicate/units/laser_battery.md > Placeholder stats`), and the battery where a test
//! needs the sim's inside: through a change of kind into its other mode, and held in the Tunnel Network.

use std::collections::BTreeMap;

use glam::Vec2;

use super::*;
use crate::command::{Ability, Command, PlayerCommand, TickInputs};
use crate::energy::units;
use crate::ids::{ObjectId, PlayerId};
use crate::network::{Held, Place};
use crate::orders::OrderKind;
use crate::player::Faction;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn laser_battery() -> SurgeDef {
    of_kind(ObjectKind::LaserBattery).expect("the Laser Battery's surges")
}

/// "**Damage per surge = 4 × charge^(2/3)**: a normal surge deals about 18.6; an empty-battery surge (5) about 11.7;
/// Amplify from full (100) about 86", to f32 precision: within an f32 epsilon of the reference in f64 (`libm`, the
/// same bits on every platform, as `math::cbrt`'s).
#[test]
fn a_surge_deals_4_times_the_charge_it_uses_to_the_two_thirds() {
    let def = laser_battery();
    for (points, about) in [(10.0, "18.6"), (5.0, "11.7"), (100.0, "86")] {
        let dealt = damage(&def, units(points as f32));
        let root = libm::cbrt(points);
        let reference = 4.0 * root * root;
        assert!(
            (f64::from(dealt) - reference).abs() <= reference * f64::from(f32::EPSILON),
            "{points}: {dealt} against {reference}"
        );
        let shown = if about.contains('.') {
            format!("{dealt:.1}")
        } else {
            format!("{dealt:.0}")
        };
        assert_eq!(shown, about, "{points}");
    }
    assert_eq!(damage(&def, 0), 0.0, "nothing from nothing");
}

/// `> Battery`: "a surge uses the normal surge charge if the battery holds it, otherwise whatever is left"; Amplify's,
/// the whole battery.
#[test]
fn a_surge_uses_its_draw_what_is_left_or_all_once_amplified() {
    let def = laser_battery();
    assert_eq!(used(&def, units(100.0), false), units(10.0));
    assert_eq!(used(&def, units(7.5), false), units(7.5));
    assert_eq!(used(&def, units(100.0), true), units(100.0));
    assert_eq!(used(&def, units(3.0), true), units(3.0));
    assert_eq!((used(&def, 0, false), used(&def, 0, true)), (0, 0));
}

/// Amplify is the units' whose attack has surges: the Laser Battery's, in both modes, and no other type's.
#[test]
fn only_the_laser_battery_s_two_types_have_surges() {
    let with: Vec<_> = (ObjectKind::ALL.iter())
        .filter(|&&kind| of_kind(kind).is_some())
        .collect();
    assert_eq!(with, [&ObjectKind::LaserBattery, &ObjectKind::LaserBatteryDeployed]);
}

/// The battery always charges, 5 a second (20480 a tick): emptied by an Amplified surge, it charges on the map, keeps
/// its charge and Amplify's mark through the change of kind into its other mode and charges there, and charges held
/// in the Tunnel Network (`energy::step`, which charges every held unit).
#[test]
fn the_battery_charges_through_a_change_of_kind_and_in_the_network() {
    let builder = ScenarioBuilder::new(&vec![".".repeat(30); 22].join("\n"))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(
            ObjectKind::LaserBattery,
            Some(P1),
            SpawnAt::Point(Vec2::new(640.0, 640.0)),
        )
        .spawn(
            ObjectKind::FieldEngineer,
            Some(P2),
            SpawnAt::Point(Vec2::new(900.0, 640.0)),
        );
    let mut sim = Sim::new(&builder.build());
    let battery = ObjectId(1);
    let energy = |sim: &Sim| sim.object(battery).and_then(|o| o.energy).expect("a battery");
    let a_tick = energy::per_tick(5.0);
    assert_eq!((energy(&sim), a_tick), (units(100.0), 20480), "full; 5 a second");
    let give = |command| PlayerCommand { player: P1, command };
    let amplify = Command::Ability {
        units: vec![battery],
        ability: Ability::Amplify,
    };
    let attack = Command::Order {
        units: vec![battery],
        kind: OrderKind::Attack(ObjectId(2)),
        queue: false,
    };
    let idle = TickInputs { commands: Vec::new() };
    sim.step(&TickInputs {
        commands: vec![give(amplify.clone()), give(attack)],
    });
    for _ in 0..40 {
        if energy(&sim) <= a_tick {
            break;
        }
        sim.step(&idle);
    }
    assert_eq!(
        energy(&sim),
        a_tick,
        "emptied by the Amplified surge, then a tick's charge"
    );
    let o = sim.object(battery).unwrap();
    assert!(!o.amplified, "the mark is spent with its surge");
    sim.step(&TickInputs {
        commands: vec![give(amplify)],
    });
    let before = energy(&sim);
    assert_eq!(before, 2 * a_tick, "charging on the map");
    assert!(sim.change_kind(battery, ObjectKind::LaserBatteryDeployed, None));
    let o = sim.object(battery).unwrap();
    assert_eq!(
        (o.kind, o.energy, o.amplified),
        (ObjectKind::LaserBatteryDeployed, Some(before), true),
        "its charge and its mark kept"
    );
    sim.step(&idle);
    assert_eq!(energy(&sim), before + a_tick, "charging deployed");
    let mut object = sim.object(battery).unwrap().clone();
    object.energy = Some(0);
    let mut held = BTreeMap::from([(
        battery,
        Held {
            object,
            place: Place::Network,
            since: 0,
            entered: true,
            leaving: None,
            service: Default::default(),
        },
    )]);
    energy::step(&mut BTreeMap::new(), &mut held, ObjectId(u32::MAX));
    assert_eq!(
        held[&battery].object.energy,
        Some(a_tick),
        "charging in the Tunnel Network"
    );
}
