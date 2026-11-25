//! The prerequisites' one rule (`crate::tech`) through the gate, for every faction, where a test needs a requirement
//! the design's types don't have yet (`defs::test_defs`): a GDO unit's (a "Requires" clause on the Barracks'
//! Peacekeeper) and building's (a TechPrerequisite on the Barracks), a Colonists building's (on the Robo Portal), a
//! Syndicate expansion's (the Vault's kind, on the Chemical Plant) and a worker-built one's (on the Tunnel), and the
//! Grenadier's, which isn't in the game: a stand-in recipe at the Hideout behind the Chemical Plant. The Optics Lab
//! and the Guard, the design's own, are played in `crates/sim/tests/prerequisites/`.

use crate::command::{Command, DebugCommand, PlayerCommand, TunnelCommand};
use crate::construction::ConstructionCommand;
use crate::defs::test_defs::TestDef;
use crate::defs::{ObjectKind, Recipe};
use crate::ids::{ObjectId, PlayerId};
use crate::orders::OrderKind;
use crate::player::Faction;
use crate::production::ProductionCommand;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::sim::Reject;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const OPTICS_LAB: ObjectKind = ObjectKind::OpticsLab;
const CHEMICAL_PLANT: ObjectKind = ObjectKind::ChemicalPlant;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const GENERATOR: ObjectKind = ObjectKind::Generator;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// `scenario` started, both players with `crystals` Space Crystals and 1000 Supplies from tick 1, which has run.
fn started(scenario: ScenarioBuilder, crystals: u32) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    for player in [P1, P2] {
        let banks = DebugCommand::SetResources {
            player,
            crystals,
            supplies: 1000,
            alloys: 0,
            essence: 0,
            flux: 0,
        };
        t.debug_at(1, banks);
    }
    t.run_until(1);
    t
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    t.step();
}

/// A debug command in the next tick, which runs.
fn next_debug(t: &mut TestSim, command: DebugCommand) {
    next(t, P1, Command::Debug(command));
}

fn remove(id: ObjectId) -> DebugCommand {
    DebugCommand::Remove { id }
}

fn spawn(kind: ObjectKind, owner: Option<PlayerId>, x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner,
        at: tile(x, y),
        orientation: BuildingOrientation::default(),
    }
}

fn train(producer: ObjectId, kind: ObjectKind) -> Command {
    Command::Production {
        objects: vec![producer],
        command: ProductionCommand::Train(kind),
    }
}

fn expand(tunnel: ObjectId, kind: ObjectKind, x: i32, y: i32) -> Command {
    let command = TunnelCommand::Expand {
        kind,
        origin: TilePos::new(x, y),
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel { tunnel, command }
}

/// The site `tunnel` builds; panics if it builds none.
fn site_of(t: &TestSim, tunnel: ObjectId) -> ObjectId {
    match t.sim().object(tunnel).and_then(|o| o.tunnel.as_ref()?.operation) {
        Some(crate::tunnel::Operation::Building(site)) => site,
        other => panic!("Tunnel {tunnel:?} builds nothing: {other:?}"),
    }
}

/// Runs until `site` is complete, at most `ticks` ticks.
fn complete(t: &mut TestSim, site: ObjectId, ticks: u32) {
    for _ in 0..ticks {
        if t.sim().object(site).is_some_and(|o| !o.is_site()) {
            return;
        }
        t.step();
    }
    panic!("{site:?} isn't complete after {ticks} ticks");
}

/// The units of `kind` in the game, on the map or in a Tunnel Network (where a Hideout releases its units).
fn units(t: &TestSim, kind: ObjectKind) -> usize {
    let held = t.sim().held().filter(|held| held.object.kind == kind).count();
    held + t.sim().objects().filter(|o| o.kind == kind).count()
}

/// `core/tech_and_research.md > Tech Prerequisites > Counting Rule`: "A structure only counts toward a TechPrerequisite
/// once it is **fully constructed and present on the map**". Player 1's Optics Lab counts for player 1 once its site is
/// complete (a Syndicate expansion "must have finished construction"), never while it is a site, and no longer once it
/// is gone; player 2's counts for player 2 alone, and a neutral one (an expansion whose Tunnel fell, slice M4-H) for
/// no one.
#[test]
fn only_a_complete_one_of_the_players_own_counts() {
    let scenario = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 4)) // 1: area (2, 2) to (9, 9)
        .spawn(OPTICS_LAB, Some(P2), tile(30, 4)) // 2
        .spawn(OPTICS_LAB, None, tile(30, 12)); // 3: neutral
    let mut t = started(scenario, 1000);
    let met = |t: &TestSim| [P1, P2].map(|player| t.sim().requirement_met(player, OPTICS_LAB));
    assert_eq!(met(&t), [false, true]);
    next(&mut t, P1, expand(ObjectId(1), OPTICS_LAB, 2, 2));
    let site = site_of(&t, ObjectId(1));
    assert_eq!(met(&t), [false, true], "a site counts for no one");
    complete(&mut t, site, 401);
    assert_eq!(met(&t), [true, true]);
    next_debug(&mut t, remove(site));
    next_debug(&mut t, remove(ObjectId(2)));
    assert_eq!(met(&t), [false, false], "the neutral one counts for no one");
    assert!(t.sim().object(ObjectId(3)).is_some_and(|o| o.owner.is_none()));
}

