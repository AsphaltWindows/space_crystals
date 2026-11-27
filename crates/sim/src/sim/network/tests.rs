//! The Tunnel Network in a game, where a test needs the sim's inside or a type no game has (`defs::test_defs`): an
//! air unit, a worker with a load, a unit taken off the map and put back, the held units at a defeat, (M6-SV) a
//! vehicle of the Subterranean Transport's 72 × 48 su coming out of the exit path.

use glam::Vec2;

use crate::command::{Command, DebugCommand, PlayerCommand, TunnelCommand};
use crate::defs::test_defs::TestDef;
use crate::defs::{Footprint, ObjectDef, ObjectKind, UnitBase};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::math;
use crate::network::{self, NoTransit, Paths, Place};
use crate::object::Rotation;
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::production::ProductionCommand;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::shape::Shape;
use crate::sim::Reject;
use crate::testing::{ScenarioBuilder, TestSim, dangling_references};
use crate::units::{SU_PER_GU, TilePos};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TUNNEL: ObjectId = ObjectId(1);

/// Player 1's Tunnel (1) on tiles (12, 10) to (15, 13) of a 30 × 22 map: its entry path's spot is (866, 640).
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&"..............................\n".repeat(22))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
}

fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

/// The Director's decision for slice M4-C, and the plan's acceptance: "An air test unit is refused at every tier,
/// since M4 has no air doors." The Syndicate has no air unit yet, so a Peacekeeper on a Hover Craft base stands in.
#[test]
fn an_air_unit_is_refused_at_every_tier() {
    let _flyer = TestDef::unit(ObjectKind::Peacekeeper, |unit| unit.base = UnitBase::HoverCraft);
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(866.0, 560.0)) // 2: the flyer
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(700.0, 560.0)); // 3
    t.run_until(1);
    let (flyer, guard) = (ObjectId(2), ObjectId(3));
    let enter = |units: &[ObjectId]| Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(TUNNEL),
        queue: false,
    };
    let eject = Command::Tunnel {
        tunnel: TUNNEL,
        command: TunnelCommand::Eject {
            kind: ObjectKind::Peacekeeper,
        },
    };
    for tier in 1..=3 {
        t.debug_at(t.sim().tick() + 1, DebugCommand::SetTier { id: TUNNEL, tier });
        t.step();
        let tunnel = t.sim().object(TUNNEL).unwrap();
        // At Tier 3 the table lists Air; the missing air doors refuse it all the same.
        let lists_air = crate::tunnel::tier_def(tunnel).unwrap().admits(UnitBase::HoverCraft);
        assert_eq!(lists_air, tier == 3);
        assert_eq!(
            network::admits(tunnel, ObjectKind::Peacekeeper),
            Err(NoTransit::Air),
            "Tier {tier}"
        );
        assert_eq!(network::admits(tunnel, ObjectKind::Guard), Ok(()));
        let refused = Reject::NoTransit {
            tunnel: TUNNEL,
            kind: None,
        };
        assert_eq!(validate(&t, enter(&[flyer])), Err(refused), "Tier {tier}");
        assert_eq!(validate(&t, enter(&[flyer, guard])), Ok(()), "the Guard takes it");
        let refused = Reject::NoTransit {
            tunnel: TUNNEL,
            kind: Some(ObjectKind::Peacekeeper),
        };
        assert_eq!(validate(&t, eject.clone()), Err(refused), "Tier {tier}");
        // Its default command on the Tunnel is a Move to it, never Enter.
        let default = t.sim().default_order(flyer, Target::Object(TUNNEL));
        assert_eq!(default, Some(OrderKind::Move(Target::Object(TUNNEL))), "Tier {tier}");
    }
    // Ordered in with the Guard at Tier 3, the flyer stays where it is.
    t.at(t.sim().tick() + 1, P1, enter(&[flyer, guard]));
    t.run(120);
    let inside: Vec<ObjectId> = t.sim().network(P1).map(|held| held.object.id).collect();
    assert_eq!(inside, [guard]);
    assert_eq!(t.pos(2), Vec2::new(866.0, 560.0));
}

