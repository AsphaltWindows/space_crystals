//! (M6-GK) The GDO's kinds of design change 0366 before the slices that make them live (plan decision T84, after T69's
//! rule): no player makes one. The Deployment Center constructs none of the new buildings and the Barracks trains none
//! of the new units but (M6-GD) the Drone Operator; a Field Engineer builds no Bunker and no First Aid Tent; the new
//! buildings produce and construct nothing; and no game start spawns one. (M6-GR) The Radar Station, the Defense System
//! and the Missile Turret are live since slice M6-GR, and (M6-GF) the Factory, Advanced Mechanics, the Industrial
//! Powerplant, the Raptor and the Twister since slice M6-GF ([`LIVE`]; their tests are `tests/radar/`,
//! `tests/defense_system/` and `tests/factory/`). The debug spawn, the map editor's, places every one, the new units
//! take orders and move, and a deployed Artillery Rig given a Move stays where it is (plan decision T87).

use sc_sim::defs::PLANNED_RECIPES;
use sc_sim::scenario::DEFAULT_SEED;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Category, Command, ConstructionCommand, DebugCommand, Faction, GameSetup, MapDef, MoveStatus,
    ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ProductionCommand, Reject, Scenario, Sim, SimEvent,
    SpawnAt, StartingConditions, Target, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);

/// Player 1's Deployment Center, Barracks and Field Engineer in [`base`].
const DC: u32 = 1;
const BARRACKS: u32 = 2;
const FIELD_ENGINEER: u32 = 3;

/// (M6-GR) Slice M6-GK's kinds that are live: the Deployment Center constructs the Radar Station and the Defense
/// System, the Defense System the Missile Turret; (M6-GF) the Deployment Center the Factory, Advanced Mechanics and the
/// Industrial Powerplant, the Factory the Raptor and the Twister.
const LIVE: [ObjectKind; 8] = [
    ObjectKind::RadarStation,
    ObjectKind::DefenseSystem,
    ObjectKind::MissileTurret,
    ObjectKind::Factory,
    ObjectKind::AdvancedMechanics,
    ObjectKind::IndustrialPowerplant,
    ObjectKind::Raptor,
    ObjectKind::Twister,
];

/// Slice M6-GK's kinds, in the registry's order: the Factory to the mine.
fn new_kinds() -> &'static [ObjectKind] {
    &ObjectKind::ALL[ObjectKind::Factory as usize..=ObjectKind::Mine as usize]
}

/// The new kinds of `category`.
fn new_of(category: Category) -> Vec<ObjectKind> {
    (new_kinds().iter().copied())
        .filter(|kind| kind.def().category == category)
        .collect()
}

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn point(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// Player 1's Deployment Center 1 at (2, 2), Barracks 2 at (8, 2) and Field Engineer 3 on tile (2, 8), on a flat map of
/// 40 × 30 tiles, with a bank of 10 000 Space Crystals from tick 1.
fn base() -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(40, 30))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 2))
        .spawn(ObjectKind::FieldEngineer, Some(P1), SpawnAt::Point(point(2, 8)))
        .start();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 10_000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t.run_until(1);
    t
}

fn gate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

fn construct(constructor: u32, kind: ObjectKind) -> Command {
    Command::Construction {
        objects: vec![ObjectId(constructor)],
        command: ConstructionCommand::Construct(kind),
    }
}

fn train(producer: u32, kind: ObjectKind) -> Command {
    Command::Production {
        objects: vec![ObjectId(producer)],
        command: ProductionCommand::Train(kind),
    }
}

fn build(unit: u32, kind: ObjectKind, origin: TilePos) -> Command {
    Command::Order {
        units: vec![ObjectId(unit)],
        kind: OrderKind::Build {
            kind,
            origin,
            orientation: BuildingOrientation::default(),
        },
        queue: false,
    }
}

fn move_to(units: &[ObjectId], to: Vec2) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Move(Target::Point(to)),
        queue: false,
    }
}

fn spawn(kind: ObjectKind, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(P1),
        at,
        orientation: Default::default(),
    }
}