/// `factions/syndicate/structures/README.md > Tunnel Expansions > Expansion Requirements`: "If an expansion requires
/// other expansion type(s), those may be located in **any** Tunnel the player owns", and the check holds back the
/// menu's entry (`Sim::can_expand`) and the Expand alike. A test kind, the Chemical Plant requiring a Hideout (as the
/// design's Vault does), is refused at Tunnel 1, whose area has no Hideout, while player 1 owns none, has one being
/// built by its other Tunnel, or sees a neutral one or player 2's; and taken once that site is complete, whatever the
/// Tunnel: the Tier Requirement comes first, then the prerequisite, then the bank.
#[test]
fn an_expansion_requires_one_anywhere_in_the_network() {
    let _vault_like = TestDef::structure(CHEMICAL_PLANT, |structure| structure.requires = &[HIDEOUT]);
    let scenario = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 4)) // 1: area (2, 2) to (9, 9)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(20, 4)) // 2: area (18, 2) to (25, 9)
        .spawn(HIDEOUT, Some(P2), tile(30, 14)) // 3
        .spawn(HIDEOUT, None, tile(4, 14)); // 4: neutral
    let mut t = started(scenario, 1000);
    let refused = Err(Reject::Requires(HIDEOUT));
    assert_eq!(t.sim().can_expand(P1, ObjectId(1), CHEMICAL_PLANT), refused);
    assert_eq!(gate(&t, P1, expand(ObjectId(1), CHEMICAL_PLANT, 2, 2)), refused);
    // Not a requirement of the Optics Lab, nor of the Hideout itself.
    assert_eq!(t.sim().can_expand(P1, ObjectId(1), OPTICS_LAB), Ok(()));
    next(&mut t, P1, expand(ObjectId(2), HIDEOUT, 18, 2));
    let site = site_of(&t, ObjectId(2));
    assert_eq!(t.sim().can_expand(P1, ObjectId(1), CHEMICAL_PLANT), refused, "a site");
    complete(&mut t, site, 401);
    assert_eq!(t.sim().can_expand(P1, ObjectId(1), CHEMICAL_PLANT), Ok(()));
    next(&mut t, P1, expand(ObjectId(1), CHEMICAL_PLANT, 2, 2));
    let plant = site_of(&t, ObjectId(1));
    assert_eq!(t.sim().object(plant).map(|o| o.kind), Some(CHEMICAL_PLANT));
    // Player 2 owns a Hideout, but no Tunnel: its refusal is the Tunnel's, not the prerequisite's.
    assert_eq!(
        t.sim().can_expand(P2, ObjectId(1), CHEMICAL_PLANT),
        Err(Reject::NotOwned(ObjectId(1)))
    );
}

