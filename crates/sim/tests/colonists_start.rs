//! A Colonists player's start (slice M3-C; `factions/colonists/economy.md > Starting Conditions`): the banks, the
//! objects of the Starting Layout on `dev_skirmish` in either slot (plan decision T24), and Beacon Capacity
//! (`factions/colonists/overview.md > ColonistResources > Beacon Capacity`, `economy.md > Warp In > Beacon Capacity
//! Retention`; plan decision T20): reserved when a unit is queued (placeholder: Q0041 point 1), checked by the gate
//! for every producer of a group (design change 0083), and kept by the units and the queues when the capacity falls
//! (placeholder: Q0041 point 2). Also the refined banks that `SetResources` sets (T21). The Starting Conditions table
//! is unit-tested in `scenario/starting.rs`, the caps in `economy/army.rs`, the bar and the camera in the client.

use sc_sim::scenario::DEFAULT_SEED;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    ArmyCap, ArmyResource, Banks, Command, DebugCommand, Faction, GameSetup, MapDef, ObjectId, ObjectKind,
    PlayerCommand, PlayerId, ProductionCommand, Refusal, Reject, SU_PER_GU, Scenario, ScenarioProblem, Sim, SimEvent,
    SlotId, SpawnAt, TilePos, Vec2, scenarios,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DRONE: ObjectKind = ObjectKind::Drone;
const NODE: ObjectKind = ObjectKind::EgressNode;
const PORTAL: ObjectKind = ObjectKind::RoboPortal;
const GDO: Faction = Faction::Gdo;
const COLONISTS: Faction = Faction::Colonists;

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// `dev_skirmish` with player 1 in slot 1 and player 2 in slot 2, playing `factions`.
fn dev_skirmish_as(factions: [Faction; 2]) -> Scenario {
    let map = scenarios::dev_skirmish_map();
    let setup = GameSetup::per_slot(&map, &factions, DEFAULT_SEED);
    Scenario { map, setup }
}

/// `player`'s objects: kind and position, in id order.
fn objects_of(sim: &Sim, player: PlayerId) -> Vec<(ObjectKind, Vec2)> {
    sim.objects()
        .filter(|o| o.owner == Some(player))
        .map(|o| (o.kind, o.pos))
        .collect()
}

/// The first object of `kind` that `player` owns.
fn find(sim: &Sim, player: PlayerId, kind: ObjectKind) -> ObjectId {
    let found = sim.objects().find(|o| o.owner == Some(player) && o.kind == kind);
    found.expect("the player owns one").id
}

fn beacon_capacity(used: u32, available: u32) -> Option<ArmyCap> {
    Some(ArmyCap {
        resource: ArmyResource::BeaconCapacity,
        used,
        available,
    })
}

fn train(producers: &[ObjectId]) -> Command {
    Command::Production {
        objects: producers.to_vec(),
        command: ProductionCommand::Train(DRONE),
    }
}

fn validate(sim: &Sim, player: PlayerId, command: Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand { player, command })
}

fn queue(sim: &Sim, id: ObjectId) -> usize {
    sim.object(id).unwrap().production.as_ref().unwrap().queue.len()
}

fn set_banks(player: PlayerId, banks: Banks) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals: banks.crystals,
        supplies: banks.supplies,
        alloys: banks.alloys,
        essence: banks.essence,
        flux: banks.flux,
    }
}

fn crystals(player: PlayerId, crystals: u32) -> DebugCommand {
    set_banks(
        player,
        Banks {
            crystals,
            ..Banks::default()
        },
    )
}

