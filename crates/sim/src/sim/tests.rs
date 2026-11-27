mod hash;
mod turns;

use super::*;
use crate::combat::{AttackPhase, AttackState, Projectile};
use crate::command::{Command, DebugCommand, PlayerCommand, TickInputs};
use crate::construction::Constructor;
use crate::conversion::Conversion;
use crate::ids::SlotId;
use crate::map::{Tile, TilePresetId};
use crate::mover::Mover;
use crate::object::Rotation;
use crate::orders::{Order, OrderGroup, OrderKind, Orders, Target};
use crate::player::Faction;
use crate::riot_shield::RiotShield;
use crate::scenario::{DEFAULT_DELIVERY_INTERVAL, DEFAULT_DELIVERY_SIZE, Spawn, SpawnAt, SpawnSettings};
use crate::scenarios;
use crate::units::TilePos;

fn tile_spawn(kind: ObjectKind, x: i32, y: i32) -> Spawn {
    Spawn::new(kind, None, SpawnAt::Tile(TilePos::new(x, y)))
}

/// An 8×6 map (512 × 384 su) with a patch (id 1) and a station (id 2), debug commands on.
fn small_sim() -> Sim {
    let mut scenario = scenarios::empty(8, 6);
    scenario.map.spawns = vec![
        tile_spawn(ObjectKind::CrystalPatch, 1, 1),
        tile_spawn(ObjectKind::SupplyDeliveryStation, 4, 2),
    ];
    scenario.setup.config = SimConfig::DEBUG;
    Sim::new(&scenario)
}

/// An idle Peacekeeper of player 1 at `pos`, inserted directly (no checks).
fn insert_unit(sim: &mut Sim, pos: Vec2) -> ObjectId {
    let id = ObjectId(sim.next_id);
    sim.next_id += 1;
    let unit = Object {
        id,
        kind: ObjectKind::Peacekeeper,
        owner: Some(PlayerId(1)),
        pos,
        prev_pos: pos,
        facing: 0.5,
        prev_facing: 0.25,
        hp: Some(40.0),
        shields: None,
        placement: None,
        resource: None,
        orders: Some(Orders::default()),
        mover: Some(Mover::default()),
        attack: Some(AttackState::default()),
        riot_shield: Some(RiotShield::default()),
        extractor: None,
        production: None,
        constructor: None,
        worker: None,
        transform: None,
        conversion: None,
        tunnel: None,
        construction: None,
        decay: None,
        popup: None,
        lineage: Vec::new(),
        augmentation: Default::default(),
        energy: None,
        kept_research: None,
        buffs: Vec::new(),
        stock: None,
        tether: None,
        drone: None,
        cooldown: None,
        aircraft: None,
        illusion_of: None,
    };
    sim.objects.insert(id, unit);
    // Its grid, as every unit in the game has (slice M5-Z2, `paths`).
    sim.cover_kind(ObjectKind::Peacekeeper);
    id
}

fn attack(sim: &mut Sim, id: u32) -> &mut AttackState {
    obj(sim, id).attack.as_mut().expect("a unit that attacks")
}

fn mover(sim: &mut Sim, id: u32) -> &mut Mover {
    obj(sim, id).mover.as_mut().expect("a unit")
}

fn obj(sim: &mut Sim, id: u32) -> &mut Object {
    sim.objects.get_mut(&ObjectId(id)).expect("object exists")
}

fn station(sim: &mut Sim) -> (&mut u32, &mut u32, &mut u32, &mut Option<u32>) {
    match &mut obj(sim, 2).resource {
        Some(ResourceState::SupplyStation {
            delivery_size,
            delivery_interval,
            current_supplies,
            next_delivery_in,
        }) => (delivery_size, delivery_interval, current_supplies, next_delivery_in),
        other => panic!("not a station: {other:?}"),
    }
}

fn station_supplies(sim: &Sim) -> (u32, Option<u32>) {
    match sim.object(ObjectId(2)).and_then(|o| o.resource.as_ref()) {
        Some(ResourceState::SupplyStation {
            current_supplies,
            next_delivery_in,
            ..
        }) => (*current_supplies, *next_delivery_in),
        other => panic!("not a station: {other:?}"),
    }
}

fn debug(command: DebugCommand) -> TickInputs {
    TickInputs {
        commands: vec![PlayerCommand {
            player: PlayerId(1),
            command: Command::Debug(command),
        }],
    }
}

