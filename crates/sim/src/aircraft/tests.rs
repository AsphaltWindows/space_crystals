use glam::Vec2;

use super::*;
use crate::hash::StateHasher;
use crate::ids::PlayerId;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::units::TilePos;

/// An Airfield (1) and two Hornets (2, 3) of player 1, and a Peacekeeper (4), as they come into a game.
fn game() -> Sim {
    let builder = crate::testing::ScenarioBuilder::new(&"..........\n".repeat(8))
        .spawn(
            ObjectKind::Airfield,
            Some(PlayerId(1)),
            SpawnAt::Tile(TilePos::new(1, 1)),
        )
        .spawn(
            ObjectKind::Hornet,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(400.0, 100.0)),
        )
        .spawn(
            ObjectKind::Hornet,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(500.0, 100.0)),
        )
        .spawn(
            ObjectKind::Peacekeeper,
            Some(PlayerId(1)),
            SpawnAt::Point(Vec2::new(500.0, 300.0)),
        );
    Sim::new(&builder.build())
}

/// "Ammo: **5 missiles**": a Hornet comes into the game with its type's full ammo, unassigned (a spawn's: no Airfield
/// built it); a type without ammo has no aircraft state, and its weapon never runs out.
#[test]
fn an_aircraft_starts_full_and_unassigned() {
    let sim = game();
    let hornet = sim.object(ObjectId(2)).expect("spawned");
    let full = AircraftState { ammo: 5, base: None };
    assert_eq!(hornet.aircraft, Some(full));
    assert!(armed(hornet) && may_fire(hornet) && base_of(hornet).is_none());
    let peacekeeper = sim.object(ObjectId(4)).expect("spawned");
    assert_eq!((peacekeeper.aircraft, fresh(ObjectKind::Peacekeeper)), (None, None));
    assert!(armed(peacekeeper));
}

/// Each shot spends a round; the last leaves it unarmed, and says so once; none is spent past it, nor of a unit
/// without ammo.
#[test]
fn a_round_a_shot_until_none_is_left() {
    let sim = game();
    let mut hornet = sim.object(ObjectId(2)).expect("spawned").clone();
    let spent: Vec<bool> = (0..6).map(|_| spend(&mut hornet)).collect();
    assert_eq!(spent, [false, false, false, false, true, false]);
    assert_eq!(hornet.aircraft.map(|state| state.ammo), Some(0));
    assert!(!armed(&hornet));
    let mut peacekeeper = sim.object(ObjectId(4)).expect("spawned").clone();
    assert!(!spend(&mut peacekeeper) && peacekeeper.aircraft.is_none());
}

/// `factions/gdo/structures/airfield.md > Aircraft Support`: "its assignments plus the aircraft already in its
/// production queue", against its capacity of 4: the aircraft assigned to it and the aircraft its queue trains count, the others don't; an object that is no
/// Airfield has no room.
#[test]
fn an_airfields_room_counts_its_assignments_and_its_queue() {
    let mut objects: BTreeMap<ObjectId, Object> = game().objects().map(|o| (o.id, o.clone())).collect();
    let airfield = ObjectId(1);
    assert_eq!((taken(&objects, airfield), room(&objects, airfield)), (0, 4));
    if let Some(state) = objects.get_mut(&ObjectId(2)).and_then(|o| o.aircraft.as_mut()) {
        state.base = Some(airfield);
    }
    let production = objects
        .get_mut(&airfield)
        .and_then(|o| o.production.as_mut())
        .expect("a producer");
    production.queue = vec![
        QueueEntry::Unit(ObjectKind::Hornet),
        QueueEntry::Unit(ObjectKind::Hornet),
    ];
    assert_eq!((taken(&objects, airfield), room(&objects, airfield)), (3, 1));
    assert!(is_airfield(&objects[&airfield]) && !is_airfield(&objects[&ObjectId(2)]));
    assert_eq!(room(&objects, ObjectId(4)), 0);
    assert!(takes_place(&QueueEntry::Unit(ObjectKind::Hornet)));
    assert!(!takes_place(&QueueEntry::Unit(ObjectKind::Peacekeeper)) && !takes_place(&QueueEntry::Release));
}

/// Hashed field by field: its ammo and its base each count, and (M6-GH2) its landing only while it has one, under a
/// sub-tag: an aircraft in the air hashes as it did before the field existed, its ammo and its base alone.
#[test]
fn its_ammo_and_its_base_are_hashed() {
    let hash = |state: AircraftState| {
        let mut h = StateHasher::new();
        state.hash(&mut h);
        h.finish()
    };
    let full = AircraftState { ammo: 5, base: None };
    assert_ne!(hash(full), hash(AircraftState { ammo: 4, ..full }));
    assert_ne!(
        hash(full),
        hash(AircraftState {
            base: Some(ObjectId(1)),
            ..full
        })
    );
    let mut before = StateHasher::new();
    full.ammo.hash(&mut before);
    full.base.hash(&mut before);
    assert_eq!(hash(full), before.finish(), "in the air, the bytes of slice M6-GH1");
    // Its landing is its object's (`Object::landing`), hashed with its state: landed, and rearming, it hashes apart.
    let hornet = game().object(ObjectId(2)).expect("spawned").clone();
    let object_hash = |landing| {
        let mut o = hornet.clone();
        o.landing = landing;
        let mut h = StateHasher::new();
        o.hash(&mut h);
        h.finish()
    };
    let landing = crate::landing::down_on(ObjectId(1), 16);
    let rearming = crate::landing::Landing {
        rearm: Some(crate::progress::Progress::new(32)),
        ..landing
    };
    assert_ne!(object_hash(None), object_hash(Some(landing)));
    assert_ne!(object_hash(Some(landing)), object_hash(Some(rearming)));
}

/// (M6-GH2) On a slot, landing or down, an aircraft with ammo holds fire (`may_fire`), though it is armed: it takes
/// an Attack, which lifts it off (`crate::landing`).
#[test]
fn on_a_slot_it_holds_fire() {
    let sim = game();
    let mut hornet = sim.object(ObjectId(2)).expect("spawned").clone();
    hornet.landing = Some(crate::landing::Landing {
        at: crate::landing::Spot::Structure(ObjectId(1)),
        descent: crate::progress::Progress::new(16),
        rearm: None,
    });
    assert!(armed(&hornet) && !may_fire(&hornet));
    hornet.landing = None;
    assert!(may_fire(&hornet));
}
