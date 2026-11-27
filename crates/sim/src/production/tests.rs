//! The production state's rules (`crate::production`), without a game.

use super::*;
use crate::defs::ExitRule;
use crate::defs::Recipe;
use crate::sides::SideLetter;
const QUEUE_LIMIT: usize = crate::tuning::production::BUILTIN.queue_limit;
const RELEASE_FRAMES: u32 = crate::tuning::production::BUILTIN.release_frames;

const PK: QueueEntry = QueueEntry::Unit(ObjectKind::Peacekeeper);

const BARRACKS: ProductionDef = ProductionDef {
    exits: ExitRule::Side(SideLetter::B),
    recipes: &[Recipe {
        kind: ObjectKind::Peacekeeper,
        cost: 50,
        frames: 80,
        requires: &[],
        supplies: 0,
    }],
    recruitment: None,
    augment: &[],
    default_rally: None,
};

fn queued(n: usize) -> Production {
    Production {
        queue: vec![PK; n],
        ..Production::default()
    }
}

/// `factions/gdo/structures/barracks.md > Produces`: 80 frames at full power; at the ratio of 30/51 exactly 136
/// ticks (plan decision T6).
#[test]
fn a_unit_takes_its_frames_at_full_power_and_longer_at_a_ratio() {
    let ticks = |speed: u32| {
        let mut p = queued(1);
        (1..).find(|_| p.advance(&BARRACKS, speed)).unwrap()
    };
    assert_eq!(ticks(RATIO_ONE), 80);
    assert_eq!(ticks(38_551), 136, "30/51, rounded up");
    assert_eq!(ticks(RATIO_ONE / 2), 160);
}

/// Design change 0085: a finished unit stays at 100% until it is released; the next starts from nothing.
#[test]
fn a_finished_unit_waits_at_full_progress() {
    let mut p = queued(2);
    for _ in 0..200 {
        p.advance(&BARRACKS, RATIO_ONE);
    }
    let status = p.status(&BARRACKS, None).unwrap();
    assert_eq!(
        (status.progress, status.total, status.waiting),
        (80 << 16, 80 << 16, true)
    );
    assert_eq!(p.in_production(), Some(ObjectKind::Peacekeeper), "it still counts");
    assert_eq!(p.release(), Some((PK, Vec::new())));
    assert_eq!((p.queue.len(), p.progress), (1, 0));
    assert!(!p.status(&BARRACKS, None).unwrap().waiting);
    assert_eq!(queued(0).status(&BARRACKS, None), None);
}

/// "X: Cancel Production" takes the last entry; the unit in production last of all, with its progress.
#[test]
fn cancelling_takes_the_last_entry() {
    let mut p = queued(2);
    p.advance(&BARRACKS, RATIO_ONE);
    assert_eq!(p.cancel_last(), Some((PK, Vec::new())));
    assert_eq!(
        (p.queue.len(), p.progress),
        (1, RATIO_ONE),
        "the unit in production goes on"
    );
    assert_eq!(p.cancel_last(), Some((PK, Vec::new())));
    assert_eq!((p.queue.len(), p.progress), (0, 0));
    assert_eq!(p.cancel_last(), None);
}

/// Design change 0143: the unit at the front is in production, and takes its army cost, once the building has
/// begun progress on it: not while it is queued, nor at 0%, whether it is about to start or waits for room.
#[test]
fn a_unit_is_in_production_once_progress_has_begun() {
    let mut p = queued(3);
    assert_eq!(p.in_production(), None, "queued, not started");
    p.blocked = true;
    assert_eq!(p.in_production(), None, "waiting at 0%");
    p.blocked = false;
    p.advance(&BARRACKS, 0);
    assert_eq!(p.in_production(), None, "no progress at a power ratio of 0");
    p.advance(&BARRACKS, 1);
    assert_eq!(p.in_production(), Some(ObjectKind::Peacekeeper));
    assert_eq!(queued(0).in_production(), None);
}

/// Design change 0143: a waiting unit at the front shows the cap it waits for; the wait ends with the unit's
/// place at the front: cancelled ("the player can still cancel it for a full refund"), refunded with the whole
/// queue, or, once it started, released. A cancel of an entry behind it leaves it waiting.
#[test]
fn the_wait_goes_with_the_unit_at_the_front() {
    let cap = Some(ArmyResource::UnitControl);
    let waiting = |n| Production {
        blocked: true,
        ..queued(n)
    };
    let status = waiting(2).status(&BARRACKS, cap).unwrap();
    assert_eq!((status.progress, status.blocked), (0, cap));
    assert_eq!(queued(2).status(&BARRACKS, cap).unwrap().blocked, None, "not waiting");
    let mut p = waiting(2);
    p.cancel_last();
    assert!(p.blocked, "the unit behind it went");
    p.cancel_last();
    assert!(!p.blocked && p.queue.is_empty());
    let mut p = waiting(3);
    assert_eq!(p.take_queue().len(), 3);
    assert_eq!((p.queue.len(), p.progress, p.blocked), (0, 0, false));
    let mut p = queued(2);
    while !p.advance(&BARRACKS, RATIO_ONE) {}
    p.blocked = true;
    p.release();
    assert!(!p.blocked, "the next unit starts its own check");
}

