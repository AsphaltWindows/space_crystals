//! `Sim::change_kind` (plan decision T15): the object keeps its id, its capabilities are rebuilt from the new kind,
//! the state hash changes and a `SimEvent::KindChanged` says so. The changes the design names: a Legionnaire's weapon
//! (`factions/colonists/units.md > Weapon Conversion`), a Drone into a Beacon and back (`economy.md > Beacon
//! Deployment`), a Beacon into a building and back (`> Warp In`).

use glam::Vec2;

use super::*;
use crate::command::{Command, PlayerCommand, TickInputs};
use crate::construction::ConstructionCommand;
use crate::defs::Footprint;
use crate::ids::PlayerId;
use crate::mover::MoveStatus;
use crate::nav::NavClass;
use crate::orders::{OrderKind, Target};
use crate::production::ProductionCommand;
use crate::scenario::SpawnAt;
use crate::testing::ScenarioBuilder;
use crate::testing::invariants::Invariants;
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// A flat 24 × 12 map with `spawns`, ids from 1 in their order, and the units' auto-targeting off.
fn sim(spawns: &[(ObjectKind, PlayerId, SpawnAt)]) -> Sim {
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(24); 12].join("\n"));
    for &(kind, owner, at) in spawns {
        builder = builder.spawn(kind, Some(owner), at);
    }
    let mut sim = Sim::new(&builder.build());
    sim.auto_targeting = false;
    sim
}

/// One tick with `commands`, checking the invariants after it.
fn step(sim: &mut Sim, invariants: &mut Invariants, commands: Vec<PlayerCommand>) {
    let inputs = TickInputs { commands };
    sim.step(&inputs);
    if let Err(violation) = invariants.check(sim, &inputs) {
        panic!("{violation:?}");
    }
}

fn command(player: PlayerId, command: Command) -> PlayerCommand {
    PlayerCommand { player, command }
}

/// A `kind` structure placed unturned with its top-left tile at (`x`, `y`).
fn placement(kind: ObjectKind, x: i32, y: i32) -> Placement {
    Placement::new(TilePos::new(x, y), kind.def().footprint)
}

fn object(sim: &Sim, id: u32) -> &Object {
    &sim.objects[&ObjectId(id)]
}

/// `> Weapon Conversion`: a Sabre Legionnaire that becomes a Rifle Legionnaire is the same unit, where it was, with its
/// HP, its orders and its move (design change 0150); its weapon is the Rifle's, ready (design change 0152), aimed at
/// what its order engages. So it carries on with the order and fires its rifle.
#[test]
fn a_legionnaire_changes_its_weapon_and_carries_on_with_its_order() {
    let mut sim = sim(&[
        (
            ObjectKind::SabreLegionnaire,
            P1,
            SpawnAt::Point(Vec2::new(200.0, 400.0)),
        ),
        (ObjectKind::Peacekeeper, P2, SpawnAt::Point(Vec2::new(520.0, 400.0))),
    ]);
    let mut invariants = Invariants::new(&sim);
    let (sabre, pk) = (ObjectId(1), ObjectId(2));
    let attack = Command::Order {
        units: vec![sabre],
        kind: OrderKind::Attack(pk),
        queue: false,
    };
    step(&mut sim, &mut invariants, vec![command(P1, attack)]);
    for _ in 0..3 {
        step(&mut sim, &mut invariants, Vec::new());
    }
    sim.objects.get_mut(&sabre).unwrap().hp = Some(42.0);
    let before = object(&sim, 1).clone();
    assert_eq!(before.mover.as_ref().unwrap().status, MoveStatus::Moving, "on its way");
    let hash = sim.state_hash();
    sim.events.clear();

    assert!(sim.change_kind(sabre, ObjectKind::RifleLegionnaire, None));
    let after = object(&sim, 1);
    assert_eq!(after.kind, ObjectKind::RifleLegionnaire);
    assert_eq!(
        (after.id, after.owner, after.pos, after.prev_pos, after.facing, after.hp),
        (sabre, Some(P1), before.pos, before.prev_pos, before.facing, Some(42.0))
    );
    assert_eq!((&after.orders, &after.mover), (&before.orders, &before.mover));
    assert_eq!(
        after.attack,
        Some(AttackState {
            target: Some(pk),
            phase: None,
            elapsed: 0,
            pending: None,
        })
    );
    assert_eq!(after.placement, None);
    assert_eq!(
        sim.events,
        [SimEvent::KindChanged {
            id: sabre,
            from: ObjectKind::SabreLegionnaire,
            to: ObjectKind::RifleLegionnaire,
        }]
    );
    assert_ne!(sim.state_hash(), hash);

    // Within the rifle's 4 grid units, it stops and shoots.
    let mut shots = Vec::new();
    for _ in 0..16 {
        step(&mut sim, &mut invariants, Vec::new());
        shots.extend(sim.events().iter().filter_map(|e| match e {
            SimEvent::Hit {
                attacker,
                target,
                damage,
                shield_damage: 0.0,
            } => Some((*attacker, *target, *damage)),
            _ => None,
        }));
    }
    // The Rifle's Damage 10 less the Peacekeeper's PointArmor 1.
    assert_eq!(shots.first(), Some(&(sabre, pk, 9.0)), "the Rifle Legionnaire fired");
}

