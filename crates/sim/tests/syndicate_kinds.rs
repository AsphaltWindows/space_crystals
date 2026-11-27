//! Slice M6-SK's kinds through the gate (plan decision T84, design change 0367): the Syndicate's six other expansions
//! and its seven other units, with the Laser Battery's and the Subterranean Transport's second types, are in the game,
//! and no player makes one, each kept out until its slice by T69's rule, but the Vehicle Assembly and the Assault Bike,
//! live since slice M6-SV (`tests/vehicle_assembly`), the Forge, the Aerospace Assembly and the Phantom, live since
//! slice M6-SD (`tests/air_doors`), and the Null Chamber, live since slice M6-SN (`tests/null_chamber`). No Tunnel
//! builds one of the two other expansions, a Tier 3 one with the bank, the place and every prerequisite included; no building trains one of the other units, the Vehicle Assembly training
//! the Assault Bike alone and the Aerospace Assembly the Phantom alone; no Tunnel ejects one, none being inside; and
//! only the debug spawn, with debug commands on, makes one, each then an object of its kind that works, a unit moving
//! when ordered to. A deployed Laser Battery, immobile by its def (plan decision T87), stays where it is when ordered
//! to move, and its move ends.
//!
//! Every test runs under the invariants of `TestSim`.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, Reject, SimConfig, SpawnAt, Target, TilePos, TunnelCommand, Vec2, gu,
};

const P1: PlayerId = PlayerId(1);

/// The two expansions still out of play, in the Tech Tree's order (`factions/syndicate/structures/README.md`): the six
/// but the Vehicle Assembly (slice M6-SV), the Forge and the Aerospace Assembly (slice M6-SD), and the Null Chamber
/// (slice M6-SN).
const EXPANSIONS: [ObjectKind; 2] = [ObjectKind::Vault, ObjectKind::OrbitalCommand];

/// The seven unit types still out of play, each unit's second type after its first
/// (`factions/syndicate/units/README.md`): the nine but the Assault Bike (slice M6-SV) and the Phantom (slice M6-SD).
const UNITS: [ObjectKind; 7] = [
    ObjectKind::Shadebringer,
    ObjectKind::LaserBattery,
    ObjectKind::LaserBatteryDeployed,
    ObjectKind::SubterraneanTransport,
    ObjectKind::SubterraneanTransportBurrowed,
    ObjectKind::Specter,
    ObjectKind::Prism,
];

/// Player 1's Tunnel in [`full_base`], on tiles (4, 8) to (7, 11), at Tier 3: its area is (0, 4) to (11, 15).
const T1: ObjectId = ObjectId(1);
/// Player 1's Hideout in [`full_base`], on tiles (8, 12) to (9, 13).
const H1: ObjectId = ObjectId(2);
/// Player 1's Vehicle Assembly in [`full_base`], a debug spawn: the Forge's prerequisite, and a producer of the
/// Assault Bike alone.
const VA: ObjectId = ObjectId(5);
/// Player 1's Aerospace Assembly in [`full_base`], a debug spawn, a producer of the Phantom alone (slice M6-SD).
const AA: ObjectId = ObjectId(7);
/// A free place in [`T1`]'s Tier 3 area for a 3 × 3 or a 2 × 2: (8, 8) to (10, 10).
const FREE: TilePos = TilePos::new(8, 8);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn spawn(kind: ObjectKind, at: SpawnAt) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(P1),
        at,
        orientation: BuildingOrientation::default(),
    }
}

/// What the gate says to player 1's `command` now (`Sim::validate`).
fn gate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// Player 1 plays the Syndicate on a flat 40 × 20 map with every prerequisite of the two expansions and of the seven
/// units' recipes, and 10 000 Space Crystals: the Tunnel [`T1`] at Tier 3 and the Hideout [`H1`], then, debug spawns
/// in its area from tick 1, which has run, an Optics Lab (3), a Chemical Plant (4), the Vehicle Assembly [`VA`], a
/// Forge (6), the Aerospace Assembly [`AA`], a Vault (8) and an Orbital Command (9). Auto-targeting is off.
fn full_base() -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), tile(4, 8))
        .spawn(ObjectKind::Hideout, Some(P1), tile(8, 12))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, DebugCommand::SetTier { id: T1, tier: 3 });
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 10_000,
        supplies: 100,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    let prerequisites = [
        (ObjectKind::OpticsLab, tile(0, 4)),
        (ObjectKind::ChemicalPlant, tile(2, 4)),
        (ObjectKind::VehicleAssembly, tile(8, 4)),
        (ObjectKind::Forge, tile(0, 12)),
        (ObjectKind::AerospaceAssembly, tile(0, 6)),
        (ObjectKind::Vault, tile(4, 4)),
        (ObjectKind::OrbitalCommand, tile(6, 4)),
    ];
    for (kind, at) in prerequisites {
        t.debug_at(1, spawn(kind, at));
    }
    t.run_until(1);
    let kinds: Vec<ObjectKind> = t.sim().objects().map(|o| o.kind).collect();
    let mut expected = vec![ObjectKind::Tunnel, ObjectKind::Hideout];
    expected.extend(prerequisites.map(|(kind, _)| kind));
    assert_eq!(kinds, expected);
    assert_eq!(t.sim().object(VA).unwrap().kind, ObjectKind::VehicleAssembly);
    assert_eq!(t.sim().object(AA).unwrap().kind, ObjectKind::AerospaceAssembly);
    t
}