/// The wait is hashed only while it lasts (the goldens of games without one stand), and left out of files while
/// it doesn't, so that a state saved before it existed loads.
#[test]
fn the_wait_is_hashed_and_saved_only_while_it_lasts() {
    let hash = |p: &Production| {
        let mut h = StateHasher::new();
        p.hash(&mut h);
        h.finish()
    };
    let (idle, waiting) = (
        queued(1),
        Production {
            blocked: true,
            ..queued(1)
        },
    );
    assert_ne!(hash(&idle), hash(&waiting));
    let json = serde_json::to_string(&idle).unwrap();
    assert!(!json.contains("blocked"), "{json}");
    let old: Production = serde_json::from_str(r#"{"queue": ["Peacekeeper"], "progress": 0, "rally": null}"#).unwrap();
    assert_eq!(old, idle);
    let saved: Production = serde_json::from_str(&serde_json::to_string(&waiting).unwrap()).unwrap();
    assert_eq!(saved, waiting);
}

/// (M5-C) A Recruitment Hall's completion tick is hashed only while it isn't 0 (the goldens stand), and left out
/// of files while it is, so that a state saved before it existed loads. Its Recruit's total is 192 frames of
/// its area's 100 tiles: 192 ticks at 100 tiles a frame, 384 at 50 (`factions/cults/structures/recruitment_hall.md >
/// BaseUnitControlCapacity - 20`: "production time = 24 seconds per Recruit").
#[test]
fn a_completion_tick_is_hashed_and_saved_only_while_it_is_set() {
    let hash = |p: &Production| {
        let mut h = StateHasher::new();
        p.hash(&mut h);
        h.finish()
    };
    let (pre_built, built) = (
        queued(1),
        Production {
            completed_at: 300,
            ..queued(1)
        },
    );
    assert_ne!(hash(&pre_built), hash(&built));
    assert!(!serde_json::to_string(&pre_built).unwrap().contains("completed_at"));
    let saved: Production = serde_json::from_str(&serde_json::to_string(&built).unwrap()).unwrap();
    assert_eq!(saved, built);
    let hall = ObjectKind::RecruitmentHall
        .def()
        .structure
        .as_ref()
        .and_then(|s| s.production.as_ref())
        .unwrap();
    assert_eq!(total_of(hall, 192), 19_200);
    let (mut full, mut half) = (Production::default(), Production::default());
    full.push(QueueEntry::Unit(ObjectKind::Recruit), Vec::new());
    half.push(QueueEntry::Unit(ObjectKind::Recruit), Vec::new());
    assert_eq!((1..=192).filter(|_| full.advance(hall, 100)).count(), 1);
    assert_eq!((1..=384).position(|_| half.advance(hall, 50)), Some(383));
}

/// Design change 0083: the work queued is the time still ahead, the unit in production counted by what is left
/// of it.
#[test]
fn the_work_queued_is_the_production_time_still_ahead() {
    assert_eq!(queued(0).remaining_work(&BARRACKS), 0);
    assert_eq!(queued(3).remaining_work(&BARRACKS), 3 * (80 << 16));
    let mut nearly = queued(1);
    for _ in 0..79 {
        nearly.advance(&BARRACKS, RATIO_ONE);
    }
    assert_eq!(nearly.remaining_work(&BARRACKS), 1 << 16);
    assert!(queued(QUEUE_LIMIT).is_full() && !queued(QUEUE_LIMIT - 1).is_full());
}

#[test]
fn a_rally_on_an_object_that_is_gone_is_cleared() {
    let mut p = Production {
        rally: Some(Rally::Object {
            id: ObjectId(7),
            at: glam::Vec2::ONE,
        }),
        ..Production::default()
    };
    p.forget(|id| id == ObjectId(8));
    assert!(p.rally.is_some());
    p.forget(|id| id == ObjectId(7));
    assert_eq!(p.rally, None);
    let mut ground = Production {
        rally: Some(Rally::Point(glam::Vec2::ONE)),
        ..Production::default()
    };
    ground.forget(|_| true);
    assert!(ground.rally.is_some());
}

/// (M5-S6) The types that have a RallyPoint, which an unfinished building of theirs takes too (design change 0396: "A
/// building that will produce or release units"): those that train, recruit or augment units and let them out, the
/// Hideout and (M6-SV) the Vehicle Assembly into the Tunnel Network, (M5-N7) the Airstrip off its runway, (M5-N6) the
/// Chop Shop; not the Temple, which researches (slice M5-N3), nor the Garrison, which lets its units out but "has no
/// rally point" (`factions/cults/structures/garrison.md > Placeholder stats > Panel`), nor any other structure or unit.
#[test]
fn the_types_that_produce_or_release_units_take_a_rally_point() {
    use ObjectKind::*;
    let taking: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|&kind| takes_rally(kind))
        .collect();
    let mut expected = vec![
        Barracks,
        RoboPortal,
        BioPortal,
        Hideout,
        RecruitmentHall,
        Armory,
        Altar,
        Cathedral,
        Airstrip,
        ChopShop,
        VehicleAssembly,
    ];
    expected.sort_unstable();
    assert_eq!(taking, expected);
    for kind in [
        Temple, Garrison, Shrine, Storage, Generator, EgressNode, Beacon, Tunnel, Recruit,
    ] {
        assert!(!takes_rally(kind), "{kind:?}");
    }
}

