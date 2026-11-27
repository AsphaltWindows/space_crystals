//! The SpeedTurnRadiusMovement numbers alone: the tread law, the signed speed, braking to a corner's speed, and the
//! speed the corner choice takes each turn at. The scenario tests in `crates/sim/tests/tracked/` cover a Hand of
//! Heaven in a game.

use super::*;
use crate::defs::ObjectKind;
use crate::math::PI;
const CORNER_SPEEDS: u32 = crate::tuning::movement::BUILTIN.corner_speeds;

/// The Hand of Heaven's movement numbers (`factions/cults/units/hand_of_heaven.md`).
fn hand_of_heaven(reverses: bool) -> Tracked {
    let movement = ObjectKind::HandOfHeaven.def().unit.as_ref().expect("a unit").movement;
    let mover = Mover {
        reverses,
        ..Mover::default()
    };
    Tracked::of(&movement, &mover).expect("a SpeedTurnRadiusMovement unit")
}

/// The tightest radius at the speed `v`, space units: the speed over the law's rate.
fn radius(model: &Tracked, v: f32) -> f32 {
    v / model.rate(v)
}

#[test]
fn these_are_the_hand_of_heaven_s_numbers() {
    let model = hand_of_heaven(false);
    assert_eq!(
        model,
        Tracked {
            spin: 1.0 / 3.0,
            max: 5.0,
            reverses: false,
            top: 5.0,
            accel: 0.5,
            decel: 1.0
        }
    );
    // Backing up: ReverseMaxSpeed 2.5, the same Acceleration and Deceleration, the same law.
    let back = hand_of_heaven(true);
    assert_eq!((back.top, back.accel, back.decel, back.max), (2.5, 0.5, 1.0, 5.0));
    let walker = MovementAttrs::TurnRate {
        turn_rate: 1.0,
        acceleration: None,
        deceleration: None,
        max_speed: 4.0,
    };
    assert_eq!(Tracked::of(&walker, &Mover::default()), None);
}

/// Design change 0417's law and its example: "spins at about 19° a frame (90° in about 0.3 s); at half speed about
/// 9.5° a frame on a 15 su radius; at 4 su/frame about 3.8° a frame on a 60 su radius; at 4.5 about 1.9° on 135 su; at
/// 5 it can't turn". Backing up, the same with the speed's size.
#[test]
fn the_tread_law_turns_it_less_the_faster_it_goes() {
    let model = hand_of_heaven(false);
    let degrees = |v: f32| model.rate(v) * 180.0 / PI;
    assert!((degrees(0.0) - 19.1).abs() < 0.05);
    assert!((degrees(2.5) - 9.55).abs() < 0.05 && (radius(&model, 2.5) - 15.0).abs() < 1e-4);
    assert!((degrees(4.0) - 3.82).abs() < 0.05 && (radius(&model, 4.0) - 60.0).abs() < 1e-3);
    assert!((degrees(4.5) - 1.91).abs() < 0.05 && (radius(&model, 4.5) - 135.0).abs() < 1e-2);
    assert_eq!(model.rate(5.0), 0.0);
    assert_eq!(model.rate(-2.5), model.rate(2.5));
    // 90° at SpinRate: in its fifth tick.
    assert_eq!((PI / 2.0 / model.rate(0.0)).ceil(), 5.0);
}

/// The speed carries from tick to tick: up by Acceleration, down by Deceleration, backing up no faster than
/// ReverseMaxSpeed, and braking to rest before it sets off the other way.
#[test]
fn it_speeds_up_by_acceleration_and_brakes_by_deceleration() {
    let model = hand_of_heaven(false);
    assert_eq!(model.next_speed(0.0, 9.0), 0.5);
    assert_eq!(model.next_speed(4.8, 9.0), 5.0);
    assert_eq!(model.next_speed(5.0, 2.5), 4.0);
    assert_eq!(model.next_speed(-1.5, 5.0), -0.5, "it brakes to rest first");
    let back = hand_of_heaven(true);
    assert_eq!(back.next_speed(-2.2, 9.0), -2.5);
    assert_eq!(back.next_speed(3.0, 9.0), 2.0, "moving forward, it brakes first");
}

/// Braking to a corner's speed: from the speed it allows now, Deceleration each tick brings the unit to the corner's
/// speed within the room it has, and never sooner than it must.
#[test]
fn it_brakes_in_time_for_a_corner() {
    let model = hand_of_heaven(false);
    for (room, corner) in [
        (0.0, 2.5),
        (3.0, 2.5),
        (10.0, 2.5),
        (12.0, 2.5),
        (30.0, 0.0),
        (7.0, 0.0),
        (100.0, 3.0),
    ] {
        let first = model.brake_to(room, corner);
        // Braking every tick from it: the moves until it is at the corner's speed.
        let (mut speed, mut way) = (first, 0.0);
        while speed > corner {
            way += speed;
            speed = (speed - model.decel).max(corner);
        }
        assert!(
            way <= room + 1e-4,
            "{first} su a tick takes {way} su to slow to {corner}, more than {room}"
        );
        // One more Deceleration than that, and it wouldn't be in time.
        let faster = (first + model.decel).min(model.top);
        if faster > first {
            let (mut speed, mut way) = (faster, 0.0);
            while speed > corner {
                way += speed;
                speed = (speed - model.decel).max(corner);
            }
            assert!(
                way > room,
                "{faster} su a tick would also slow to {corner} within {room}"
            );
        }
    }
    assert_eq!(model.brake_to(0.0, 2.5), 2.5);
    assert_eq!(model.brake_to(1000.0, 0.0), 5.0);
}

/// The speed the corner choice takes a turn at, where its arc fits (module docs): the one that makes the turn
/// soonest. From full speed, the sharper the turn the slower; a gentle curve within a tenth of MaxSpeed; a right
/// angle at about half speed, pivoting about its inner tread; a turn about almost stopped. From rest it spins.
#[test]
fn the_corner_choice_slows_the_more_the_sharper_the_turn() {
    let model = hand_of_heaven(false);
    let at =
        |from: f32, degrees: f32| model.top * model.soonest(from, degrees * PI / 180.0) as f32 / CORNER_SPEEDS as f32;
    let from_full: Vec<f32> = [5.0, 15.0, 30.0, 45.0, 90.0, 135.0, 180.0]
        .iter()
        .map(|&d| at(5.0, d))
        .collect();
    println!("from full speed, 5° to 180°: {from_full:?}");
    assert!(from_full.windows(2).all(|pair| pair[0] >= pair[1]), "{from_full:?}");
    assert!(
        from_full[1] >= 4.5,
        "a gentle curve within 10% of MaxSpeed: {from_full:?}"
    );
    assert!(
        (2.0..=3.0).contains(&from_full[4]),
        "a right angle at about half speed: {from_full:?}"
    );
    assert!(from_full[6] <= 1.0, "a turn about almost stopped: {from_full:?}");
    for degrees in [10.0, 90.0, 180.0] {
        assert_eq!(at(0.0, degrees), 0.0, "from rest it spins first, {degrees}°");
    }
    // Backing up, the same law at its reverse speed.
    let back = hand_of_heaven(true);
    let backing = back.top * back.soonest(2.5, PI / 2.0) as f32 / CORNER_SPEEDS as f32;
    assert!(backing > 0.0 && backing <= 2.5, "{backing}");
}