/// The gate's ownership rule, for the commands a player gives their own objects (none exist yet: debug commands may
/// refer to anyone's objects).
#[test]
fn a_players_orders_may_only_command_their_own_objects() {
    let mut sim = small_sim();
    let unit = insert_unit(&mut sim, Vec2::new(100.0, 100.0)); // player 1's
    assert!(sim.referenced(unit, Some(PlayerId(1))).is_ok());
    assert_eq!(
        sim.referenced(unit, Some(PlayerId(2))).err(),
        Some(Reject::NotOwned(unit))
    );
    // Neutral objects belong to no player.
    let patch = ObjectId(1);
    assert_eq!(
        sim.referenced(patch, Some(PlayerId(1))).err(),
        Some(Reject::NotOwned(patch))
    );
    // Debug commands and targets refer to anyone's objects, as long as they exist.
    assert!(sim.referenced(patch, None).is_ok());
    let missing = ObjectId(42);
    assert_eq!(
        sim.referenced(missing, None).err(),
        Some(Reject::UnknownObject(missing))
    );
    assert_eq!(
        sim.referenced(missing, Some(PlayerId(1))).err(),
        Some(Reject::UnknownObject(missing))
    );
}

/// `docs/movement.md > 4`, `systems/controls/command_behaviors.md > MovingToLocation > Failure`: a unit that gets no
/// path (here `PathFail::StartBlocked`: put on the station directly, which the gate and validation never allow) gives
/// up with `MoveFailed`, stays where it is, and its order completes.
#[test]
fn a_move_without_a_path_fails_and_its_order_completes() {
    let mut sim = small_sim();
    let start = Vec2::new(320.0, 192.0); // inside the station, x 256..384, y 128..256
    let unit = insert_unit(&mut sim, start);
    let order = TickInputs {
        commands: vec![PlayerCommand {
            player: PlayerId(1),
            command: Command::Order {
                units: vec![unit],
                kind: OrderKind::Move(Target::Point(Vec2::new(100.0, 300.0))),
                queue: false,
            },
        }],
    };
    sim.step(&order);
    assert_eq!(sim.events(), [SimEvent::MoveFailed { unit }]);
    let o = sim.object(unit).unwrap();
    assert_eq!(o.pos, start);
    assert_eq!(o.mover.as_ref().unwrap().status, crate::mover::MoveStatus::Failed);
    assert!(o.orders.as_ref().unwrap().current.is_some());
    sim.step(&TickInputs::default());
    assert!(sim.object(unit).unwrap().orders.as_ref().unwrap().is_idle());
    assert!(sim.events().is_empty());
}

#[test]
fn set_position_teleports_to_finite_points_on_the_map_only() {
    let mut sim = small_sim();
    let unit = insert_unit(&mut sim, Vec2::new(200.0, 50.0));
    let target = Vec2::new(200.0, 300.0);
    sim.step(&debug(DebugCommand::SetPosition { id: unit, pos: target }));
    let o = sim.object(unit).unwrap();
    assert_eq!((o.pos, o.prev_pos), (target, target));
    let bad = [
        Vec2::new(f32::NAN, 10.0),
        Vec2::new(10.0, f32::NAN),
        Vec2::splat(f32::INFINITY),
        Vec2::new(f32::NEG_INFINITY, 10.0),
        Vec2::new(-1.0, 10.0),
        Vec2::new(512.0, 10.0),
        Vec2::new(10.0, 384.0),
    ];
    for pos in bad {
        let teleport = debug(DebugCommand::SetPosition { id: unit, pos });
        assert_eq!(sim.validate(&teleport.commands[0]), Err(Reject::OffMap), "{pos}");
        sim.step(&teleport);
        assert_eq!(sim.object(unit).unwrap().pos, target, "{pos}");
    }
    // Tile-based objects keep their placement.
    let patch = sim.object(ObjectId(1)).unwrap().pos;
    sim.step(&debug(DebugCommand::SetPosition {
        id: ObjectId(1),
        pos: target,
    }));
    assert_eq!(sim.object(ObjectId(1)).unwrap().pos, patch);
}

#[test]
fn station_countdown_restarts_when_emptied() {
    let mut sim = small_sim();
    let interval = DEFAULT_DELIVERY_INTERVAL;
    for _ in 0..interval {
        sim.step(&TickInputs::default());
    }
    assert_eq!(station_supplies(&sim), (DEFAULT_DELIVERY_SIZE, None));
    // Stand-in for a future collector emptying the station between ticks.
    *station(&mut sim).2 = 0;
    sim.step(&TickInputs::default());
    assert_eq!(station_supplies(&sim), (0, Some(interval - 1)));
    for _ in 1..interval {
        assert!(sim.events().is_empty());
        sim.step(&TickInputs::default());
    }
    assert_eq!(station_supplies(&sim), (DEFAULT_DELIVERY_SIZE, None));
    let delivered = SimEvent::SuppliesDelivered {
        station: ObjectId(2),
        amount: DEFAULT_DELIVERY_SIZE,
    };
    assert_eq!(sim.events(), [delivered]);
}