/// The Armory's training (slice M5-F2), as a pool building's entries have it: 20 Space Crystals and 32 frames for a
/// Militant (`factions/cults/structures/armory.md > Training`).
const ARMORY: ProductionDef = ProductionDef {
    exits: ExitRule::Side(SideLetter::C),
    recipes: &[Recipe {
        kind: ObjectKind::Militant,
        cost: 20,
        frames: 32,
        requires: &[],
        supplies: 0,
    }],
    recruitment: None,
    augment: &[],
    default_rally: None,
};

/// (M5-F2) A pool building's entries each come with the units they reserve, which the queue hands back with the entry
/// as it leaves: at the front once finished (`Production::release`), at the end by X (`Production::cancel_last`:
/// "returns its reserved Recruit to StoredRecruits", `factions/cults/structures/armory.md >
/// ObjectInterfaceState[Armory]`), all of them when the building goes (`Production::take_queue`). A Release is free
/// and takes `RELEASE_FRAMES` (design change 0337: "**free**, very fast (**4 frames**, placeholder)"), and produces
/// nothing, so it is never "in production" (no army cost).
#[test]
fn a_pool_buildings_entries_hand_back_their_reserved_units() {
    let militant = QueueEntry::Unit(ObjectKind::Militant);
    let (a, b, c) = (ObjectId(4), ObjectId(5), ObjectId(6));
    let mut p = Production::default();
    p.push(militant, vec![a]);
    p.push(QueueEntry::Release, vec![b]);
    p.push(militant, vec![c]);
    assert_eq!(
        (p.reserved_by_current(), p.reserves(b), p.reserves(ObjectId(7))),
        (&[a][..], true, false)
    );
    assert_eq!((militant.cost(&ARMORY), militant.frames(&ARMORY)), (20, 32));
    assert_eq!(
        (QueueEntry::Release.cost(&ARMORY), QueueEntry::Release.frames(&ARMORY)),
        (0, RELEASE_FRAMES)
    );
    assert_eq!(
        (militant.units_reserved(), QueueEntry::Release.units_reserved()),
        (1, 1)
    );
    assert_eq!(
        p.remaining_work(&ARMORY),
        u64::from((32 + RELEASE_FRAMES + 32) * RATIO_ONE)
    );
    assert_eq!(p.cancel_last(), Some((militant, vec![c])));
    assert_eq!((1..).find(|_| p.advance(&ARMORY, RATIO_ONE)), Some(32));
    assert_eq!(p.release(), Some((militant, vec![a])));
    assert_eq!((p.reserved_by_current(), p.in_production()), (&[b][..], None));
    p.advance(&ARMORY, RATIO_ONE);
    assert_eq!(p.in_production(), None, "a Release produces nothing");
    let status = p.status(&ARMORY, None).unwrap();
    assert_eq!(
        (status.entry, status.total),
        (QueueEntry::Release, RELEASE_FRAMES * RATIO_ONE)
    );
    p.push(militant, vec![a]);
    assert_eq!(p.take_queue(), [(QueueEntry::Release, vec![b]), (militant, vec![a])]);
    assert!(p.queue.is_empty() && p.reserved.is_empty());
}

/// (M5-F2) The reservations are hashed only while there are some, and left out of files while there are none, so
/// that the states of games without a pool building at work hash and save as before they existed (the goldens).
#[test]
fn the_reservations_are_hashed_and_saved_only_while_there_are_some() {
    let hash = |p: &Production| {
        let mut h = StateHasher::new();
        p.hash(&mut h);
        h.finish()
    };
    let militant = QueueEntry::Unit(ObjectKind::Militant);
    let (mut plain, mut reserving) = (Production::default(), Production::default());
    plain.push(militant, Vec::new());
    reserving.push(militant, vec![ObjectId(4)]);
    assert_ne!(hash(&plain), hash(&reserving));
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("reserved"), "{json}");
    let saved: Production = serde_json::from_str(&serde_json::to_string(&reserving).unwrap()).unwrap();
    assert_eq!(saved, reserving);
    let old: Production = serde_json::from_str(r#"{"queue": ["Militant"], "progress": 0, "rally": null}"#).unwrap();
    assert_eq!(old, plain);
}