/// `factions/syndicate/structures/hideout.md > Produces`: "Grenadier: 150 Space Crystals, 160 frames (10 seconds),
/// placeholder. Unit Prerequisite: player owns at least one Chemical Plant". The Grenadier isn't in the game (slice
/// M4-U), so a stand-in recipe at the Hideout with its numbers and its prerequisite, for a Guard: the M4 plan's "The
/// Chemical Plant: same, for a Grenadier test recipe". Refused before the plant, and while it is a site; the plant
/// ordered at the Tunnel is complete 400 ticks later (`factions/syndicate/structures/chemical_plant.md`: "Build Time -
/// 400 frames (25 seconds)"), and the recipe is taken. Losing it: the two queued units are produced, and a new one is
/// refused (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`: "units already queued or in
/// production continue and complete normally; new ones cannot be queued").
#[test]
fn a_grenadier_recipe_waits_for_the_chemical_plant() {
    let mut hideout = HIDEOUT.def().clone();
    let production = hideout.structure.as_mut().and_then(|s| s.production.as_mut()).unwrap();
    let grenadier = Recipe {
        kind: ObjectKind::Guard,
        cost: 150,
        frames: 160,
        requires: &[CHEMICAL_PLANT],
    };
    production.recipes = Box::leak(Box::new([production.recipes[0], grenadier]));
    let _grenadier = TestDef::set(hideout);
    let scenario = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 4)) // 1: area (2, 2) to (9, 9)
        .spawn(HIDEOUT, Some(P1), tile(8, 8)); // 2
    let mut t = started(scenario, 1000);
    let hideout = ObjectId(2);
    let refused = Err(Reject::Requires(CHEMICAL_PLANT));
    assert_eq!(gate(&t, P1, train(hideout, ObjectKind::Guard)), refused);
    assert_eq!(gate(&t, P1, train(hideout, ObjectKind::Agent)), Ok(()));
    next(&mut t, P1, expand(ObjectId(1), CHEMICAL_PLANT, 2, 2));
    let (ordered, plant) = (t.sim().tick(), site_of(&t, ObjectId(1)));
    assert_eq!(t.sim().player(P1).map(|p| p.crystals), Some(850), "150 Space Crystals");
    t.run_until(ordered + 399);
    assert!(t.sim().object(plant).is_some_and(|o| o.is_site()));
    assert_eq!(gate(&t, P1, train(hideout, ObjectKind::Guard)), refused, "a site");
    t.step();
    assert!(
        t.sim().object(plant).is_some_and(|o| !o.is_site()),
        "complete at the end of tick {ordered} + 400"
    );
    assert_eq!(gate(&t, P1, train(hideout, ObjectKind::Guard)), Ok(()));
    next(&mut t, P1, train(hideout, ObjectKind::Guard));
    next(&mut t, P1, train(hideout, ObjectKind::Guard));
    next_debug(&mut t, remove(plant));
    assert_eq!(gate(&t, P1, train(hideout, ObjectKind::Guard)), refused, "lost");
    t.run(2 * 160 + 2);
    assert_eq!(units(&t, ObjectKind::Guard), 2, "both queued ones");
}