/// `factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit types` (design change 0173): "They share everything
/// except what the shield changes", the weapon included, so a Peacekeeper that becomes a Peacekeeper (Shielded) and
/// back keeps its attack as it was, the reload under way and a deferred ability with it, unlike a Legionnaire's new
/// weapon; and it keeps its shield where it was, and its speed factor, which `riot_shield::settle` sets after a toggle.
#[test]
fn the_shields_two_types_keep_the_weapon_and_the_shield_as_they_were() {
    let mut sim = sim(&[(ObjectKind::Peacekeeper, P1, SpawnAt::Point(Vec2::new(300.0, 300.0)))]);
    let pk = ObjectId(1);
    let reloading = AttackState {
        target: Some(ObjectId(7)),
        phase: Some(crate::combat::AttackPhase::Reloading),
        elapsed: 5,
        pending: Some(crate::command::Ability::RiotShield { raise: true }),
    };
    let o = sim.objects.get_mut(&pk).unwrap();
    o.attack = Some(reloading.clone());
    o.riot_shield = Some(crate::riot_shield::RiotShield { up: 2 });
    o.mover.as_mut().unwrap().speed_factor = 0.5;
    for kind in [ObjectKind::PeacekeeperShielded, ObjectKind::Peacekeeper] {
        assert!(sim.change_kind(pk, kind, None));
        let o = object(&sim, 1);
        assert_eq!(o.kind, kind);
        assert_eq!(o.attack.as_ref(), Some(&reloading), "{kind:?}");
        assert_eq!(
            o.riot_shield,
            Some(crate::riot_shield::RiotShield { up: 2 }),
            "{kind:?}"
        );
        assert_eq!(o.mover.as_ref().unwrap().speed_factor, 0.5, "{kind:?}");
    }
}

