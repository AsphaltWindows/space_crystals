//! The worker's state and the worker kinds (`super`).

use super::*;

fn hash_of(worker: &Worker) -> u64 {
    let mut h = StateHasher::new();
    worker.hash(&mut h);
    h.finish()
}

/// Every field of a worker is state: each one changes the hash.
#[test]
fn the_hash_covers_every_field() {
    let worker = Worker {
        load: 8,
        drop_off: Some(ObjectId(2)),
        resource: Resource::Crystals,
    };
    let changed = [
        Worker { load: 7, ..worker },
        Worker {
            drop_off: Some(ObjectId(3)),
            ..worker
        },
        Worker {
            drop_off: None,
            ..worker
        },
        Worker {
            resource: Resource::Supplies,
            ..worker
        },
    ];
    for other in changed {
        assert_ne!(hash_of(&other), hash_of(&worker), "{other:?}");
    }
}

/// Slice M4-I: a worker's load of Space Crystals hashes, and is saved, as before the resource existed; one of
/// Supplies says so in its file.
#[test]
fn the_resource_shows_only_for_supplies() {
    let worker = Worker {
        load: 8,
        drop_off: Some(ObjectId(2)),
        resource: Resource::Crystals,
    };
    let mut h = StateHasher::new();
    let (load, drop_off) = (worker.load, worker.drop_off);
    load.hash(&mut h);
    drop_off.hash(&mut h);
    assert_eq!(hash_of(&worker), h.finish(), "as before the field existed");
    let json = serde_json::to_string(&worker).unwrap();
    assert!(!json.contains("resource"), "{json}");
    let supply = Worker {
        load: 1,
        resource: Resource::Supplies,
        ..worker
    };
    let json = serde_json::to_string(&supply).unwrap();
    assert_eq!(serde_json::from_str::<Worker>(&json).unwrap(), supply, "{json}");
}

/// The removal path: a worker forgets the drop-off that is gone, and keeps its load. A worker saved with the
/// patch slice M3-H kept loads as it was, without it.
#[test]
fn a_worker_forgets_what_is_gone() {
    let mut worker = Worker {
        load: 8,
        drop_off: Some(ObjectId(2)),
        resource: Resource::Crystals,
    };
    worker.forget(&|id| id == ObjectId(9));
    assert_eq!(worker.drop_off, Some(ObjectId(2)));
    worker.forget(&|id| id == ObjectId(2));
    let emptied = Worker {
        load: 8,
        ..Worker::default()
    };
    assert_eq!(worker, emptied);
    let saved: Worker = serde_json::from_str(r#"{"load":8,"drop_off":null,"patch":3}"#).unwrap();
    assert_eq!(saved, emptied);
}

/// `factions/colonists/units/drone.md > Gathering`, `factions/syndicate/units/agent.md > Gathering`: the Drone
/// and the Agent are the workers, the Egress Node and the Tunnel the drop-offs; and the Cults' Recruit and Storage
/// (slice M5-D; `factions/cults/units/recruit.md > Gathering`, whose numbers the Recruit's def test pins).
#[test]
fn the_drone_the_agent_and_the_recruit_are_the_workers() {
    use crate::defs::{DropOffChoice, LoadDef};
    for kind in ObjectKind::ALL.iter().copied() {
        let worker = matches!(kind, ObjectKind::Drone | ObjectKind::Agent | ObjectKind::Recruit);
        assert_eq!(worker_def(kind).is_some(), worker, "{kind:?}");
        let drop_off = matches!(kind, ObjectKind::EgressNode | ObjectKind::Tunnel | ObjectKind::Storage);
        assert_eq!(is_drop_off(kind), drop_off, "{kind:?}");
    }
    let def = worker_def(ObjectKind::Drone).expect("a worker");
    let crystals = LoadDef {
        carry_capacity: 8,
        pick_up_ticks: 32,
        drop_off_ticks: 2,
        pick_up_reach: 8.0,
        drop_off_reach: 8.0,
    };
    assert_eq!((def.crystals, def.supplies), (crystals, None));
    assert_eq!(
        def.search_radius, 6.0,
        "> Patch Occupancy > Search radius (design change 0123)"
    );
    assert!(def.pass_through && def.drop_off == DropOffChoice::Assigned && !def.shares_patches);
    // `factions/syndicate/units/agent.md > Gathering > Space Crystals`: 50 a load, 64 frames of mining (design
    // change 0294), 64 of unloading, reaches of 8 su; `factions/syndicate/units/agent.md > Supplies`: 1 a trip, 48 frames each way, the
    // same reaches (design change 0292 point 1); design change 0255: shared patches, no Set Drop-off; 0106: "Drones
    // only" pass through each other.
    let def = worker_def(ObjectKind::Agent).expect("a worker");
    let crystals = LoadDef {
        carry_capacity: 50,
        pick_up_ticks: 64,
        drop_off_ticks: 64,
        pick_up_reach: 8.0,
        drop_off_reach: 8.0,
    };
    let supplies = LoadDef {
        carry_capacity: 1,
        pick_up_ticks: 48,
        drop_off_ticks: 48,
        pick_up_reach: 8.0,
        drop_off_reach: 8.0,
    };
    assert_eq!((def.crystals, def.supplies), (crystals, Some(supplies)));
    assert_eq!(
        (def.load(Resource::Crystals), def.load(Resource::Supplies)),
        (Some(crystals), Some(supplies))
    );
    assert!(def.shares_patches && def.drop_off == DropOffChoice::FirstReturn && !def.pass_through);
}