/// The GDO's units (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`: "the structure(s) the player
/// must own before that unit can be queued at its production building"; `factions/gdo/structures/barracks.md >
/// Produces`, e.g. "Rocketman ... Requires: player owns at least one Arsenal"): a test recipe, the Barracks'
/// Peacekeeper requiring a Generator. Refused without one, the producers that would take it refusing it all; a
/// Generator held ready by the Deployment Center doesn't count ("A completed building sitting in a constructor's
/// ReadyToPlace slot does not count"), one placed does. Losing it: the queued Peacekeepers are trained, and no new one
/// is queued.
#[test]
fn a_gdo_unit_requires_a_placed_building() {
    let mut barracks = ObjectKind::Barracks.def().clone();
    let production = barracks.structure.as_mut().and_then(|s| s.production.as_mut()).unwrap();
    let peacekeeper = Recipe {
        requires: &[GENERATOR],
        ..production.recipes[0]
    };
    production.recipes = Box::leak(Box::new([peacekeeper]));
    let _requires = TestDef::set(barracks);
    let scenario = ScenarioBuilder::new(&flat(40, 24))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2)) // 1
        .spawn(ObjectKind::Barracks, Some(P1), tile(10, 2)) // 2
        .spawn(ObjectKind::Barracks, Some(P1), tile(10, 8)); // 3
    let mut t = started(scenario, 1000);
    let both = Command::Production {
        objects: vec![ObjectId(2), ObjectId(3)],
        command: ProductionCommand::Train(ObjectKind::Peacekeeper),
    };
    let refused = Err(Reject::Requires(GENERATOR));
    assert_eq!(gate(&t, P1, both.clone()), refused);
    let construct = Command::Construction {
        objects: vec![ObjectId(1)],
        command: ConstructionCommand::Construct(GENERATOR),
    };
    next(&mut t, P1, construct);
    let ready = |t: &TestSim| t.sim().object(ObjectId(1)).and_then(|o| o.constructor?.ready());
    while ready(&t).is_none() {
        assert!(t.sim().tick() < 400, "the Generator is ready");
        t.step();
    }
    assert_eq!(gate(&t, P1, both.clone()), refused, "ready, not placed");
    let place = Command::Construction {
        objects: vec![ObjectId(1)],
        command: ConstructionCommand::Place {
            origin: TilePos::new(6, 2),
            orientation: BuildingOrientation::default(),
        },
    };
    next(&mut t, P1, place);
    let generator = (t.sim().objects())
        .find(|o| o.kind == GENERATOR)
        .map(|o| o.id)
        .expect("placed");
    assert_eq!(gate(&t, P1, both.clone()), Ok(()));
    next(&mut t, P1, both.clone());
    next_debug(&mut t, remove(generator));
    assert_eq!(gate(&t, P1, both), refused, "lost");
    for _ in 0..600 {
        if units(&t, ObjectKind::Peacekeeper) == 2 {
            break;
        }
        t.step();
    }
    assert_eq!(units(&t, ObjectKind::Peacekeeper), 2, "both queued ones");
}

/// The GDO's buildings (`factions/gdo/structures/deployment_center.md > Constructs`: "a prerequisite building only
/// counts once it has been **placed** on the map ... and is checked only when construction is started"): a test
/// TechPrerequisite, the Barracks requiring a Generator, holds back the Deployment Center's Construct
/// ([`Reject::Requires`] before the constructor's being busy or the bank), until a Generator stands. Losing it, the
/// Barracks under construction is completed and placed ("Constructions already started (including GDO buildings in
/// progress or ReadyToPlace) continue and can be completed/placed normally"), and a new one is refused.
#[test]
fn a_gdo_building_requires_a_placed_building() {
    let _requires = TestDef::structure(ObjectKind::Barracks, |structure| structure.requires = &[GENERATOR]);
    let scenario = ScenarioBuilder::new(&flat(40, 24))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2)) // 1
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 10)); // 2
    let mut t = started(scenario, 1000);
    let construct = |dc: u32| Command::Construction {
        objects: vec![ObjectId(dc)],
        command: ConstructionCommand::Construct(ObjectKind::Barracks),
    };
    let refused = Err(Reject::Requires(GENERATOR));
    assert_eq!(gate(&t, P1, construct(1)), refused);
    next_debug(&mut t, spawn(GENERATOR, Some(P1), 10, 2));
    let generator = (t.sim().objects()).find(|o| o.kind == GENERATOR).map(|o| o.id).unwrap();
    assert_eq!(gate(&t, P1, construct(1)), Ok(()));
    next(&mut t, P1, construct(1));
    next_debug(&mut t, remove(generator));
    assert_eq!(gate(&t, P1, construct(2)), refused, "lost");
    // The busy constructor's reason comes after the prerequisite's.
    assert_eq!(gate(&t, P1, construct(1)), refused);
    let ready = |t: &TestSim| t.sim().object(ObjectId(1)).and_then(|o| o.constructor?.ready());
    while ready(&t).is_none() {
        assert!(t.sim().tick() < 400, "the Barracks is ready");
        t.step();
    }
    let place = Command::Construction {
        objects: vec![ObjectId(1)],
        command: ConstructionCommand::Place {
            origin: TilePos::new(6, 2),
            orientation: BuildingOrientation::default(),
        },
    };
    assert_eq!(gate(&t, P1, place.clone()), Ok(()));
    next(&mut t, P1, place);
    assert!(t.sim().objects().any(|o| o.kind == ObjectKind::Barracks), "placed");
}