/// "Expand" (`factions/syndicate/tunnel_menus.md > ExpandMenu`): no Tunnel builds one of the two, neither having an
/// expansion's tier, cost and time in its def until its slice: the gate refuses each Expand
/// ([`Reject::CannotConstruct`]) at a Tier 3 Tunnel with every prerequisite, 10 000 Space Crystals and a free place in
/// its area, where it takes the Optics Lab's; and the command, sent anyway, starts nothing.
#[test]
fn no_tunnel_builds_one_of_the_new_expansions() {
    let mut t = full_base();
    let expand = |kind: ObjectKind| Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Expand {
            kind,
            origin: FREE,
            orientation: BuildingOrientation::default(),
        },
    };
    assert_eq!(
        gate(&t, expand(ObjectKind::OpticsLab)),
        Ok(()),
        "the place, the tier and the bank"
    );
    for kind in EXPANSIONS {
        assert_eq!(gate(&t, expand(kind)), Err(Reject::CannotConstruct(kind)), "{kind:?}");
    }
    let before = t.sim().objects().count();
    for kind in EXPANSIONS {
        t.at(2, P1, expand(kind));
    }
    t.run_until(2);
    let tunnel = t.sim().object(T1).unwrap().tunnel.as_ref().unwrap();
    assert_eq!(tunnel.operation, None);
    assert_eq!(t.sim().objects().count(), before);
    assert_eq!(t.sim().player(P1).unwrap().crystals, 10_000);
}

/// Producing (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`): no building trains one of the
/// seven. Sent to the Hideout with the Vehicle Assembly and the Aerospace Assembly, with every recipe's prerequisite
/// and 10 000 Space Crystals, a Train of each is refused as no producer's recipe ([`Reject::CannotProduce`]), where
/// the Hideout takes the Guard's, the Vehicle Assembly the Assault Bike's (slice M6-SV) and the Aerospace Assembly the
/// Phantom's (slice M6-SD); sent to the Vehicle Assembly alone, as not its recipe, the Shadebringer's, the Laser
/// Battery's and the Subterranean Transport's waiting for their slices; sent to the Aerospace Assembly alone, as not
/// its recipe either, the Specter's and the Prism's waiting for slices M6-SA and M6-SP.
#[test]
fn no_building_trains_one_of_the_new_units() {
    let t = full_base();
    let train = |objects: Vec<ObjectId>, kind: ObjectKind| Command::Production {
        objects,
        command: ProductionCommand::Train(kind),
    };
    let every = vec![H1, VA, AA];
    assert_eq!(
        gate(&t, train(every.clone(), ObjectKind::Guard)),
        Ok(()),
        "the Hideout's Guard"
    );
    assert_eq!(
        gate(&t, train(every.clone(), ObjectKind::AssaultBike)),
        Ok(()),
        "the Vehicle Assembly's Assault Bike"
    );
    assert_eq!(
        gate(&t, train(every.clone(), ObjectKind::Phantom)),
        Ok(()),
        "the Aerospace Assembly's Phantom"
    );
    for kind in UNITS {
        assert_eq!(
            gate(&t, train(every.clone(), kind)),
            Err(Reject::CannotProduce(kind)),
            "{kind:?}"
        );
        assert_eq!(
            gate(&t, train(vec![VA], kind)),
            Err(Reject::CannotProduce(kind)),
            "{kind:?}"
        );
        assert_eq!(
            gate(&t, train(vec![AA], kind)),
            Err(Reject::CannotProduce(kind)),
            "{kind:?}"
        );
    }
}

/// "Eject" (`factions/syndicate/tunnel_menus.md > EjectMenu`): no Tunnel ejects one of the seven, none being in the
/// network. At the Tier 3 Tunnel, which lets vehicles through and (slice M6-SD) aircraft by its air doors, each Eject
/// is refused as none inside ([`Reject::NotInNetwork`]: its tile's count is 0), the Specter's and the Prism's too.
#[test]
fn no_tunnel_ejects_one_of_the_new_units() {
    let t = full_base();
    for kind in UNITS {
        let eject = Command::Tunnel {
            tunnel: T1,
            command: TunnelCommand::Eject { kind },
        };
        assert_eq!(gate(&t, eject), Err(Reject::NotInNetwork(kind)), "{kind:?}");
    }
}

