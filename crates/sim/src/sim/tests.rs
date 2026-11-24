use super::*;
use crate::combat::{AttackPhase, AttackState};
use crate::command::{Command, DebugCommand};
use crate::ids::SlotId;
use crate::map::{Tile, TilePresetId};
use crate::object::Rotation;
use crate::player::Faction;
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
        placement: None,
        resource: None,
        orders: Some(Orders::default()),
        mover: Some(Mover::default()),
        attack: Some(AttackState::default()),
    };
    sim.objects.insert(id, unit);
    id
}

/// Gives unit `id` a current order, a queued one, a path and an attack being aimed, so every field of its orders,
/// mover and attack has a value.
fn make_busy(sim: &mut Sim, id: u32) {
    let order = Order {
        kind: OrderKind::Move(Target::Point(Vec2::new(40.0, 50.0))),
        group: OrderGroup(3),
        slot: Some(Vec2::new(41.0, 51.0)),
    };
    let o = orders(sim, id);
    o.give(order.clone(), false);
    o.current = Some(crate::orders::ActiveOrder {
        order,
        since: 2,
        state: crate::orders::Behaviour::Plain,
    });
    let pos = obj(sim, id).pos;
    let m = mover(sim, id);
    m.go_to(crate::mover::MoveGoal::Point(Vec2::new(41.0, 51.0)), None, 2);
    m.follow(vec![pos, Vec2::new(41.0, 51.0)], Vec2::new(41.0, 51.0), 0);
    m.avoid_tiles.push(TilePos::new(1, 2));
    obj(sim, id).attack = Some(AttackState {
        target: Some(ObjectId(2)),
        phase: Some(AttackPhase::Aiming { target: ObjectId(2) }),
        elapsed: 1,
    });
}

