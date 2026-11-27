//! The Arsenal and its Rocketmen (`core/tech_and_research.md > Tech Prerequisites`): the Barracks' "Rocketman [...]
//! Requires: player owns at least one Arsenal" (`factions/gdo/structures/barracks.md > Produces`), and the Arsenal's
//! own TechPrerequisite, "player owns at least one Barracks" (`factions/gdo/structures/deployment_center.md >
//! Constructs`); and what it adds to its owner's build area (`factions/gdo/structures/arsenal.md`).

use sc_sim::{Command, ConstructionCommand, Constructor, DebugCommand, PlayerCommand, ProductionCommand, Reject, Sim};

use super::*;

// The objects of [`gdo`], by id: 1 and 3 below, 2 player 1's Generator on tiles (8, 2) to (9, 3) (Power +20, for the
// Arsenal's −20), 4 player 2's Deployment Center, far off on tiles (34, 24) to (37, 27).
/// Player 1's Deployment Center, on tiles (2, 2) to (5, 5) (Power +30; its build area reaches tile 17 east and south).
const DC: u32 = 1;
/// Player 1's Barracks, on tiles (12, 3) to (14, 4) (Power −30).
const BARRACKS: u32 = 3;
/// Player 1's bank in [`gdo`], Space Crystals.
const BANK: u32 = 1000;
/// `factions/gdo/structures/barracks.md > Produces`: "Rocketman: 100 Space Crystals, 160 frames (10 seconds)
/// (placeholder)".
const ROCKETMAN_COST: u32 = 100;
/// `factions/gdo/structures/deployment_center.md > Constructs`: "Arsenal: 150 Space Crystals, 320 frames (20 seconds)
/// (placeholder)".
const ARSENAL_COST: u32 = 150;
/// The longest training in these tests, ticks.
const LONGEST: u32 = 1000;

/// Both players play the GDO on a flat 40 × 30 map: player 1 with [`DC`], a Generator and [`BARRACKS`] and [`BANK`]
/// Space Crystals, player 2 with a Deployment Center far off; `more` adds to the scenario after them. Tick 1 has run.
fn gdo(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let scenario = ScenarioBuilder::new(&flat(40, 30))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(8, 2))
        .spawn(ObjectKind::Barracks, Some(P1), tile(12, 3))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(34, 24));
    let mut t = more(scenario).start();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: BANK,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.run_until(1);
    t
}

/// What the gate says to player 1's `command` now (`Sim::validate`).
fn gate(sim: &Sim, command: Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand { player: P1, command })
}

/// Player 1's `command` in the next tick, which runs.
fn next(t: &mut TestSim, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    t.step();
}

/// A Train of a Rocketman at the Barracks: its W.
fn train() -> Command {
    Command::Production {
        objects: vec![id(BARRACKS)],
        command: ProductionCommand::Train(ROCKETMAN),
    }
}

/// The Deployment Center's Construct of an Arsenal: BuildMenu\[Basic\]'s S.
fn construct_arsenal() -> Command {
    Command::Construction {
        objects: vec![id(DC)],
        command: ConstructionCommand::Construct(ARSENAL),
    }
}

/// Player 1's Space Crystals.
fn crystals(sim: &Sim) -> u32 {
    sim.player(P1).expect("in the game").crystals
}

/// The units the Barracks produced so far: (tick, unit).
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    let produced = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::UnitProduced { producer, unit } if producer == id(BARRACKS) => Some((*tick, unit)),
        _ => None,
    };
    t.events().iter().filter_map(produced).collect()
}

/// Runs `t` until the Barracks has produced `n` units in all; panics after [`LONGEST`] ticks of waiting for each.
fn until_produced(t: &mut TestSim, n: usize) -> Vec<(Tick, ObjectId)> {
    for _ in 0..n as u32 * LONGEST {
        if produced(t).len() >= n {
            return produced(t);
        }
        t.step();
    }
    panic!("the Barracks produced {} of {n}", produced(t).len());
}

/// "Requires: player owns at least one Barracks": with its only Barracks gone, player 1's Deployment Center is refused
/// the Arsenal (`Reject::Requires`, the client's "Requires a Barracks") and stays idle; with a Barracks it takes it,
/// for 150 Space Crystals.
#[test]
fn the_arsenal_requires_a_barracks() {
    let mut t = gdo(|scenario| scenario);
    assert_eq!(gate(t.sim(), construct_arsenal()), Ok(()));
    next(&mut t, Command::Debug(DebugCommand::Remove { id: id(BARRACKS) }));
    let refused = Err(Reject::Requires(ObjectKind::Barracks));
    assert_eq!(gate(t.sim(), construct_arsenal()), refused);
    next(&mut t, construct_arsenal());
    let dc = t.sim().object(id(DC)).expect("the Deployment Center");
    assert_eq!((dc.constructor, crystals(t.sim())), (Some(Constructor::Idle), BANK));

    let mut t = gdo(|scenario| scenario);
    next(&mut t, construct_arsenal());
    let dc = t.sim().object(id(DC)).expect("the Deployment Center");
    assert!(matches!(
        dc.constructor,
        Some(Constructor::Constructing { kind: ARSENAL, .. })
    ));
    assert_eq!(crystals(t.sim()), BANK - ARSENAL_COST);
}

