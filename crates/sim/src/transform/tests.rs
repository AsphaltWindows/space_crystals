//! The arithmetic of a transformation (`crate::transform`): its progress, the interpolated maxima and what becomes
//! of the current values. A game's Warp Ins are `tests/warp_in.rs`.

use glam::Vec2;

use super::*;
use crate::ids::ObjectId;
use crate::units::TilePos;

use ObjectKind::{Beacon, BioPortal, EgressNode, RoboPortal};

/// A transformation of a Beacon into `kind`, as a Warp In starts it.
fn warping(kind: ObjectKind) -> Transform {
    let beacon = Placement::new(TilePos::new(4, 4), Beacon.def().footprint);
    Transform::new(
        Beacon,
        Some(beacon),
        warp_in_recipe(Beacon, kind).expect("on the Beacon's list"),
    )
}

/// A `kind` with `transform`, undamaged as far as its kind goes.
fn object(kind: ObjectKind, transform: Option<Transform>) -> Object {
    Object {
        id: ObjectId(1),
        kind,
        owner: None,
        pos: Vec2::ZERO,
        prev_pos: Vec2::ZERO,
        facing: 0.0,
        prev_facing: 0.0,
        hp: kind.def().max_hp,
        placement: None,
        resource: None,
        orders: None,
        mover: None,
        attack: None,
        riot_shield: None,
        extractor: None,
        production: None,
        constructor: None,
        worker: None,
        transform,
    }
}

/// `factions/colonists/economy.md > Warp In > Constructs List`: the three buildings with a cost and a WarpInTime;
/// nothing else warps anything in.
#[test]
fn the_beacon_warps_in_its_three_buildings() {
    let list: Vec<(ObjectKind, u32, u32)> = warp_in_recipes(Beacon)
        .iter()
        .map(|r| (r.kind, r.cost, r.frames))
        .collect();
    assert_eq!(
        list,
        [(EgressNode, 75, 320), (RoboPortal, 400, 640), (BioPortal, 150, 480)]
    );
    assert_eq!(warp_in_recipe(Beacon, ObjectKind::Barracks), None);
    for &kind in ObjectKind::ALL {
        if kind != Beacon {
            assert!(warp_in_recipes(kind).is_empty(), "{kind:?}");
        }
    }
}

/// Progress counts whole frames, as the design's "frames elapsed", and completes at the WarpInTime; what was paid
/// and what it was stay as they started.
#[test]
fn progress_counts_whole_frames_up_to_the_warp_in_time() {
    let mut t = warping(EgressNode);
    assert_eq!((t.from, t.paid, t.total), (Beacon, 75, 320));
    assert_eq!((t.frames_done(), t.percent(), t.ratio()), (0, 0, 0.0));
    assert!(!t.advance(RATIO_ONE / 2));
    assert_eq!(t.frames_done(), 0, "half a frame isn't one");
    assert!(!t.advance(RATIO_ONE / 2));
    assert_eq!(t.frames_done(), 1);
    for _ in 1..319 {
        assert!(!t.advance(RATIO_ONE));
    }
    assert_eq!(
        (t.frames_done(), t.percent()),
        (319, 99),
        "rounded down: 100% only once complete"
    );
    assert!(t.advance(RATIO_ONE));
    assert_eq!((t.frames_done(), t.percent(), t.ratio()), (320, 100, 1.0));
    assert!(t.is_complete());
    // A hand-edited save's progress saturates.
    t.progress = u32::MAX - 1;
    assert!(t.advance(RATIO_ONE) && t.progress == u32::MAX && t.frames_done() == 320);
}

/// `> HP/Shields Interpolation`: "HP = Beacon MaxHP + (Building MaxHP - Beacon MaxHP) x progress", the same for
/// Shields, at 0, 25, 50 and 100% of each WarpInTime: a Robo Portal at 50% has 360 HP and 15 Shields (the plan's
/// example). Once complete, the maxima are the building's own.
#[test]
fn the_maxima_interpolate_by_the_formulas() {
    let cases = [
        (RoboPortal, [120.0, 240.0, 360.0, 600.0], [10.0, 12.5, 15.0, 20.0]),
        (EgressNode, [120.0, 140.0, 160.0, 200.0], [10.0; 4]),
        (BioPortal, [120.0, 152.5, 185.0, 250.0], [10.0, 11.25, 12.5, 15.0]),
    ];
    for (kind, hp, shields) in cases {
        let mut t = warping(kind);
        for (i, quarter) in [0, 1, 2, 4].into_iter().enumerate() {
            t.progress = t.total / 4 * quarter * RATIO_ONE;
            let o = object(kind, Some(t.clone()));
            assert_eq!(
                (o.max_hp(), o.max_shields()),
                (Some(hp[i]), Some(shields[i])),
                "{kind:?} at {}%",
                quarter * 25
            );
        }
        let done = object(kind, None);
        assert_eq!((done.max_hp(), done.max_shields()), (Some(hp[3]), Some(shields[3])));
    }
}

