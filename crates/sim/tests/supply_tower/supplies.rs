//! The GDO's Supplies: the bank's 1 at the start (`factions/gdo/economy.md > Starting Conditions`), and those a run
//! brings, which the units that cost Supplies need.

use sc_sim::defs::PLANNED_RECIPES;
use sc_sim::scenario::DEFAULT_SEED;
use sc_sim::{Faction, GameSetup, MapDef, ProductionCommand, Scenario, Sim};

use super::*;

const BARRACKS: ObjectKind = ObjectKind::Barracks;
const CAMPUS: ObjectKind = ObjectKind::CareerCampus;

fn train(producer: u32, kind: ObjectKind) -> Command {
    Command::Production {
        objects: vec![id(producer)],
        command: ProductionCommand::Train(kind),
    }
}

/// A new GDO game (`factions/gdo/economy.md > Starting Conditions`): on the playtest map, the GDO in both slots, 100
/// Space Crystals and 1 Supply each; from the starting bank, a Field Engineer is queued once a Career Campus stands (a
/// Barracks and a Career Campus spawned: its 75 Space Crystals and its 1 Supply, refused before the Campus stands,
/// `Reject::Requires`), and a Prospector's 8 Supplies are 7 more than the bank's (the Prospector itself is held,
/// `defs::PLANNED_RECIPES`).
#[test]
fn a_new_gdo_game_starts_with_1_supply() {
    let map: MapDef = serde_json::from_str(include_str!("../../../../maps/playtest.json")).unwrap();
    let setup = GameSetup::per_slot(&map, &[Faction::Gdo, Faction::Gdo], DEFAULT_SEED);
    let sim = Sim::try_new(&Scenario { map, setup }).unwrap();
    for player in sim.players() {
        assert_eq!((player.crystals, player.supplies), (100, 1), "player {}", player.id.0);
    }

    let mut t = ScenarioBuilder::new(&flat(40, 30))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(BARRACKS, Some(P1), tile(12, 3))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(34, 24))
        .start();
    t.run_until(1);
    assert_eq!(banks(&t, P1), (100, 1));
    let field_engineer = train(3, ObjectKind::FieldEngineer);
    assert_eq!(gate(&t, &field_engineer), Err(Reject::Requires(CAMPUS)));
    let campus = DebugCommand::Spawn {
        kind: CAMPUS,
        owner: Some(P1),
        at: tile(12, 7),
        orientation: Default::default(),
    };
    debug(&mut t, campus);
    next(&mut t, field_engineer);
    assert_eq!(banks(&t, P1), (25, 0), "75 Space Crystals and its Supply");

    let prospector = (PLANNED_RECIPES.iter())
        .find(|(_, recipe)| recipe.kind == ObjectKind::Prospector)
        .map(|(_, recipe)| recipe.supplies);
    let start = sc_sim::StartingConditions::of(Faction::Gdo).supplies;
    assert_eq!(prospector.map(|supplies| supplies - start), Some(7));
}

/// The units that cost Supplies (`factions/gdo/units/field_engineer.md > Placeholder stats`,
/// `factions/gdo/units/drone_operator.md > Placeholder stats`), with the loop: with 0 Supplies the Barracks is refused
/// a Drone Operator and a Field Engineer (`Reject::InsufficientSupplies`, the client's greyed buttons) and still trains
/// a Peacekeeper; the free chopper's run to a station holding 4 brings them, and the Drone Operator is queued, for its
/// 1 Supply.
#[test]
fn a_run_brings_the_supplies_a_drone_operator_needs() {
    let mut t = base(|b| {
        b.spawn_with(station(4, 960), tile(14, 8))
            .spawn(BARRACKS, Some(P1), tile(12, 2))
            .spawn(CAMPUS, Some(P1), tile(18, 2))
            .spawn(ObjectKind::Generator, Some(P1), tile(22, 2))
    });
    let (s, barracks) = (FIRST, FIRST + 1);
    let (tower, chopper) = placed_tower(&mut t, TilePos::new(2, 8));
    assert_eq!(banks(&t, P1).1, 0);
    let operator = train(barracks, ObjectKind::DroneOperator);
    for kind in [ObjectKind::DroneOperator, ObjectKind::FieldEngineer] {
        assert_eq!(
            gate(&t, &train(barracks, kind)),
            Err(Reject::InsufficientSupplies),
            "{kind:?}"
        );
    }
    assert_eq!(gate(&t, &train(barracks, ObjectKind::Peacekeeper)), Ok(()));
    next(&mut t, pick_up(&[chopper], s));
    step_until(&mut t, 600, "loaded", |t| carried(t, chopper) == DELIVERY_SIZE);
    next(&mut t, drop_off(&[chopper], tower));
    step_until(&mut t, 600, "paid in", |t| banks(t, P1).1 == DELIVERY_SIZE);
    let crystals = banks(&t, P1).0;
    next(&mut t, operator);
    assert_eq!(banks(&t, P1), (crystals - 100, DELIVERY_SIZE - 1));
}