/// Design change 0249: "An Agent keeps its load." Plan decision T30: a unit taken off the map and put back is the
/// same object, with its id, HP, load and modes, and without its orders, its move or its attack. A worker with a load
/// stands in for the Agent, whose gathering is slice M4-D's.
#[test]
fn a_unit_keeps_its_id_its_hp_and_its_load_through_the_network() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Drone, P1, Vec2::new(500.0, 300.0)) // 2
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(300.0, 300.0)) // 3: follows the Drone
        .order_at(2, P1, &[ObjectId(2)], OrderKind::Patrol(Vec2::new(900.0, 300.0)), false)
        .order_at(
            2,
            P1,
            &[ObjectId(3)],
            OrderKind::Move(Target::Object(ObjectId(2))),
            false,
        );
    t.run_until(10);
    let drone = ObjectId(2);
    let mut sim = t.sim().clone();
    let o = sim.objects.get_mut(&drone).unwrap();
    o.worker.as_mut().unwrap().load = 5;
    o.hp = Some(33.0);
    assert!(!o.orders.as_ref().unwrap().is_idle());
    assert!(sim.take_off_map(drone, Place::Network, true));
    assert!(
        !sim.take_off_map(drone, Place::Network, true),
        "it is off the map already"
    );
    assert!(sim.object(drone).is_none());
    let held = sim.held_unit(drone).unwrap();
    assert_eq!((held.since, held.entered, held.place), (10, true, Place::Network));
    let inside = &held.object;
    assert_eq!(
        (inside.id, inside.kind, inside.owner),
        (drone, ObjectKind::Drone, Some(P1))
    );
    assert_eq!((inside.hp, inside.worker.unwrap().load), (Some(33.0), 5));
    assert!(inside.orders.as_ref().unwrap().is_idle());
    assert_eq!(inside.mover, Some(crate::mover::Mover::default()));
    // The Guard's Follow ended, as for a removal; nothing names the Drone.
    assert!(sim.object(ObjectId(3)).unwrap().orders.as_ref().unwrap().is_idle());
    assert_eq!(dangling_references(&sim), []);
    // Back on the map, elsewhere: the same object, standing.
    let at = Vec2::new(1500.0, 900.0);
    assert!(sim.put_on_map(drone, at, Vec2::new(0.0, 1.0)));
    assert!(!sim.put_on_map(drone, at, Vec2::X), "it is on the map already");
    assert!(sim.held_unit(drone).is_none());
    let back = sim.object(drone).unwrap();
    assert_eq!((back.pos, back.prev_pos), (at, at));
    assert_eq!((back.hp, back.worker.unwrap().load), (Some(33.0), 5));
    assert!(back.orders.as_ref().unwrap().is_idle());
}

/// Plan decision T30's defeat-time cleanup: when a player is defeated, the units inside their network leave the
/// game (their last Tunnel is gone, and nothing could bring them out); those on the map become neutral.
#[test]
fn a_defeated_players_network_goes_with_them() {
    let mut t = base().start();
    t.without_auto_targeting()
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(866.0, 560.0)) // 2: goes in
        .spawn_unit_at(1, ObjectKind::Guard, P1, Vec2::new(300.0, 300.0)) // 3: stays out
        .spawn_unit_at(1, ObjectKind::Guard, P2, Vec2::new(300.0, 900.0)); // 4
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: ObjectKind::Tunnel,
            owner: Some(P2),
            at: SpawnAt::Tile(TilePos::new(22, 2)),
            orientation: Default::default(),
        },
    ); // 5
    t.order_at(2, P1, &[ObjectId(2)], OrderKind::Enter(TUNNEL), false);
    t.run_until(40);
    assert_eq!(t.sim().held().count(), 1);
    t.debug_at(41, DebugCommand::Remove { id: TUNNEL });
    t.run_until(41);
    assert!(t.sim().player(P1).unwrap().is_defeated());
    assert_eq!(t.sim().held().count(), 0);
    let removed: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(tick, e)| match e {
            SimEvent::Removed(id) if *tick == 41 => Some(*id),
            _ => None,
        })
        .collect();
    assert_eq!(removed, [TUNNEL, ObjectId(2)]);
    assert_eq!(t.sim().object(ObjectId(3)).unwrap().owner, None, "neutral, on the map");
    assert_eq!(dangling_references(t.sim()), []);
}

