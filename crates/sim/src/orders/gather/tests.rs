//! The job's changes of patch, its end and the claims (`go_on`, `end`, `Claims`); the behaviour itself is tested
//! in the game, `tests/gathering*.rs`.

use glam::Vec2;

use super::deliver::delivered;
use super::patch::{end, go_on};
use super::*;
use crate::defs::ObjectKind;
use crate::economy::Worker;
use crate::ids::ObjectId;
use crate::mover::Mover;

const PATCH: ObjectId = ObjectId(1);
const OTHER: ObjectId = ObjectId(2);
const NODE: ObjectId = ObjectId(3);
const ME: ObjectId = ObjectId(5);

fn job(phase: GatherPhase) -> (OrderKind, Gather) {
    let mut state = Gather::new(phase);
    state.moving = true;
    state.spot = Some(Vec2::new(96.0, 160.0));
    (OrderKind::Gather(PATCH), state)
}

/// The order follows the worker to another patch (design change 0132: "its gather target follows where it
/// actually mined"; 0134: the spot of a patch that is gone is forgotten): at once if the worker was on its way there
/// or at work, and it claims a free one; after its drop-off if it carries a load, and it claims nothing meanwhile.
#[test]
fn a_job_moves_on_to_another_patch() {
    for phase in [
        GatherPhase::ToPatch,
        GatherPhase::Waiting { since: 5 },
        GatherPhase::Mining { done: 9 },
    ] {
        let (mut kind, mut state) = job(phase);
        let mut claims = Claims::default();
        go_on(&mut kind, &mut state, (ME, None), OTHER, true, &mut claims);
        assert_eq!(kind, OrderKind::Gather(OTHER));
        assert_eq!(
            (state.phase, state.moving, state.spot),
            (GatherPhase::ToPatch, false, None),
            "{phase:?}"
        );
        assert!(claims.by_other(OTHER, NODE) && !claims.by_other(OTHER, ME), "{phase:?}");
    }
    for phase in [
        GatherPhase::ToDropOff,
        GatherPhase::DroppingOff { done: 1 },
        GatherPhase::Delivered,
    ] {
        let (mut kind, mut state) = job(phase);
        let mut claims = Claims::default();
        go_on(&mut kind, &mut state, (ME, None), OTHER, true, &mut claims);
        assert_eq!((kind, state.spot), (OrderKind::Gather(OTHER), None));
        assert_eq!((state.phase, state.moving), (phase, true), "it delivers first");
        assert!(!claims.by_other(OTHER, NODE), "{phase:?}");
    }
    // A patch that isn't free is no claim.
    let (mut kind, mut state) = job(GatherPhase::ToPatch);
    let mut claims = Claims::default();
    go_on(&mut kind, &mut state, (ME, None), OTHER, false, &mut claims);
    assert!(!claims.by_other(OTHER, NODE));
}

/// Design change 0123: with no patch left, the Gather ends; a worker that carries a load delivers it first.
#[test]
fn with_no_patch_left_a_job_ends_or_delivers_its_load_first() {
    let mut mover = Mover::default();
    for (phase, load) in [
        (GatherPhase::ToPatch, 0),
        (GatherPhase::Waiting { since: 5 }, 0),
        (GatherPhase::Mining { done: 9 }, 0),
        (GatherPhase::ToPatch, 8),
        (GatherPhase::ToDropOff, 8),
        (GatherPhase::DroppingOff { done: 1 }, 8),
    ] {
        let mut worker = Worker {
            load,
            drop_off: Some(NODE),
            ..Worker::default()
        };
        let unit = Unit {
            id: ME,
            owner: None,
            kind: ObjectKind::Drone,
            pos: Vec2::ZERO,
            facing: 0.0,
            mover: &mut mover,
            attack: None,
            weapon: None,
            max_speed: ObjectKind::Drone.def().unit.as_ref().unwrap().movement.max_speed(),
            may_attack: true,
            converting: false,
            worker: Some(&mut worker),
        };
        let (_, mut state) = job(phase);
        let flow = end(&mut state, &unit);
        if load == 0 {
            assert!(
                matches!(flow, Flow::Done),
                "{phase:?}: nothing to deliver, the order is complete"
            );
            assert!(!state.ending);
            continue;
        }
        assert!(matches!(flow, Flow::Next), "{phase:?}");
        assert!(state.ending, "{phase:?}");
        let delivering = if phase == GatherPhase::ToPatch {
            GatherPhase::ToDropOff
        } else {
            phase
        };
        assert_eq!(state.phase, delivering, "on to the drop-off, or on with it");
        assert!(matches!(delivered(OrderKind::Gather(PATCH), &mut state), Flow::Done));
    }
}

/// The phases after a delivery: a Gather goes back to its patch unless it is ending; a Drop Off is complete
/// (design change 0127).
#[test]
fn after_a_delivery_a_gather_goes_back_and_a_drop_off_is_complete() {
    let mut state = Gather::new(GatherPhase::Delivered);
    state.node = Some(NODE);
    assert!(matches!(delivered(OrderKind::Gather(PATCH), &mut state), Flow::Next));
    assert_eq!((state.phase, state.node), (GatherPhase::ToPatch, None));
    let mut state = Gather::new(GatherPhase::Delivered);
    assert!(matches!(delivered(OrderKind::DropOff(NODE), &mut state), Flow::Done));
}