/// Plan acceptance, start: a Colonists player starts with 100 Space Crystals and 0 Alloys, Essence and Flux, with the
/// Egress Node, the Robo Portal and the four Drones where `dev_skirmish`'s layout puts them, in either slot (the second
/// slot's layout is the first's 180° image); Beacon Capacity 4/10 (the two buildings provide 5 each, the Drones cost
/// 1 each), and the camera starts on the Robo Portal (`core/camera.md > Starting Position`). The GDO beside them
/// starts as ever, at 0/200 Unit Control.
#[test]
fn a_colonists_player_starts_with_their_starting_conditions() {
    let hundred = Banks {
        crystals: 100,
        ..Banks::default()
    };
    let scenario = dev_skirmish_as([COLONISTS, GDO]);
    let sim = Sim::new(&scenario);
    assert_eq!(sim.player(P1).unwrap().banks(), hundred);
    // Slot 1: the node on (4, 4) to (5, 5), the portal on (6, 6) to (9, 9), the Drones south-west of the node.
    let slot_1 = [
        (NODE, point(5.0, 5.0)),
        (PORTAL, point(8.0, 8.0)),
        (DRONE, point(3.5, 9.0)),
        (DRONE, point(4.5, 9.0)),
        (DRONE, point(3.5, 10.0)),
        (DRONE, point(4.5, 10.0)),
    ];
    assert_eq!(objects_of(&sim, P1), slot_1);
    assert_eq!(sim.army_cap(P1), beacon_capacity(4, 10));
    assert_eq!(scenario.primary_structure(P1), Some(point(8.0, 8.0)), "the Robo Portal");
    let gdo_cap = ArmyCap {
        resource: ArmyResource::UnitControl,
        used: 0,
        available: 200,
    };
    assert_eq!(
        (sim.player(P2).unwrap().banks(), sim.army_cap(P2)),
        (hundred, Some(gdo_cap))
    );
    assert_eq!(
        scenario.primary_structure(P2),
        Some(point(57.0, 41.0)),
        "the Deployment Center"
    );

    let scenario = dev_skirmish_as([GDO, COLONISTS]);
    let sim = Sim::new(&scenario);
    let image = |(kind, at): (ObjectKind, Vec2)| (kind, point(64.0, 48.0) - at);
    assert_eq!(objects_of(&sim, P2), slot_1.map(image));
    assert_eq!(
        (sim.player(P2).unwrap().banks(), sim.army_cap(P2)),
        (hundred, beacon_capacity(4, 10))
    );
    assert_eq!(scenario.primary_structure(P2), Some(point(56.0, 40.0)));
}

/// The M3-C brief: `dev_skirmish`'s starting Drones don't block the Egress Node's sides, where the gatherers will
/// drop off, nor any side of the Robo Portal, the doors its Drones will come out of (question Q0041 point 3): each
/// Drone stays at least 1 grid unit clear of both buildings, in both slots.
#[test]
fn the_starting_drones_keep_clear_of_the_node_and_the_portal() {
    let sim = Sim::new(&dev_skirmish_as([COLONISTS, COLONISTS]));
    for player in [P1, P2] {
        let owned = || sim.objects().filter(move |o| o.owner == Some(player));
        let buildings: Vec<(Vec2, Vec2)> = owned()
            .filter_map(|o| {
                let p = o.placement?;
                let min = Vec2::new(p.origin.x as f32, p.origin.y as f32) * SU_PER_GU;
                Some((min, min + Vec2::new(p.w as f32, p.h as f32) * SU_PER_GU))
            })
            .collect();
        assert_eq!(buildings.len(), 2);
        for drone in owned().filter(|o| o.kind == DRONE) {
            let half = Vec2::splat(18.0); // `factions/colonists/units.md > Drone > Silhouette - 36x36`
            for &(min, max) in &buildings {
                let gap = (min - (drone.pos + half)).max((drone.pos - half) - max).max(Vec2::ZERO);
                assert!(
                    gap.length() >= SU_PER_GU,
                    "{player:?}'s Drone {:?} is {gap} from a building",
                    drone.id
                );
            }
        }
    }
}