/// Plan decision T100 ("SV tests a 72x48 box at the exit spot"), for slice M6-ST1's Subterranean Transport
/// (`factions/syndicate/units/subterranean_transport.md`: "Silhouette 72x48"), which only a test can give a vehicle
/// that comes out of the exit path today: the Assault Bike with a 72 × 48 Silhouette, trained at a Vehicle Assembly
/// into the network and ejected at the Tier 2 Tunnel, turned each of the four ways. It comes out on the exit path's
/// spot, facing out of Side A, its box turned with it, 72 su deep out of the side and 48 su along it, its box against
/// the wall, a sixty-fourth of a su clear of the Tunnel's footprint; then it drives 1 grid unit straight out, under
/// the invariants (the wheeled W1 and the no-overlap I1 and I2 among them).
#[test]
fn a_72_by_48_vehicle_comes_out_of_side_a_facing_out_its_box_turned_with_it() {
    let bike = ObjectKind::AssaultBike;
    let _transport_sized = TestDef::set(ObjectDef {
        footprint: Footprint::Silhouette { w: 72.0, h: 48.0 },
        ..bike.def().clone()
    });
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        let mut scenario = base()
            .spawn(ObjectKind::ChemicalPlant, Some(P1), SpawnAt::Tile(TilePos::new(16, 14))) // 2
            .spawn(ObjectKind::VehicleAssembly, Some(P1), SpawnAt::Tile(TilePos::new(9, 7))) // 3
            .build();
        scenario.map.spawns[0].orientation = BuildingOrientation {
            rotation,
            flip_h: false,
            flip_v: false,
        };
        let mut t = TestSim::new(scenario);
        t.without_auto_targeting();
        t.debug_at(1, DebugCommand::SetTier { id: TUNNEL, tier: 2 });
        let banks = DebugCommand::SetResources {
            player: P1,
            crystals: 500,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        };
        t.debug_at(1, banks);
        let train = Command::Production {
            objects: vec![ObjectId(3)],
            command: ProductionCommand::Train(bike),
        };
        t.at(2, P1, train);
        t.run_until(241);
        let unit = ObjectId(4);
        assert!(
            t.sim().held_unit(unit).is_some(),
            "{rotation:?}: trained into the network"
        );
        let eject = Command::Tunnel {
            tunnel: TUNNEL,
            command: TunnelCommand::Eject { kind: bike },
        };
        assert_eq!(validate(&t, eject.clone()), Ok(()));
        t.at(242, P1, eject).run_until(242);
        let tunnel = t.sim().object(TUNNEL).unwrap();
        let paths = Paths::of_object(tunnel).unwrap();
        let footprint = Shape::footprint(tunnel.placement.as_ref().unwrap());
        let o = t.sim().object(unit).expect("out on the map");
        let out = paths.exit + paths.outward * (36.0 + 1.0 / 64.0);
        assert!(o.pos.distance(out) < 1e-3, "{rotation:?}: at {}, not {out}", o.pos);
        assert!(
            math::facing_vector(o.facing).distance(paths.outward) < 1e-6,
            "{rotation:?}: facing out"
        );
        let shape = Shape::of(o);
        let half = shape.bound_half();
        let along = paths.outward.abs();
        let (deep, across) = (half.dot(along), half.dot(Vec2::ONE - along));
        assert!(
            (deep - 36.0).abs() < 1e-3 && (across - 24.0).abs() < 1e-3,
            "{rotation:?}: {half}"
        );
        assert!(
            !shape.overlaps(&footprint) && shape.gap(&footprint) < 0.1,
            "{rotation:?}: against the wall"
        );
        t.run(40);
        let parked = out + paths.outward * SU_PER_GU;
        assert!(
            t.pos(unit.0).distance(parked) < 1.0,
            "{rotation:?}: 1 grid unit out, at {}",
            t.pos(unit.0)
        );
    }
}
