//! The Barracks' Drone Operator (`factions/gdo/structures/barracks.md > Produces`): "Drone Operator: 100 Space
//! Crystals, 160 frames (10 seconds) (placeholder). Requires: player owns at least one Career Campus", and its Supply
//! (design changes 0435 and 0441: "deducts 100 Space Crystals and 1 Supply"), the first unit with a Supply cost: paid
//! with the crystals as it is queued, both required, refunded with them on a cancel and with its Barracks.

use sc_sim::{ProductionCommand, Refusal, TilePos};

use super::*;

// The objects of [`barracks`], by id: 1 player 1's Deployment Center on tiles (2, 2) to (5, 5) (Power +30), 2 a
// Generator on tiles (8, 2) to (9, 3) (+20), 3 the Barracks below, 4 the Career Campus below, 5 player 2's
// Deployment Center, far off on tiles (34, 24) to (37, 27).
/// Player 1's Barracks, on tiles (12, 3) to (14, 4) (Power −30).
const BARRACKS: u32 = 3;
/// Player 1's Career Campus, on tiles (8, 7) to (9, 8) (Power −20).
const CAMPUS: u32 = 4;
/// Player 1's Space Crystals in [`barracks`].
const BANK: u32 = 1000;
/// `> Produces`: the operator's price and time.
const COST: u32 = 100;
const FRAMES: Tick = 160;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// Both players play the GDO on a flat 40 × 30 map, player 1 with [`BARRACKS`], a Career Campus and power for it,
/// [`BANK`] Space Crystals and `supplies` Supplies from tick 1, and `more` after them; tick 1 has run.
fn barracks(supplies: u32, more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(40, 30))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::Barracks, Some(P1), tile(12, 3))
        .spawn(ObjectKind::CareerCampus, Some(P1), tile(8, 7))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(34, 24));
    let mut t = more(builder).start();
    bank(&mut t, 1, BANK, supplies);
    t.run_until(1);
    t
}

/// R at the Barracks (or `producers`): a Train of a Drone Operator.
fn train_at(producers: &[u32], kind: ObjectKind) -> Command {
    Command::Production {
        objects: producers.iter().copied().map(id).collect(),
        command: ProductionCommand::Train(kind),
    }
}

fn train(kind: ObjectKind) -> Command {
    train_at(&[BARRACKS], kind)
}

fn queued(sim: &Sim, producer: u32) -> Vec<ObjectKind> {
    let production = object(sim, producer).production.as_ref().expect("a producer");
    production.queue.iter().filter_map(|entry| entry.unit()).collect()
}

/// The units produced so far: (tick, producer, unit).
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId, ObjectId)> {
    let unit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::UnitProduced { producer, unit } => Some((*tick, producer, unit)),
        _ => None,
    };
    t.events().iter().filter_map(unit).collect()
}

/// "**R: Build Drone Operator** [...] deducts 100 Space Crystals and 1 Supply" (design change 0441), "greyed without
/// it": with no Supply the gate refuses R (`Reject::InsufficientSupplies`, the client's greyed button and its feedback
/// line) and the bank keeps everything, while a Peacekeeper, which needs none, still trains. With 1 Supply the Barracks
/// takes it with the 100 Space Crystals as it queues the operator, and a second R finds no Supply. The operator comes
/// out 160 frames later with its one drone in stock and none out (`> Drone Stock`: "Starts with **1** drone when
/// trained").
#[test]
fn the_drone_operator_costs_100_crystals_and_a_supply() {
    let mut t = barracks(0, |builder| builder);
    assert_eq!(gate(t.sim(), P1, train(OPERATOR)), Err(Reject::InsufficientSupplies));
    assert_eq!(gate(t.sim(), P1, train(PK)), Ok(()));
    next(&mut t, P1, train(PK));
    assert_eq!(queued(t.sim(), BARRACKS), [PK]);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK - 50, 0), "no Supply");

    let mut t = barracks(1, |builder| builder);
    assert_eq!(gate(t.sim(), P1, train(OPERATOR)), Ok(()));
    next(&mut t, P1, train(OPERATOR));
    let queued_at = t.sim().tick();
    assert_eq!(queued(t.sim(), BARRACKS), [OPERATOR]);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK - COST, 0));
    assert_eq!(
        gate(t.sim(), P1, train(OPERATOR)),
        Err(Reject::InsufficientSupplies),
        "the crystals suffice, the Supply is spent"
    );
    while produced(&t).is_empty() {
        assert!(t.sim().tick() < queued_at + 2 * FRAMES, "the operator is late");
        t.step();
    }
    let [(tick, producer, unit)] = produced(&t)[..] else {
        panic!("one unit: {:?}", produced(&t));
    };
    assert_eq!((tick, producer), (queued_at + FRAMES - 1, id(BARRACKS)), "160 frames");
    let operator = t.sim().object(unit).expect("out of the Barracks");
    assert_eq!(
        (operator.kind, operator.owner, operator.drone),
        (OPERATOR, Some(P1), None)
    );
    assert_eq!(operator.stock, Some(Stock { count: 1, buying: None }));
}

