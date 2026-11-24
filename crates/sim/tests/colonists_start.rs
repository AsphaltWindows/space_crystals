//! A Colonists player's start (slice M3-C; `factions/colonists/economy.md > Starting Conditions`): the banks, the
//! objects of the Starting Layout on `dev_skirmish` in either slot (plan decision T24), and Beacon Capacity
//! (`factions/colonists/overview.md > ColonistResources > Beacon Capacity`, `economy.md > Warp In > Beacon Capacity
//! Retention`; plan decision T20): taken when a unit's production starts, not when it is queued, so that group
//! production leaves it out (design change 0143, answering Q0041 points 1 and 2, slice M3-P; design change 0083), and
//! kept by the units and the unit in production when the capacity falls (design change 0112). Also the refined banks
//! that `SetResources` sets (T21). The Starting Conditions table is unit-tested in `scenario/starting.rs`, the caps in
//! `economy/army.rs`, the bar and the camera in the client; the waits at production start in
//! `tests/production/capacity.rs`.

use sc_sim::scenario::DEFAULT_SEED;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    ArmyCap, ArmyResource, Banks, Command, DebugCommand, Faction, GameSetup, MapDef, ObjectId, ObjectKind,
    PlayerCommand, PlayerId, ProductionCommand, Reject, SU_PER_GU, Scenario, ScenarioProblem, Sim, SimEvent, SlotId,
    SpawnAt, TilePos, Vec2, scenarios,
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

/// Plan acceptance, capacity, as design change 0143 has it ("Queuing a unit needs only its cost in resources (paid
/// when queued) and a free queue slot; it does **not** check or reserve Beacon Capacity"): with 10 available and 4
/// used, 7 Drones queue at the Robo Portal, all paid; each takes its 1 as its production starts, one at a time, so
/// that six come out and the 7th waits at 0% once they fill the 10.
#[test]
fn seven_drones_queue_at_the_robo_portal_and_the_seventh_waits_for_room() {
    let mut t = TestSim::new(dev_skirmish_as([COLONISTS, GDO]));
    let portal = find(t.sim(), P1, PORTAL);
    t.debug_at(1, crystals(P1, 1000));
    for _ in 0..7 {
        t.at(2, P1, train(&[portal]));
    }
    t.run(2);
    // `factions/colonists/structures.md > Robo Portal`: a Drone costs 50 Space Crystals.
    assert_eq!((queue(t.sim(), portal), t.sim().player(P1).unwrap().crystals), (7, 650));
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(5, 10), "the first in production");
    assert_eq!(validate(t.sim(), P1, train(&[portal])), Ok(()));
    let drones = |sim: &Sim| sim.objects().filter(|o| o.owner == Some(P1) && o.kind == DRONE).count();
    // 160 frames each, one after another.
    while drones(t.sim()) < 10 {
        assert!(t.sim().tick() < 1100, "six come out");
        t.step();
    }
    t.step();
    assert_eq!(
        (queue(t.sim(), portal), t.sim().army_cap(P1)),
        (1, beacon_capacity(10, 10))
    );
    let status = t.sim().production_status(portal).unwrap();
    assert_eq!(
        (status.progress, status.blocked),
        (0, Some(ArmyResource::BeaconCapacity))
    );
}

/// Design change 0083 as 0143 has it ("0083 (group production with limited resources) no longer considers Unit
/// Control", nor the Beacon Capacity): with room for one more Drone, a Train to two Robo Portals queues at both, the
/// idlest first, and none is refused; the idle one starts in the room, and the other's second Drone waits behind
/// its first. Two portals provide 10; eight Drones and one in production take 9.
#[test]
fn group_production_queues_past_the_capacity() {
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
    assert_eq!((queue(t.sim(), a), queue(t.sim(), b)), (2, 1), "both queue");
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(10, 10), "B started");
    let queued: Vec<ObjectId> = t
        .events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::ProductionQueued { producer, .. } if *tick == 2 => Some(*producer),
            _ => None,
        })
        .collect();
    assert_eq!(queued, [b, a], "the idlest first");
    assert!(
        !t.events()
            .iter()
            .any(|(_, e)| matches!(e, SimEvent::ProductionRefused { .. }))
    );
    // A's first comes out after its 160 frames, and its second has no room.
    t.run_until(161);
    let status = t.sim().production_status(a).unwrap();
    assert_eq!(
        (status.progress, status.blocked),
        (0, Some(ArmyResource::BeaconCapacity))
    );
}

/// `factions/colonists/overview.md > Beacon Capacity > Over capacity` (design change 0112, as 0143 amends it): when
/// buildings are lost and the capacity falls below what is used, the units stay, "units already in production
/// finish", and "no new unit starts production until there is room again": the Drones queued behind wait at 0%.
#[test]
fn units_and_the_drone_in_production_outlast_a_fall_in_capacity() {
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
    assert_eq!(
        t.sim().army_cap(P1),
        beacon_capacity(9, 5),
        "the portal's 5 left; eight Drones and one in production"
    );
    assert_eq!(queue(t.sim(), portal), 3);
    assert_eq!(validate(t.sim(), P1, train(&[portal])), Ok(()));
    // The one in production comes out after its 160 frames; the two behind it wait.
    t.run_until(400);
    let drones = t
        .sim()
        .objects()
        .filter(|o| o.owner == Some(P1) && o.kind == DRONE)
        .count();
    assert_eq!(
        (drones, queue(t.sim(), portal)),
        (9, 2),
        "the eight stayed, and one came out"
    );
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(9, 5));
    let status = t.sim().production_status(portal).unwrap();
    assert_eq!(
        (status.progress, status.blocked),
        (0, Some(ArmyResource::BeaconCapacity))
    );
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