/// Plan acceptance, validation (plan decision T24): `dev_skirmish` validates with the GDO and the Colonists in any
/// slot combination, no layout is outdated, and its file is the built-in map. An incomplete Colonists layout lacks
/// its kinds (`ScenarioProblem::LayoutMissing`): no game starts on the map, a GDO one included, and the layout is
/// flagged as outdated (it holds objects but not the Starting Conditions).
#[test]
fn dev_skirmish_validates_with_either_faction_in_either_slot() {
    for factions in [[GDO, GDO], [GDO, COLONISTS], [COLONISTS, GDO], [COLONISTS, COLONISTS]] {
        let scenario = dev_skirmish_as(factions);
        assert_eq!(scenario.validate(), [], "{factions:?}");
        assert!(Sim::try_new(&scenario).is_ok(), "{factions:?}");
    }
    assert_eq!(scenarios::dev_skirmish_map().outdated_layouts(), []);
    let file: MapDef = serde_json::from_str(include_str!("../../../maps/dev_skirmish.json")).unwrap();
    assert_eq!(
        file,
        scenarios::dev_skirmish_map(),
        "maps/dev_skirmish.json is the built-in map"
    );

    let mut map = scenarios::dev_skirmish_map();
    let layout = map.slots[0].layouts.get_mut(COLONISTS);
    layout.remove(0); // the Egress Node
    layout.pop(); // a Drone
    map.regenerate();
    let lacks = |slot, kind| ScenarioProblem::LayoutMissing {
        slot: SlotId(slot),
        faction: COLONISTS,
        kind,
        missing: 1,
    };
    let expected = [lacks(1, NODE), lacks(1, DRONE), lacks(2, NODE), lacks(2, DRONE)];
    assert_eq!(map.validate(), expected);
    assert_eq!(map.outdated_layouts(), [(SlotId(1), COLONISTS), (SlotId(2), COLONISTS)]);
    assert_eq!(Sim::try_new(&Scenario::new(map)).err(), Some(expected.to_vec()));
}

/// Plan acceptance, capacity (placeholder: Q0041 point 1, as design change 0082 for the GDO): with 10 available and
/// 4 used, 6 Drones queue at the Robo Portal, paid and reserved when queued, and the 7th is refused although the
/// bank (set by debug) holds plenty; the gate's refusal changes nothing. A produced Drone takes over the capacity
/// its queue entry held.
#[test]
fn six_drones_queue_at_the_robo_portal_and_the_seventh_is_refused() {
    let mut t = TestSim::new(dev_skirmish_as([COLONISTS, GDO]));
    let portal = find(t.sim(), P1, PORTAL);
    t.debug_at(1, crystals(P1, 1000));
    for _ in 0..6 {
        t.at(2, P1, train(&[portal]));
    }
    t.run(2);
    // `factions/colonists/structures.md > Robo Portal`: a Drone costs 50 Space Crystals.
    assert_eq!((queue(t.sim(), portal), t.sim().player(P1).unwrap().crystals), (6, 700));
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(10, 10));
    assert_eq!(validate(t.sim(), P1, train(&[portal])), Err(Reject::UnitControlCap));
    t.at(3, P1, train(&[portal]));
    t.run(1);
    assert_eq!((queue(t.sim(), portal), t.sim().player(P1).unwrap().crystals), (6, 700));
    // The first Drone comes out after its 160 frames: 5 Drones and 5 queued.
    t.run_until(170);
    let drones = t
        .sim()
        .objects()
        .filter(|o| o.owner == Some(P1) && o.kind == DRONE)
        .count();
    assert_eq!((drones, queue(t.sim(), portal)), (5, 5));
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(10, 10));
}

