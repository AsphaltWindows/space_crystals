//! The job's changes of patch (`gather::retarget`); the behaviour itself is tested in the game, `tests/gathering.rs`.

use super::*;

const PATCH: ObjectId = ObjectId(1);
const OTHER: ObjectId = ObjectId(2);
const NODE: ObjectId = ObjectId(3);

fn job(phase: GatherPhase, load: u32) -> (OrderKind, Gather, Worker) {
    let worker = Worker {
        load,
        drop_off: Some(NODE),
        patch: Some(PATCH),
    };
    (OrderKind::Gather(PATCH), Gather::new(phase), worker)
}

/// Question Q0037 point 7: a worker whose patch goes turns to another one, at once if it was on its way there
/// or at work, after its drop-off if it carries a load.
#[test]
fn a_job_moves_on_to_another_patch() {
    for phase in [
        GatherPhase::ToPatch,
        GatherPhase::Waiting { since: 5 },
        GatherPhase::Mining { done: 9 },
    ] {
        let (mut kind, mut state, mut worker) = job(phase, 0);
        state.moving = true;
        assert!(retarget(&mut kind, &mut state, &mut worker, Some(OTHER)));
        assert_eq!((kind, worker.patch), (OrderKind::Gather(OTHER), Some(OTHER)));
        assert_eq!((state.phase, state.moving), (GatherPhase::ToPatch, false), "{phase:?}");
    }
    for phase in [
        GatherPhase::ToDropOff,
        GatherPhase::DroppingOff { done: 1 },
        GatherPhase::Delivered,
    ] {
        let (mut kind, mut state, mut worker) = job(phase, 8);
        state.moving = true;
        assert!(retarget(&mut kind, &mut state, &mut worker, Some(OTHER)));
        assert_eq!((kind, worker.patch), (OrderKind::Gather(OTHER), Some(OTHER)));
        assert_eq!((state.phase, state.moving), (phase, true), "it delivers first");
    }
}

/// Question Q0037 point 7: "With no patch left, the Drone delivers any load and idles."
#[test]
fn with_no_patch_left_a_job_ends_or_delivers_its_load() {
    // Nothing to deliver: the order ends.
    for phase in [
        GatherPhase::ToPatch,
        GatherPhase::Waiting { since: 5 },
        GatherPhase::Delivered,
    ] {
        let (mut kind, mut state, mut worker) = job(phase, 0);
        assert!(!retarget(&mut kind, &mut state, &mut worker, None), "{phase:?}");
        assert_eq!(worker.patch, None);
    }
    // A load on the way: a Drop Off at the drop-off the worker heads for, else at its AssignedDropOff.
    let (mut kind, mut state, mut worker) = job(GatherPhase::ToDropOff, 8);
    assert!(retarget(&mut kind, &mut state, &mut worker, None));
    assert_eq!((kind, state.phase), (OrderKind::DropOff(NODE), GatherPhase::ToDropOff));
    let (mut kind, mut state, mut worker) = job(GatherPhase::DroppingOff { done: 1 }, 8);
    state.node = Some(OTHER);
    assert!(retarget(&mut kind, &mut state, &mut worker, None));
    assert_eq!(kind, OrderKind::DropOff(OTHER));
    // A load and no drop-off: it idles holding it (point 8).
    let (mut kind, mut state, mut worker) = job(GatherPhase::ToDropOff, 8);
    worker.drop_off = None;
    assert!(!retarget(&mut kind, &mut state, &mut worker, None));
    assert_eq!(worker.load, 8);
}