/// Design change 0110: each frame, "current += (newMax − oldMax)". Undamaged, it stays exactly at the maximum;
/// damaged, it keeps the damage; a maximum that falls takes it down by as much, but doesn't kill.
#[test]
fn the_current_value_follows_its_maximum() {
    assert_eq!(follow_max(120.0, 120.0, 120.75), 120.75);
    assert_eq!(follow_max(100.0, 120.0, 360.0), 340.0);
    assert_eq!(follow_max(100.0, 120.0, 90.0), 70.0);
    assert_eq!(follow_max(20.0, 120.0, 90.0), 20.0, "not killed");
    // Never above its maximum, whatever it was.
    assert_eq!(follow_max(130.0, 120.0, 200.0), 200.0);
    // Frame by frame, undamaged, a Robo Portal stays at exactly its interpolated maximum.
    let mut t = warping(RoboPortal);
    let mut hp = 120.0;
    for _ in 0..640 {
        let before = t.interpolate(120.0, 600.0);
        t.advance(RATIO_ONE);
        hp = follow_max(hp, before, t.interpolate(120.0, 600.0));
        assert_eq!(hp, t.interpolate(120.0, 600.0), "frame {}", t.frames_done());
    }
    assert_eq!(hp, 600.0);
}

/// The current HP of a transformation of `t` from `from` to `to` HP, starting at `hp`, through `frames` frames, each
/// one's maximum followed ([`follow_max`]).
fn follow(t: &mut Transform, (from, to): (f32, f32), mut hp: f32, frames: u32) -> f32 {
    for _ in 0..frames {
        let before = t.interpolate(from, to);
        t.advance(RATIO_ONE);
        hp = follow_max(hp, before, t.interpolate(from, to));
    }
    hp
}

/// Design change 0110's Verification, to the HP: "Drone at 10/20 HP deploys undisturbed: completes at 110/120"; "Hit
/// a deploying Beacon for 30 at 50%: shows 40/70, completes at 90/120" (a deployment of 48 frames, `> Beacon
/// Deployment`: 3 seconds); "Same arithmetic for a Warp In (Beacon 120 → target MaxHP)": a Beacon 40 short of its
/// 120 is 40 short of each building's MaxHP once warped in, exactly.
#[test]
fn damage_is_carried_through_unchanged() {
    let deploying = || Transform {
        from: ObjectKind::Drone,
        from_placement: None,
        progress: 0,
        total: 48,
        paid: 0,
    };
    let drone_to_beacon = (20.0, 120.0);
    assert_eq!(follow(&mut deploying(), drone_to_beacon, 10.0, 48), 110.0);
    let mut t = deploying();
    let hp = follow(&mut t, drone_to_beacon, 20.0, 24);
    assert_eq!((hp, t.interpolate(20.0, 120.0)), (70.0, 70.0), "at 50%");
    assert_eq!(follow(&mut t, drone_to_beacon, hp - 30.0, 24), 90.0);
    for (kind, to) in [(EgressNode, 200.0), (RoboPortal, 600.0), (BioPortal, 250.0)] {
        let mut t = warping(kind);
        let frames = t.total;
        assert_eq!(follow(&mut t, (120.0, to), 80.0, frames), to - 40.0, "{kind:?}");
    }
}

/// Design change 0111: a cancel restores the old maxima with "the same fraction of HP and Shields", "original_max ×
/// current_before ÷ max_before"; its Verification: "Cancel a deploy at 90/120 → Drone 15/20", "Cancel a Warp In at
/// 50% HP → Beacon 60/120". The minimum of 1 HP ([`MIN_HP_AFTER_CANCEL`]) is the cancel's (`tests/warp_in.rs`).
#[test]
fn a_cancel_keeps_the_fraction() {
    assert_eq!(keep_fraction(90.0, 120.0, 20.0), 15.0, "the deployment's example");
    assert_eq!(keep_fraction(180.0, 360.0, 120.0), 60.0, "the Warp In's example");
    assert_eq!(
        keep_fraction(360.0, 360.0, 120.0),
        120.0,
        "undamaged: exactly the Beacon's"
    );
    assert_eq!(keep_fraction(50.0, 200.0, 120.0), 30.0);
    assert!(
        keep_fraction(0.5, 200.0, 120.0) < MIN_HP_AFTER_CANCEL,
        "no minimum of its own: Shields have none"
    );
    assert_eq!(
        keep_fraction(10.0, 0.0, 120.0),
        120.0,
        "no maximum to take a fraction of"
    );
}