/// `factions/gdo/structures/deployment_center.md > Constructs`, held (`sc_sim::defs::PLANNED_RECIPES`): the
/// Deployment Center constructs none of the new buildings but the live ones ([`LIVE`]), its three held ones among
/// them, with the bank to pay for each (`Reject::CannotConstruct`), and still constructs a Generator. (M6-GF) Of the
/// live ones, the Factory, which [`base`]'s Barracks allows, and Advanced Mechanics and the Industrial Powerplant, each
/// refused for its own TechPrerequisite there (a Factory, Advanced Mechanics: `Reject::Requires`).
#[test]
fn the_deployment_center_constructs_none_of_them_but_the_live_ones() {
    let t = base();
    let buildings = new_of(Category::Structure);
    assert_eq!(buildings.len(), 12);
    let held = (PLANNED_RECIPES.iter()).filter(|(maker, _)| *maker == ObjectKind::DeploymentCenter);
    assert_eq!(held.clone().count(), 3);
    assert!(held.clone().all(|(_, recipe)| buildings.contains(&recipe.kind)));
    for kind in buildings.into_iter().filter(|kind| !LIVE.contains(kind)) {
        assert_eq!(
            gate(&t, construct(DC, kind)),
            Err(Reject::CannotConstruct(kind)),
            "{kind:?}"
        );
    }
    let factorys = [
        (ObjectKind::Factory, Ok(())),
        (
            ObjectKind::AdvancedMechanics,
            Err(Reject::Requires(ObjectKind::Factory)),
        ),
        (
            ObjectKind::IndustrialPowerplant,
            Err(Reject::Requires(ObjectKind::AdvancedMechanics)),
        ),
    ];
    for (kind, answer) in factorys {
        assert_eq!(gate(&t, construct(DC, kind)), answer, "{kind:?}");
    }
    assert_eq!(gate(&t, construct(DC, ObjectKind::Generator)), Ok(()));
}

/// `factions/gdo/structures/barracks.md > Produces`: the Barracks trains none of the new units but (M6-GD) the Drone
/// Operator, which requires a Career Campus (`Reject::Requires`), nor a mine (`Reject::CannotProduce`), and still trains
/// a Peacekeeper.
#[test]
fn the_barracks_trains_none_of_them_but_the_drone_operator() {
    let t = base();
    let units: Vec<ObjectKind> = (new_kinds().iter().copied())
        .filter(|kind| kind.def().category != Category::Structure)
        .collect();
    assert_eq!(units.len(), 11);
    for kind in units {
        let refused = match kind {
            ObjectKind::DroneOperator => Reject::Requires(ObjectKind::CareerCampus),
            _ => Reject::CannotProduce(kind),
        };
        assert_eq!(gate(&t, train(BARRACKS, kind)), Err(refused), "{kind:?}");
    }
    assert_eq!(gate(&t, train(BARRACKS, ObjectKind::Peacekeeper)), Ok(()));
}

/// `factions/gdo/units/field_engineer.md`: its Build Bunker and Set up First Aid Tent are held, so it builds neither,
/// nor any other new building (`Reject::NoWorkers`: no unit of the order builds it).
#[test]
fn the_field_engineer_builds_none_of_them() {
    let t = base();
    for kind in new_of(Category::Structure) {
        let order = build(FIELD_ENGINEER, kind, TilePos::new(20, 20));
        assert_eq!(gate(&t, order), Err(Reject::NoWorkers), "{kind:?}");
    }
}

/// The new buildings that will produce, spawned: the Airfield and the Supply Tower produce nothing yet
/// (`Reject::NoProducers`), not even their own units, nor (M6-GF) the Factory its held Prospector and Artillery Rig
/// (`Reject::CannotProduce`: it produces its Raptor and Twister, `tests/factory/`). (M6-GR) The Defense System
/// constructs its Missile Turret (`tests/defense_system/`).
#[test]
fn the_new_buildings_make_nothing_yet() {
    let mut t = base();
    let makers = [
        (ObjectKind::Factory, tile(14, 2)),
        (ObjectKind::Airfield, tile(19, 2)),
        (ObjectKind::SupplyTower, tile(24, 2)),
    ];
    let now = t.sim().tick() + 1;
    for (kind, at) in makers {
        t.debug_at(now, spawn(kind, at));
    }
    t.step();
    let spawned: Vec<ObjectKind> = (t.sim().events().iter())
        .filter_map(|e| match e {
            SimEvent::Spawned(id) => Some(t.sim().object(*id).unwrap().kind),
            _ => None,
        })
        .collect();
    assert_eq!(spawned, makers.map(|(kind, _)| kind));
    let ids: Vec<u32> = (4..7).collect();
    for &(maker, recipe) in PLANNED_RECIPES {
        let Some(n) = makers.iter().position(|&(kind, _)| kind == maker) else {
            continue;
        };
        let (id, kind) = (ids[n], recipe.kind);
        match maker {
            ObjectKind::Factory => assert_eq!(gate(&t, train(id, kind)), Err(Reject::CannotProduce(kind))),
            _ => assert_eq!(gate(&t, train(id, kind)), Err(Reject::NoProducers), "{kind:?}"),
        }
    }
}