#[test]
fn a_stocked_station_waits_until_emptied() {
    let mut scenario = scenarios::empty(8, 6);
    let mut stocked = tile_spawn(ObjectKind::SupplyDeliveryStation, 4, 2);
    stocked.settings = Some(SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 6,
        delivery_size: 3,
        delivery_interval: 100,
    });
    scenario.map.spawns = vec![tile_spawn(ObjectKind::CrystalPatch, 1, 1), stocked];
    let mut sim = Sim::new(&scenario);
    for _ in 0..500 {
        sim.step(&TickInputs::default());
        assert_eq!(station_supplies(&sim), (6, None));
    }
    // Stand-in for a future collector emptying the station between ticks: only now does the countdown start.
    *station(&mut sim).2 = 0;
    for _ in 0..99 {
        sim.step(&TickInputs::default());
        assert!(sim.events().is_empty());
    }
    assert_eq!(station_supplies(&sim), (0, Some(1)));
    sim.step(&TickInputs::default());
    assert_eq!(station_supplies(&sim), (3, None));
}

#[test]
fn spawns_are_ignored_once_ids_run_out() {
    let mut sim = small_sim();
    sim.next_id = u32::MAX;
    let spawn = debug(DebugCommand::Spawn {
        kind: ObjectKind::CrystalPatch,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(0, 0)),
        orientation: Default::default(),
    });
    assert_eq!(sim.validate(&spawn.commands[0]), Err(Reject::OutOfIds));
    sim.step(&spawn);
    let ids: Vec<ObjectId> = sim.objects.keys().copied().collect();
    assert_eq!(ids, [ObjectId(1), ObjectId(2)]);
    assert_eq!(sim.next_id, u32::MAX);
    assert!(sim.events().is_empty());
}

#[test]
fn try_new_and_new_agree_on_valid_scenarios() {
    let scenario = scenarios::dev_skirmish();
    assert_eq!(Sim::try_new(&scenario), Ok(Sim::new(&scenario)));
}

#[test]
#[should_panic(expected = "invalid scenario \"empty\": spawn 0 (CrystalPatch): resources can't have an owner")]
fn new_panics_listing_the_problems() {
    let mut scenario = scenarios::empty(8, 6);
    let mut patch = tile_spawn(ObjectKind::CrystalPatch, 0, 0);
    patch.owner = Some(SlotId(1));
    scenario.map.spawns.push(patch);
    Sim::new(&scenario);
}

/// JSON keeps every finite `f32` bit for bit, so a loaded state hashes the same. Random
/// bit patterns cover subnormals, huge values and -0.0.
#[test]
fn json_round_trip_keeps_every_float_bit() {
    let mut sim = small_sim();
    let mut rng = Rng::from_seed(11);
    let mut random_finite = || loop {
        let v = f32::from_bits(rng.next_u32());
        if v.is_finite() {
            return v;
        }
    };
    for _ in 0..500 {
        let id = insert_unit(&mut sim, Vec2::new(random_finite(), random_finite()));
        let o = obj(&mut sim, id.0);
        o.prev_pos = Vec2::new(random_finite(), random_finite());
        (o.facing, o.prev_facing, o.hp) = (random_finite(), -0.0, Some(random_finite()));
    }
    // What phase 5f and the end of the tick would have seen: the units were inserted by hand.
    sim.vision.update(&sim.map, sim.objects.values(), sim.tick);
    sim.vision.remember(sim.tick, |id| sim.objects.get(&id));
    let loaded: Sim = serde_json::from_str(&serde_json::to_string(&sim).unwrap()).unwrap();
    assert_eq!(loaded.state_hash(), sim.state_hash());
}

#[test]
fn loading_rejects_broken_object_tables() {
    let sim = small_sim();
    let json = serde_json::to_value(&sim).unwrap();

    let mut duplicate = json.clone();
    let objects = duplicate["objects"].as_array_mut().unwrap();
    objects.push(objects[0].clone());
    let e = serde_json::from_value::<Sim>(duplicate).unwrap_err().to_string();
    assert_eq!(e, "object id 1 appears more than once");

    let mut stale_next_id = json.clone();
    stale_next_id["next_id"] = 2.into();
    let e = serde_json::from_value::<Sim>(stale_next_id).unwrap_err().to_string();
    assert_eq!(e, "object id 2 is not below next_id 2");

    assert_eq!(serde_json::from_value::<Sim>(json).unwrap(), sim);
}