/// "Requires: player owns at least one Arsenal": the Barracks is refused a Rocketman while player 1 has none (W
/// greyed, "Requires an Arsenal"), player 2's Arsenal not counting; with an Arsenal of player 1's it takes one, for 100
/// Space Crystals.
#[test]
fn the_rocketman_requires_an_arsenal_of_its_owner() {
    let refused = Err(Reject::Requires(ARSENAL));
    let t = gdo(|scenario| scenario.spawn(ARSENAL, Some(P2), tile(30, 24)));
    assert_eq!(gate(t.sim(), train()), refused, "an enemy's doesn't count");
    assert!(!t.sim().requirement_met(P1, ARSENAL));

    let mut t = gdo(|scenario| scenario.spawn(ARSENAL, Some(P1), tile(8, 7)));
    assert!(t.sim().requirement_met(P1, ARSENAL));
    assert_eq!(gate(t.sim(), train()), Ok(()));
    next(&mut t, train());
    assert_eq!(crystals(t.sim()), BANK - ROCKETMAN_COST);
}

/// `core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`, "Losing a Prerequisite": "units already
/// queued or in production continue and complete normally; new ones cannot be queued until the player again owns a
/// qualifying prerequisite structure". Two Rocketmen queued, the first in production, and then the only Arsenal
/// destroyed: the Barracks is refused a third (W greyed again), and both queued ones come out, for 100 Space Crystals
/// each.
#[test]
fn rocketmen_queued_before_the_arsenal_is_lost_come_out() {
    let mut t = gdo(|scenario| scenario.spawn(ARSENAL, Some(P1), tile(8, 7)));
    let arsenal = 5;
    next(&mut t, train());
    next(&mut t, train());
    let production = t.sim().object(id(BARRACKS)).and_then(|o| o.production.clone());
    let production = production.expect("a producer");
    assert_eq!(
        (production.in_production(), production.queue.len()),
        (Some(ROCKETMAN), 2)
    );
    next(&mut t, Command::Debug(DebugCommand::Remove { id: id(arsenal) }));
    assert!(!t.sim().requirement_met(P1, ARSENAL));
    assert_eq!(gate(t.sim(), train()), Err(Reject::Requires(ARSENAL)));
    let produced = until_produced(&mut t, 2);
    for (_, unit) in produced {
        let o = t.sim().object(unit).expect("out of the Barracks");
        assert_eq!((o.kind, o.owner), (ROCKETMAN, Some(P1)));
    }
    assert_eq!(crystals(t.sim()), BANK - 2 * ROCKETMAN_COST);
    let production = t.sim().object(id(BARRACKS)).and_then(|o| o.production.clone());
    assert_eq!(production.map(|p| p.queue.len()), Some(0), "no third");
}

/// "BuildRadiusExtension 2 grid units" (`factions/gdo/structures/arsenal.md`; `factions/gdo/structures/README.md >
/// GDOBuildArea`): an Arsenal of player 1's on tiles (24, 10) to (25, 11), east of what their other buildings reach
/// (tile 17), adds tiles (22, 8) to (27, 13) to their build area, its footprint grown by 2 on every side, and not a
/// tile more.
#[test]
fn the_arsenal_extends_the_build_area_by_2() {
    let t = gdo(|scenario| scenario.spawn(ARSENAL, Some(P1), tile(24, 10)));
    let area = t.sim().build_area(P1);
    let edges = [
        ((22, 10), (21, 10)),
        ((27, 11), (28, 11)),
        ((24, 8), (24, 7)),
        ((25, 13), (25, 14)),
    ];
    for ((x, y), (out_x, out_y)) in edges {
        assert!(area.contains(TilePos::new(x, y)), "({x}, {y}) is in the area");
        assert!(!area.contains(TilePos::new(out_x, out_y)), "({out_x}, {out_y}) is not");
    }
    let without = gdo(|scenario| scenario);
    assert!(!without.sim().build_area(P1).contains(TilePos::new(22, 10)));
}