fn orders(sim: &mut Sim, id: u32) -> &mut Orders {
    obj(sim, id).orders.as_mut().expect("a unit")
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

fn placement(sim: &mut Sim) -> &mut Placement {
    obj(sim, 1).placement.as_mut().expect("patch has a placement")
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

/// A named change to one field.
type Mutation = (&'static str, fn(&mut Sim));

#[test]
fn state_hash_covers_every_field() {
    let mut base = small_sim();
    insert_unit(&mut base, Vec2::new(100.0, 100.0)); // id 3
    make_busy(&mut base, 3);
    let mutations: Vec<Mutation> = vec![
        ("tick", |s| s.tick += 1),
        ("map", |s| {
            s.map.set_tile(TilePos::new(0, 0), Tile::new(TilePresetId::WATER))
        }),
        ("player id", |s| s.players[0].id = PlayerId(9)),
        ("player name", |s| s.players[0].name.push('!')),
        ("player faction", |s| s.players[0].faction = Faction::Cults),
        ("player removed", |s| {
            s.players.pop();
        }),
        ("object removed", |s| {
            s.objects.remove(&ObjectId(3));
        }),
        ("object id", |s| obj(s, 1).id = ObjectId(7)),
        ("kind", |s| obj(s, 1).kind = ObjectKind::SupplyDeliveryStation),
        ("owner", |s| obj(s, 1).owner = Some(PlayerId(2))),
        ("pos x", |s| obj(s, 3).pos.x += 1.0),
        ("pos y", |s| obj(s, 3).pos.y += 1.0),
        ("prev_pos x", |s| obj(s, 3).prev_pos.x += 1.0),
        ("prev_pos y", |s| obj(s, 3).prev_pos.y += 1.0),
        ("facing", |s| obj(s, 3).facing += 0.1),
        ("prev_facing", |s| obj(s, 3).prev_facing += 0.1),
        ("hp value", |s| obj(s, 3).hp = Some(39.5)),
        ("hp none", |s| obj(s, 3).hp = None),
        ("placement none", |s| obj(s, 1).placement = None),
        ("placement origin x", |s| placement(s).origin.x += 1),
        ("placement origin y", |s| placement(s).origin.y += 1),
        ("placement w", |s| placement(s).w += 1),
        ("placement h", |s| placement(s).h += 1),
        ("placement rotation", |s| placement(s).rotation = Rotation::R90),
        ("placement flip_h", |s| placement(s).flip_h = true),
        ("placement flip_v", |s| placement(s).flip_v = true),
        ("resource none", |s| obj(s, 1).resource = None),
        ("patch remaining", |s| {
            obj(s, 1).resource = Some(ResourceState::CrystalPatch { remaining: 1 });
        }),
        ("delivery_size", |s| *station(s).0 += 1),
        ("delivery_interval", |s| *station(s).1 += 1),
        ("current_supplies", |s| *station(s).2 += 1),
        ("next_delivery_in value", |s| *station(s).3 = Some(5)),
        ("next_delivery_in none", |s| *station(s).3 = None),
        ("next_id", |s| s.next_id += 1),
        ("rng", |s| {
            s.rng.next_u32();
        }),
        ("config debug", |s| s.config.debug = !s.config.debug),
        ("next_group", |s| s.next_group += 1),
        ("auto_targeting", |s| s.auto_targeting = !s.auto_targeting),
        ("nav_version", |s| s.nav_version += 1),
        ("orders none", |s| obj(s, 3).orders = None),
        ("queue", |s| {
            orders(s, 3).queue.pop_front();
        }),
        ("queued kind", |s| orders(s, 3).queue[0].kind = OrderKind::Hold),
        ("queued group", |s| orders(s, 3).queue[0].group = OrderGroup(4)),
        ("queued slot", |s| orders(s, 3).queue[0].slot = None),
        ("move target", |s| {
            orders(s, 3).queue[0].kind = OrderKind::Move(Target::Object(ObjectId(1)));
        }),
        ("current none", |s| orders(s, 3).current = None),
        ("current since", |s| orders(s, 3).current.as_mut().unwrap().since += 1),
        ("current order", |s| {
            orders(s, 3).current.as_mut().unwrap().order.kind = OrderKind::Stop;
        }),
        ("mover none", |s| obj(s, 3).mover = None),
        ("goal", |s| mover(s, 3).goal = crate::mover::MoveGoal::None),
        ("status", |s| mover(s, 3).status = crate::mover::MoveStatus::Failed),
        ("locomotion", |s| {
            mover(s, 3).locomotion = crate::mover::Locomotion::Moving
        }),
        ("orientation", |s| {
            mover(s, 3).orientation = crate::mover::Orientation::Turning(Vec2::ONE);
        }),
        ("velocity", |s| mover(s, 3).velocity.x += 1.0),
        ("speed_factor", |s| mover(s, 3).speed_factor = 0.5),
        ("path", |s| mover(s, 3).path[1].y += 1.0),
        ("next", |s| mover(s, 3).next += 1),
        ("path_state", |s| mover(s, 3).path_state = crate::mover::PathState::None),
        ("path group", |s| {
            mover(s, 3).path_state = crate::mover::PathState::Wanted {
                since: 2,
                group: Some(crate::nav::PathGroup {
                    id: 1,
                    anchor: Vec2::ZERO,
                }),
            };
        }),
        ("mover nav_version", |s| mover(s, 3).nav_version += 1),
        ("planned_target", |s| mover(s, 3).planned_target.x += 1.0),
        ("repaths_left", |s| mover(s, 3).repaths_left -= 1),
        ("last_repath", |s| mover(s, 3).last_repath += 1),
        ("avoid_tiles", |s| mover(s, 3).avoid_tiles.clear()),
        ("best_remaining", |s| mover(s, 3).best_remaining = 7.0),
        ("last_progress", |s| mover(s, 3).last_progress += 1),
        ("blocked_ticks", |s| mover(s, 3).blocked_ticks += 1),
        ("avoid_side", |s| mover(s, 3).avoid_side = crate::mover::Side::Left),
        ("avoid_until", |s| mover(s, 3).avoid_until += 1),
        ("sidestep", |s| {
            mover(s, 3).sidestep = Some(crate::mover::Sidestep::new(Vec2::X, 24.0, 4.0, 0, None));
        }),
        ("attack none", |s| obj(s, 3).attack = None),
        ("attack target", |s| attack(s, 3).target = Some(ObjectId(1))),
        ("attack target none", |s| attack(s, 3).target = None),
        ("attack phase none", |s| attack(s, 3).phase = None),
        ("attack phase", |s| {
            attack(s, 3).phase = Some(AttackPhase::Firing {
                target: Some(ObjectId(2)),
            });
        }),
        ("aimed target", |s| {
            attack(s, 3).phase = Some(AttackPhase::Aiming { target: ObjectId(1) });
        }),
        ("fired target", |s| {
            attack(s, 3).phase = Some(AttackPhase::Firing { target: None });
        }),
        ("cooldown", |s| attack(s, 3).phase = Some(AttackPhase::Cooldown)),
        ("reloading", |s| attack(s, 3).phase = Some(AttackPhase::Reloading)),
        ("attack elapsed", |s| attack(s, 3).elapsed += 1),
    ];
    for (name, mutate) in mutations {
        let mut sim = base.clone();
        mutate(&mut sim);
        assert_ne!(sim, base, "{name}: the mutation changed nothing");
        assert_ne!(sim.state_hash(), base.state_hash(), "{name}");
    }
}

#[test]
fn state_hash_ignores_events_only() {
    let base = small_sim();
    let mut with_event = base.clone();
    with_event.events.push(SimEvent::Removed(ObjectId(1)));
    assert_eq!(with_event.state_hash(), base.state_hash());
    assert_eq!(small_sim().state_hash(), base.state_hash());
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

/// `docs/movement.md > 4`, `systems/controls/behaviors.md > MovingToLocation > Failure`: a unit that gets no path
/// (here `PathFail::StartBlocked`: put on the station directly, which the gate and validation never allow) gives up
/// with `MoveFailed`, stays where it is, and its order completes.
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
    // What phase 5f would have seen: the units were inserted by hand.
    sim.vision.update(&sim.map, sim.objects.values());
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