/// No game start spawns a new kind: no faction's Starting Conditions hold one, and neither the playtest map nor
/// `dev_skirmish` starts with one, whoever plays.
#[test]
fn no_game_starts_with_one() {
    for &faction in Faction::ALL {
        for &kind in new_kinds() {
            assert_eq!(StartingConditions::of(faction).count(kind), 0, "{faction:?}, {kind:?}");
        }
    }
    let maps: [MapDef; 2] = [
        serde_json::from_str(include_str!("../../../maps/playtest.json")).unwrap(),
        serde_json::from_str(include_str!("../../../maps/dev_skirmish.json")).unwrap(),
    ];
    for map in maps {
        for &faction in Faction::ALL {
            let setup = GameSetup::per_slot(&map, &[faction, faction], DEFAULT_SEED);
            let scenario = Scenario {
                map: map.clone(),
                setup,
            };
            let sim = Sim::try_new(&scenario).unwrap();
            assert!(sim.objects().count() > 2, "{faction:?}");
            assert!(sim.objects().all(|o| !new_kinds().contains(&o.kind)), "{faction:?}");
        }
    }
}

/// The editor's debug spawn places every new kind, each where it fits: the buildings on their tiles, the units and the
/// mine at a point. They stand through a few hundred ticks, the units given a Move: each mobile one moves toward its
/// point, ground and air alike, the deployed Rig stays where it is, and the mine, which is no unit, takes no order.
#[test]
fn the_debug_spawn_places_each_of_them() {
    let mut t = base();
    let now = t.sim().tick() + 1;
    // The buildings in two rows of tiles, 5 apart; the units and the mine in a row of points, 3 tiles apart.
    let mut tiles = (0..12).map(|i| tile(1 + 5 * (i % 7), 9 + 5 * (i / 7)));
    let mut points = (0..11).map(|i| SpawnAt::Point(point(2 + 3 * i, 22)));
    for &kind in new_kinds() {
        let at = match kind.def().category {
            Category::Structure => tiles.next(),
            _ => points.next(),
        };
        t.debug_at(now, spawn(kind, at.expect("a place for each")));
    }
    t.step();
    let spawned: Vec<ObjectId> = (t.sim().events().iter())
        .filter_map(|e| match e {
            SimEvent::Spawned(id) => Some(*id),
            _ => None,
        })
        .collect();
    let kinds: Vec<ObjectKind> = spawned.iter().map(|id| t.sim().object(*id).unwrap().kind).collect();
    assert_eq!(kinds, new_kinds(), "every one placed");
    let units: Vec<ObjectId> = (spawned.iter().copied())
        .filter(|id| t.sim().object(*id).unwrap().def().category == Category::Unit)
        .collect();
    assert_eq!(units.len(), 10);
    let start: Vec<Vec2> = units.iter().map(|id| t.sim().object(*id).unwrap().pos).collect();
    let now = t.sim().tick() + 1;
    t.at(now, P1, move_to(&units, point(20, 27)));
    t.run(300);
    for (id, start) in units.iter().zip(start) {
        let o = t.sim().object(*id).unwrap();
        let moved = o.pos.distance(start);
        if o.kind == ObjectKind::ArtilleryRigDeployed {
            assert_eq!(o.pos, start, "the deployed Rig doesn't move");
        } else {
            assert!(moved > 64.0, "{:?} moved {moved} su", o.kind);
        }
    }
    let mine = spawned.last().copied().unwrap();
    assert_eq!(t.sim().object(mine).unwrap().kind, ObjectKind::Mine);
    assert_eq!(gate(&t, move_to(&[mine], point(5, 5))), Err(Reject::NoUnits));
}

/// Plan decision T87: Artillery Rig (Deployed) "cannot move" (`factions/gdo/units/artillery_rig.md > Modes`). Given a
/// Move, it neither drives nor turns its hull, nothing panics, and the Move ends, Failed or Idle, leaving it without
/// an order.
#[test]
fn a_deployed_rig_given_a_move_stays_where_it_is() {
    let mut t = base();
    let now = t.sim().tick() + 1;
    t.debug_at(
        now,
        spawn(ObjectKind::ArtilleryRigDeployed, SpawnAt::Point(point(20, 20))),
    );
    t.step();
    let rig = ObjectId(4);
    let (pos, facing) = {
        let o = t.sim().object(rig).unwrap();
        assert_eq!(o.kind, ObjectKind::ArtilleryRigDeployed);
        (o.pos, o.facing)
    };
    let now = t.sim().tick() + 1;
    t.at(now, P1, move_to(&[rig], point(30, 5)));
    t.run(600);
    let o = t.sim().object(rig).unwrap();
    assert_eq!((o.pos, o.facing), (pos, facing), "it doesn't move");
    assert!(o.orders.as_ref().unwrap().is_idle(), "its Move has ended");
    assert!(
        matches!(t.status(rig.0), MoveStatus::Failed | MoveStatus::Idle),
        "{:?}",
        t.status(rig.0)
    );
}