/// Where [`only_the_debug_spawn_makes_one`] spawns each of the nine on a flat 64 × 16 map: an expansion on the
/// tiles of row 1, 5 tiles apart, a unit at the centre of a tile of row 9, 4 tiles apart.
fn spawn_at(i: usize, kind: ObjectKind) -> SpawnAt {
    let i = i as i32;
    if kind.def().unit.is_some() {
        SpawnAt::Point(TilePos::new(2 + 4 * (i - EXPANSIONS.len() as i32), 9).center())
    } else {
        tile(1 + 5 * i, 1)
    }
}

/// The editor's debug spawn makes each of the nine (`DebugCommand::Spawn`), and nothing else does: with debug
/// commands off, the gate refuses each ([`Reject::DebugDisabled`]); on, each comes into the game, player 1's, of its
/// kind at its full MaxHP. Each unit but the deployed Laser Battery then moves when ordered to: 200 frames on, it is
/// nearer its goal, 4 grid units south, by a grid unit at least, the Prism's slow drift included.
#[test]
fn only_the_debug_spawn_makes_one() {
    let kinds: Vec<ObjectKind> = EXPANSIONS.into_iter().chain(UNITS).collect();
    let scenario = || {
        ScenarioBuilder::new(&flat(64, 16))
            .faction(P1, Faction::Syndicate)
            .build()
    };
    let off = TestSim::with_config(scenario(), SimConfig::default());
    let mut t = TestSim::with_config(scenario(), SimConfig::DEBUG);
    t.without_auto_targeting();
    for (i, &kind) in kinds.iter().enumerate() {
        let command = Command::Debug(spawn(kind, spawn_at(i, kind)));
        assert_eq!(gate(&off, command.clone()), Err(Reject::DebugDisabled), "{kind:?}");
        assert_eq!(gate(&t, command.clone()), Ok(()), "{kind:?}");
        t.at(1, P1, command);
    }
    t.run_until(1);
    let spawned: Vec<(ObjectKind, Option<PlayerId>, Option<f32>)> =
        t.sim().objects().map(|o| (o.kind, o.owner, o.hp)).collect();
    let expected: Vec<(ObjectKind, Option<PlayerId>, Option<f32>)> =
        kinds.iter().map(|&kind| (kind, Some(P1), kind.def().max_hp)).collect();
    assert_eq!(spawned, expected);

    let mobile: Vec<(ObjectId, Vec2)> = (t.sim().objects())
        .filter(|o| o.mover.is_some() && o.kind != ObjectKind::LaserBatteryDeployed)
        .map(|o| (o.id, o.pos + Vec2::new(0.0, gu(4.0))))
        .collect();
    assert_eq!(mobile.len(), 6);
    for &(id, goal) in &mobile {
        t.order_at(2, P1, &[id], OrderKind::Move(Target::Point(goal)), false);
    }
    t.run_until(2);
    let before: Vec<f32> = mobile.iter().map(|&(id, goal)| t.pos(id.0).distance(goal)).collect();
    t.run(200);
    for (&(id, goal), before) in mobile.iter().zip(before) {
        let kind = t.sim().object(id).unwrap().kind;
        let after = t.pos(id.0).distance(goal);
        assert!(
            after <= before - gu(1.0),
            "{kind:?}: {before} su from its goal, then {after}"
        );
    }
}

/// Plan decision T87's "immobile by its def" (`factions/syndicate/units/laser_battery.md > Deploy (ability)`:
/// "**Deployed**: **cannot move**"): a deployed Laser Battery, the debug spawn's, ordered to move 4 grid units east,
/// stays where it is, every frame, and its move ends, failed or idle, its Move order done, with no panic and the
/// invariants holding throughout. (M6-GA) The gate refuses the Move, an order the type can't carry out
/// (`sc_sim::vehicle_deploy::can_take`; `factions/gdo/units/artillery_rig.md > Modes`: "one it can't carry out (e.g.
/// **Move**, since a deployed unit can't move)"), so the order played is ignored.
#[test]
fn a_deployed_laser_battery_ordered_to_move_stays_where_it_is() {
    let start = TilePos::new(6, 8).center();
    let mut t = ScenarioBuilder::new(&flat(20, 16))
        .faction(P1, Faction::Syndicate)
        .start();
    t.without_auto_targeting();
    t.debug_at(1, spawn(ObjectKind::LaserBatteryDeployed, SpawnAt::Point(start)));
    t.run_until(1);
    let battery = ObjectId(1);
    let goal = start + Vec2::new(gu(4.0), 0.0);
    let order = OrderKind::Move(Target::Point(goal));
    assert_eq!(
        gate(
            &t,
            Command::Order {
                units: vec![battery],
                kind: order,
                queue: false
            }
        ),
        Err(Reject::CannotMove)
    );
    t.order_at(2, P1, &[battery], order, false);
    for _ in 0..400 {
        t.step();
        assert_eq!(t.pos(battery.0), start, "frame {}", t.sim().tick());
    }
    let status = t.status(battery.0);
    assert!(matches!(status, MoveStatus::Failed | MoveStatus::Idle), "{status:?}");
    let orders = t.sim().object(battery).unwrap().orders.as_ref().unwrap();
    assert!(orders.current.is_none() && orders.queue.is_empty(), "{orders:?}");
}
