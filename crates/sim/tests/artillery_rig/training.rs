//! The Factory's R (`factions/gdo/structures/factory.md > Produces`: "Artillery Rig: 250 Space Crystals, 480 frames
//! (30 seconds) (placeholder). Requires: player owns at least one Advanced Mechanics"; design change 0435: "Cost:
//! Factory, 250 Space Crystals **+ 5 Supplies**"). The GDO starts with 1 Supply (`factions/gdo/economy.md > Starting
//! Conditions`), so the tests grant them by a debug command.

use super::*;

// The objects, by id: 1 player 1's Deployment Center, 2 and 3 its Generators (Power 70 generated), 4 its Factory, 5 its
// Advanced Mechanics (50 used: production at full speed), 6 player 2's Deployment Center, far off.
const WORKS: ObjectId = ObjectId(4);
const MECHANICS: ObjectId = ObjectId(5);
/// The Space Crystals player 1 has from tick 1.
const BANK: u32 = 5000;
/// "250 Space Crystals", "+ 5 Supplies", "480 frames".
const PRICE: u32 = 250;
const SUPPLIES: u32 = 5;
const FRAMES: Tick = 480;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// Both players play the GDO on a flat 40 × 30 map: player 1 with a Deployment Center, two Generators, a Factory on
/// tiles (16, 10) to (18, 12) and Advanced Mechanics, player 2 with a Deployment Center far off; auto-targeting off,
/// and from tick 1 [`BANK`] Space Crystals and `supplies` Supplies for player 1. Tick 1 has run.
fn works(supplies: u32) -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(2, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(5, 8))
        .spawn(ObjectKind::Factory, Some(P1), tile(16, 10))
        .spawn(ObjectKind::AdvancedMechanics, Some(P1), tile(8, 2))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(34, 24))
        .start();
    t.without_auto_targeting();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: BANK,
            supplies,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.run_until(1);
    t
}

fn production(command: ProductionCommand) -> Command {
    Command::Production {
        objects: vec![WORKS],
        command,
    }
}

/// The Factory's R.
fn train() -> Command {
    production(ProductionCommand::Train(RIG))
}

/// Player 1's Space Crystals and Supplies.
fn bank(t: &TestSim) -> (u32, u32) {
    let player = t.sim().player(P1).expect("in the game");
    (player.crystals, player.supplies)
}

/// Design change 0435's Verification: "queue a Rig at the Factory: 250 Space Crystals and 5 Supplies leave the bank;
/// cancel it: both come back". Queued, the Factory's production is the Rig's.
#[test]
fn a_rig_costs_250_crystals_and_5_supplies_both_refunded_with_a_cancel() {
    let mut t = works(SUPPLIES);
    give(&mut t, P1, train());
    assert_eq!(bank(&t), (BANK - PRICE, 0));
    let status = t.sim().production_status(WORKS).expect("in production");
    assert_eq!(status.entry.unit(), Some(RIG));
    give(&mut t, P1, production(ProductionCommand::CancelProduction));
    assert_eq!(bank(&t), (BANK, SUPPLIES), "both back");
}

/// "With fewer than 5 Supplies, R is greyed": the gate refuses the Rig with 4 (`Reject::InsufficientSupplies`, which
/// the client greys R for), and takes it with 5; without Advanced Mechanics it is refused for that
/// (`Reject::Requires`).
#[test]
fn a_rig_needs_its_5_supplies_and_advanced_mechanics() {
    let t = works(SUPPLIES - 1);
    assert_eq!(gate(&t, P1, train()), Err(Reject::InsufficientSupplies));
    let mut t = works(SUPPLIES);
    assert_eq!(gate(&t, P1, train()), Ok(()));
    give(&mut t, P1, Command::Debug(DebugCommand::Remove { id: MECHANICS }));
    assert_eq!(
        gate(&t, P1, train()),
        Err(Reject::Requires(ObjectKind::AdvancedMechanics))
    );
}

/// "480 frames": queued in tick `t`, the Rig comes out of the Factory in tick `t + 479`, the recipes' timing, an
/// Artillery Rig, mobile, with its Unit Control of 3 reserved from the start of its production.
#[test]
fn the_factory_trains_a_rig_in_480_frames() {
    let mut t = works(SUPPLIES);
    let control = |t: &TestSim| t.sim().army_cap(P1).expect("player 1's").used;
    let before = control(&t);
    let queued = give(&mut t, P1, train());
    assert_eq!(control(&t), before + 3, "reserved as its production starts");
    let produced = |t: &TestSim| {
        let out = |(tick, e): &(Tick, SimEvent)| match *e {
            SimEvent::UnitProduced { producer, unit } if producer == WORKS => Some((*tick, unit)),
            _ => None,
        };
        t.events().iter().filter_map(out).collect::<Vec<_>>()
    };
    t.run_until(queued + FRAMES - 2);
    assert!(produced(&t).is_empty());
    t.step();
    let out = produced(&t);
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(out[0].0, queued + FRAMES - 1);
    assert_eq!(object(&t, out[0].1).kind, RIG);
    assert_eq!(control(&t), before + 3);
}