/// "Refunded with them on cancel" (design change 0441): the Barracks' Cancel Production gives back the operator's 100
/// Space Crystals and its Supply; and the Barracks lost with an operator queued refunds both, "as if cancelled"
/// (`factions/gdo/structures/barracks.md > Destroyed while producing`).
#[test]
fn a_cancel_and_the_barracks_loss_refund_the_supply() {
    let mut t = barracks(1, |builder| builder);
    next(&mut t, P1, train(OPERATOR));
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK - COST, 0));
    let cancel = Command::Production {
        objects: vec![id(BARRACKS)],
        command: ProductionCommand::CancelProduction,
    };
    next(&mut t, P1, cancel);
    assert_eq!(queued(t.sim(), BARRACKS), []);
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK, 1), "both back");

    next(&mut t, P1, train(OPERATOR));
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK - COST, 0));
    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: id(BARRACKS) }));
    assert_eq!(
        (crystals(t.sim()), supplies(t.sim())),
        (BANK, 1),
        "both back with the Barracks"
    );
}

/// "Requires: player owns at least one Career Campus": with its only Career Campus gone the Barracks refuses R
/// (`Reject::Requires`, the client's "Requires a Career Campus") whatever the bank.
#[test]
fn the_drone_operator_requires_a_career_campus() {
    let mut t = barracks(5, |builder| builder);
    assert_eq!(gate(t.sim(), P1, train(OPERATOR)), Ok(()));
    next(&mut t, P1, Command::Debug(DebugCommand::Remove { id: id(CAMPUS) }));
    let refused = Err(Reject::Requires(ObjectKind::CareerCampus));
    assert_eq!(gate(t.sim(), P1, train(OPERATOR)), refused);
}

/// Design change 0083's serving with a second cost (`systems/controls/selection.md > Group production with limited
/// resources`: "crystals or any other resource paid at queue time"): two Barracks given R with 1 Supply between them,
/// and crystals for both. The one with the least work queued, the first by id on a tie, takes the Supply; the other is
/// refused for want of one (`Refusal::Supplies`), and the bank keeps its crystals.
#[test]
fn of_two_barracks_only_one_finds_a_supply() {
    // A second Barracks (6) on tiles (12, 7) to (14, 8), and a Generator (7) on tiles (16, 2) to (17, 3) for it.
    let mut t = barracks(1, |builder| {
        builder
            .spawn(ObjectKind::Barracks, Some(P1), tile(12, 7))
            .spawn(ObjectKind::Generator, Some(P1), tile(16, 2))
            .spawn(ObjectKind::Generator, Some(P1), tile(16, 5))
    });
    let both = [BARRACKS, 6];
    next(&mut t, P1, train_at(&both, OPERATOR));
    assert_eq!(
        (queued(t.sim(), BARRACKS), queued(t.sim(), 6)),
        (vec![OPERATOR], vec![])
    );
    assert_eq!((crystals(t.sim()), supplies(t.sim())), (BANK - COST, 0));
    let refusal = (t.events().iter()).find_map(|(_, e)| match *e {
        SimEvent::ProductionRefused { producer, kind, reason } => Some((producer, kind, reason)),
        _ => None,
    });
    assert_eq!(refusal, Some((id(6), OPERATOR, Refusal::Supplies)));
}