/// Design change 0083 ("This applies to every faction and every cost"): with room for one more Drone under the
/// Beacon Capacity, the idlest of two Robo Portals takes it, and the other is refused for the cap. Two portals
/// provide 10; eight Drones and one queued take 9.
#[test]
fn group_production_serves_the_least_work_first_up_to_the_capacity() {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, COLONISTS)
        .spawn(PORTAL, Some(P1), tile(1, 1)) // 1: (1, 1) to (4, 4)
        .spawn(PORTAL, Some(P1), tile(8, 1)); // 2: (8, 1) to (11, 4)
    for i in 0..8 {
        builder = builder.spawn(DRONE, Some(P1), SpawnAt::Point(point(1.5 + i as f32, 10.5)));
    }
    let mut t = builder.start();
    let (a, b) = (ObjectId(1), ObjectId(2));
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(8, 10));
    t.debug_at(1, crystals(P1, 5000));
    t.at(1, P1, train(&[a]));
    t.at(2, P1, train(&[a, b]));
    t.run(2);
    assert_eq!(
        (queue(t.sim(), a), queue(t.sim(), b)),
        (1, 1),
        "B, the idle one, took the last"
    );
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(10, 10));
    let refused = SimEvent::ProductionRefused {
        producer: a,
        kind: DRONE,
        reason: Refusal::UnitControl,
    };
    assert!(t.events().contains(&(2, refused)));
}

/// Placeholder: Q0041 point 2: when buildings are lost and the capacity falls below what is used, the units stay and
/// new units are refused; the units already queued keep their reservation and complete.
#[test]
fn units_and_queues_outlast_a_fall_in_capacity() {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
        .faction(P1, COLONISTS)
        .spawn(PORTAL, Some(P1), tile(8, 6)) // 1: (8, 6) to (11, 9)
        .spawn(NODE, Some(P1), tile(1, 1)) // 2
        .spawn(ObjectKind::Beacon, Some(P1), tile(14, 1)); // 3
    for i in 0..8 {
        builder = builder.spawn(DRONE, Some(P1), SpawnAt::Point(point(1.5 + i as f32, 14.5)));
    }
    let mut t = builder.start();
    let portal = ObjectId(1);
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(8, 15));
    t.debug_at(1, crystals(P1, 1000));
    for _ in 0..3 {
        t.at(2, P1, train(&[portal]));
    }
    t.debug_at(3, DebugCommand::Remove { id: ObjectId(2) });
    t.debug_at(3, DebugCommand::Remove { id: ObjectId(3) });
    t.run(3);
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(11, 5), "the portal's 5 left");
    assert_eq!(queue(t.sim(), portal), 3);
    assert_eq!(validate(t.sim(), P1, train(&[portal])), Err(Reject::UnitControlCap));
    // 160 frames each: the three come out.
    while queue(t.sim(), portal) > 0 {
        assert!(t.sim().tick() < 1000, "the queue completes");
        t.step();
    }
    let drones = t
        .sim()
        .objects()
        .filter(|o| o.owner == Some(P1) && o.kind == DRONE)
        .count();
    assert_eq!(drones, 11, "the eight stayed");
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(11, 5));
}

/// Plan decision T21: `SetResources` sets the refined banks too, any player's, and they are part of the state's hash;
/// a command written before they existed (without them) sets them to 0.
#[test]
fn set_resources_sets_the_refined_banks() {
    let banks = Banks {
        crystals: 10,
        supplies: 20,
        alloys: 30,
        essence: 40,
        flux: 50,
    };
    let run = |banks| {
        let mut t = ScenarioBuilder::new(&vec![".".repeat(8); 8].join("\n"))
            .faction(P2, COLONISTS)
            .start();
        t.debug_at(1, set_banks(P2, banks));
        t.run(1);
        (t.sim().player(P2).unwrap().banks(), t.sim().state_hash())
    };
    let (set, hash) = run(banks);
    assert_eq!(set, banks);
    let (_, other) = run(Banks { flux: 51, ..banks });
    assert_ne!(hash, other, "Flux is hashed");
    let old: DebugCommand =
        serde_json::from_str(r#"{"SetResources": {"player": 2, "crystals": 5, "supplies": 1}}"#).unwrap();
    assert_eq!(
        old,
        set_banks(
            P2,
            Banks {
                crystals: 5,
                supplies: 1,
                ..Banks::default()
            }
        )
    );
}