/// `economy.md > Beacon Deployment`: a Drone that becomes a Beacon stands on its footprint, at its centre, and keeps
/// its HP (the deployment makes it follow the interpolated maximum, design change 0110); its unit capabilities go. The
/// footprint blocks the ground from the next phase 5b on. `> Cancel Deployment`: back to a Drone, where the Beacon
/// stood, with a unit's capabilities, idle, its HP at most the Drone's MaxHP.
#[test]
fn a_drone_becomes_a_beacon_in_place_and_back() {
    let mut sim = sim(&[(ObjectKind::Drone, P1, SpawnAt::Point(Vec2::new(300.0, 300.0)))]);
    let mut invariants = Invariants::new(&sim);
    let drone = ObjectId(1);
    step(&mut sim, &mut invariants, Vec::new());
    sim.objects.get_mut(&drone).unwrap().facing = 1.0;
    let hash = sim.state_hash();
    sim.events.clear();

    let footprint = placement(ObjectKind::Beacon, 4, 4);
    assert!(sim.change_kind(drone, ObjectKind::Beacon, Some(footprint)));
    let beacon = object(&sim, 1);
    let centre = Vec2::new(320.0, 320.0);
    assert_eq!(
        (beacon.id, beacon.kind, beacon.owner, beacon.placement),
        (drone, ObjectKind::Beacon, Some(P1), Some(footprint))
    );
    assert_eq!((beacon.pos, beacon.prev_pos, beacon.facing), (centre, centre, 1.0));
    assert_eq!(beacon.hp, Some(20.0), "the Drone's HP, below the Beacon's MaxHP");
    assert!(beacon.orders.is_none() && beacon.mover.is_none() && beacon.attack.is_none());
    assert!(beacon.production.is_none() && beacon.constructor.is_none());
    assert_eq!(
        sim.events,
        [SimEvent::KindChanged {
            id: drone,
            from: ObjectKind::Drone,
            to: ObjectKind::Beacon,
        }]
    );
    assert_ne!(sim.state_hash(), hash);
    step(&mut sim, &mut invariants, Vec::new());
    let grid = sim.nav().grid(NavClass::GroundRugged, 1).unwrap();
    assert!(
        footprint.tiles().all(|tile| grid.is_blocked(tile)),
        "the footprint blocks"
    );

    // Cancelled at full HP: a Drone at full HP.
    sim.objects.get_mut(&drone).unwrap().hp = Some(120.0);
    assert!(sim.change_kind(drone, ObjectKind::Drone, None));
    let back = object(&sim, 1);
    assert_eq!(
        (back.kind, back.pos, back.placement, back.hp),
        (ObjectKind::Drone, centre, None, Some(20.0))
    );
    assert_eq!(
        (&back.orders, &back.mover),
        (&Some(Orders::default()), &Some(Mover::default()))
    );
    // A unit again: it takes orders, and the footprint no longer blocks.
    let go = Command::Order {
        units: vec![drone],
        kind: OrderKind::Move(Target::Point(Vec2::new(1200.0, 320.0))),
        queue: false,
    };
    step(&mut sim, &mut invariants, vec![command(P1, go)]);
    for _ in 0..20 {
        step(&mut sim, &mut invariants, Vec::new());
    }
    assert!(object(&sim, 1).pos.x > 400.0, "it moved off: {}", object(&sim, 1).pos);
    let grid = sim.nav().grid(NavClass::GroundRugged, 1).unwrap();
    assert!(footprint.tiles().all(|tile| !grid.is_blocked(tile)));
}

/// `economy.md > Warp In`: a Beacon that becomes a Robo Portal has the portal's footprint, centred where it is now,
/// and an empty BuildQueue. `> Cancel Warp In` back to a Beacon: what was paid for is never lost silently. A queue
/// the Beacon has no place for is refunded as if cancelled, before the change, and its HP is at most the Beacon's.
/// A construction (a GDO constructor's; no transformation changes one) is refunded the same way.
#[test]
fn paid_state_the_new_kind_has_no_place_for_is_refunded() {
    let mut sim = sim(&[
        (ObjectKind::Beacon, P1, SpawnAt::Tile(TilePos::new(3, 3))),
        (ObjectKind::DeploymentCenter, P2, SpawnAt::Tile(TilePos::new(14, 4))),
    ]);
    let mut invariants = Invariants::new(&sim);
    let (beacon, dc) = (ObjectId(1), ObjectId(2));
    step(&mut sim, &mut invariants, Vec::new());

    let portal = placement(ObjectKind::RoboPortal, 2, 2);
    assert!(sim.change_kind(beacon, ObjectKind::RoboPortal, Some(portal)));
    let o = object(&sim, 1);
    assert_eq!(
        (o.kind, o.pos, o.hp),
        (ObjectKind::RoboPortal, Vec2::new(256.0, 256.0), Some(120.0))
    );
    assert_eq!(o.production, Some(Production::default()));
    // Two Drones queued and paid for (50 Space Crystals each, of the 100 the bank starts with).
    let train = Command::Production {
        objects: vec![beacon],
        command: ProductionCommand::Train(ObjectKind::Drone),
    };
    step(
        &mut sim,
        &mut invariants,
        vec![command(P1, train.clone()), command(P1, train)],
    );
    let queue = &object(&sim, 1).production.as_ref().unwrap().queue;
    assert_eq!((queue.len(), sim.player(P1).unwrap().crystals), (2, 0));
    sim.objects.get_mut(&beacon).unwrap().hp = Some(500.0);
    sim.events.clear();

    assert!(sim.change_kind(beacon, ObjectKind::Beacon, Some(placement(ObjectKind::Beacon, 3, 3))));
    let cancelled = SimEvent::ProductionCancelled {
        producer: beacon,
        kind: ObjectKind::Drone,
        refund: 50,
    };
    let changed = SimEvent::KindChanged {
        id: beacon,
        from: ObjectKind::RoboPortal,
        to: ObjectKind::Beacon,
    };
    assert_eq!(sim.events, [cancelled.clone(), cancelled, changed]);
    assert_eq!(sim.player(P1).unwrap().crystals, 100);
    let o = object(&sim, 1);
    assert_eq!(
        (o.production.as_ref(), o.hp, o.pos),
        (None, Some(120.0), Vec2::new(256.0, 256.0))
    );

    // A Deployment Center constructing a Generator (100 Space Crystals) that became a Barracks.
    let construct = Command::Construction {
        objects: vec![dc],
        command: ConstructionCommand::Construct(ObjectKind::Generator),
    };
    step(&mut sim, &mut invariants, vec![command(P2, construct)]);
    assert_eq!(sim.player(P2).unwrap().crystals, 0);
    sim.events.clear();
    assert!(sim.change_kind(dc, ObjectKind::Barracks, Some(placement(ObjectKind::Barracks, 14, 4))));
    assert_eq!(sim.player(P2).unwrap().crystals, 100);
    assert!(matches!(
        sim.events[..],
        [
            SimEvent::ConstructionCancelled { refund: 100, .. },
            SimEvent::KindChanged { .. }
        ]
    ));
    assert_eq!(object(&sim, 2).constructor, None);
    assert_eq!(object(&sim, 2).production, Some(Production::default()));
    step(&mut sim, &mut invariants, Vec::new());
    assert_eq!(crate::testing::dangling_references(&sim), []);
}