/// The Colonists' (`factions/colonists/structures/README.md > Tech Model`: "Defensive structures are an exception to
/// the no-prerequisites rule: they may have TechPrerequisites"; `core/tech_and_research.md > Tech Prerequisites >
/// Counting Rule`: "the Warp In transformation must have completed. A Beacon still mid-transformation does not count"):
/// a test TechPrerequisite, the Robo Portal requiring an Egress Node, holds back a Warp In ("Warp In begun") until an
/// Egress Node warped in by the other Beacon is complete.
#[test]
fn a_warp_in_requires_a_complete_building() {
    let _requires = TestDef::structure(ObjectKind::RoboPortal, |structure| {
        structure.requires = &[ObjectKind::EgressNode];
    });
    let scenario = ScenarioBuilder::new(&flat(40, 24))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::Beacon, Some(P1), tile(4, 4)) // 1
        .spawn(ObjectKind::Beacon, Some(P1), tile(20, 4)); // 2
    let mut t = started(scenario, 1000);
    let warp_in = |beacon: u32, kind| Command::Construction {
        objects: vec![ObjectId(beacon)],
        command: ConstructionCommand::WarpIn {
            kind,
            orientation: BuildingOrientation::default(),
        },
    };
    let refused = Err(Reject::Requires(ObjectKind::EgressNode));
    assert_eq!(gate(&t, P1, warp_in(1, ObjectKind::RoboPortal)), refused);
    next(&mut t, P1, warp_in(2, ObjectKind::EgressNode));
    assert!(t.sim().object(ObjectId(2)).is_some_and(|o| o.is_transforming()));
    assert_eq!(
        gate(&t, P1, warp_in(1, ObjectKind::RoboPortal)),
        refused,
        "mid-transformation"
    );
    while t.sim().object(ObjectId(2)).is_some_and(|o| o.is_transforming()) {
        assert!(t.sim().tick() < 400, "the Egress Node warps in");
        t.step();
    }
    assert_eq!(gate(&t, P1, warp_in(1, ObjectKind::RoboPortal)), Ok(()));
}

/// A worker's building (`StructureDef::requires` wherever it is built; the Laser Turret's comes with slice M4-S1): a
/// test TechPrerequisite, the Tunnel requiring a Hideout, refuses an Agent's Build order as it is given, until player 1
/// owns a Hideout.
#[test]
fn a_workers_build_requires_its_buildings_prerequisite() {
    let _requires = TestDef::structure(ObjectKind::Tunnel, |structure| structure.requires = &[HIDEOUT]);
    let scenario = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(
            ObjectKind::Agent,
            Some(P1),
            SpawnAt::Point(TilePos::new(20, 10).center()),
        ); // 1
    let mut t = started(scenario, 1000);
    let build = Command::Order {
        units: vec![ObjectId(1)],
        kind: OrderKind::Build {
            kind: ObjectKind::Tunnel,
            origin: TilePos::new(4, 4),
            orientation: BuildingOrientation::default(),
        },
        queue: false,
    };
    assert_eq!(gate(&t, P1, build.clone()), Err(Reject::Requires(HIDEOUT)));
    next_debug(&mut t, spawn(HIDEOUT, Some(P1), 30, 14));
    assert_eq!(gate(&t, P1, build), Ok(()));
}