/// Nothing changes for an object that isn't there.
#[test]
fn a_missing_object_changes_nothing() {
    let mut sim = sim(&[(ObjectKind::Drone, P1, SpawnAt::Point(Vec2::new(300.0, 300.0)))]);
    sim.remove_objects(&[ObjectId(1)]);
    let hash = sim.state_hash();
    sim.events.clear();
    assert!(!sim.change_kind(
        ObjectId(1),
        ObjectKind::Beacon,
        Some(placement(ObjectKind::Beacon, 4, 4))
    ));
    assert!(!sim.change_kind(ObjectId(7), ObjectKind::RifleLegionnaire, None));
    assert_eq!((sim.state_hash(), sim.events.len()), (hash, 0));
}

/// Whatever it was before, a changed object has the capabilities a new object of its kind has, and stands where it
/// was, or at its placement's centre. A Riot Shield that goes takes its slowdown with it.
#[test]
fn a_changed_object_has_the_capabilities_of_a_new_one() {
    let mut sim = sim(&[(ObjectKind::Peacekeeper, P1, SpawnAt::Point(Vec2::new(300.0, 300.0)))]);
    let pk = ObjectId(1);
    let o = sim.objects.get_mut(&pk).unwrap();
    o.riot_shield = Some(crate::riot_shield::RiotShield { up: 3 });
    o.mover.as_mut().unwrap().speed_factor = 0.5;
    let mut pos = Vec2::new(300.0, 300.0);
    for (kind, placement) in [
        (ObjectKind::Drone, None),
        (ObjectKind::BioPortal, Some(placement(ObjectKind::BioPortal, 8, 2))),
        (ObjectKind::SabreLegionnaire, None),
        (ObjectKind::EgressNode, Some(placement(ObjectKind::EgressNode, 8, 2))),
        (ObjectKind::Peacekeeper, None),
    ] {
        assert!(sim.change_kind(pk, kind, placement));
        let o = object(&sim, 1);
        let fresh = Capabilities::fresh(&BTreeMap::new(), kind, placement.as_ref());
        pos = placement.map_or(pos, |p| p.center());
        assert_eq!((o.kind, o.pos, o.placement), (kind, pos, placement));
        assert_eq!(
            matches!(kind.def().footprint, Footprint::Tiles { .. }),
            o.placement.is_some(),
            "{kind:?}"
        );
        if let Some(mover) = &o.mover {
            assert_eq!(mover.speed_factor, 1.0, "{kind:?}");
        }
        assert_eq!(
            (
                o.orders.is_some(),
                o.mover.is_some(),
                o.attack.is_some(),
                o.riot_shield.is_some()
            ),
            (
                fresh.orders.is_some(),
                fresh.mover.is_some(),
                fresh.attack.is_some(),
                fresh.riot_shield.is_some()
            ),
            "{kind:?}"
        );
        assert_eq!(
            (o.extractor, o.production.is_some(), o.constructor),
            (fresh.extractor, fresh.production.is_some(), fresh.constructor),
            "{kind:?}"
        );
    }
}
