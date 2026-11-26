use super::*;
use crate::units::TilePos;

fn storage_at(x: i32, y: i32) -> Placement {
    Placement {
        origin: TilePos::new(x, y),
        w: 2,
        h: 2,
        rotation: Default::default(),
        flip_h: false,
        flip_v: false,
    }
}

/// A claim keeps another unit of its owner's working at the same footprint off the spots its room overlaps, and
/// nothing else: not itself, not a unit of another player's, not a unit working at another footprint, not a room that
/// only touches it.
#[test]
fn a_claim_keeps_its_owners_others_off_its_room() {
    let at = storage_at(4, 4);
    let room = Vec2::splat(12.0);
    let claim = Claim {
        unit: ObjectId(1),
        owner: Some(PlayerId(1)),
        at,
        spot: Vec2::new(100.0, 100.0),
        room,
    };
    let keeps = |unit, owner, at: &Placement, spot| claim.keeps(ObjectId(unit), Some(PlayerId(owner)), at, spot, room);
    assert!(keeps(2, 1, &at, Vec2::new(110.0, 100.0)), "overlapping");
    assert!(!keeps(1, 1, &at, Vec2::new(110.0, 100.0)), "its own claim");
    assert!(!keeps(2, 2, &at, Vec2::new(110.0, 100.0)), "another player's unit");
    assert!(
        !keeps(2, 1, &storage_at(8, 8), Vec2::new(110.0, 100.0)),
        "another footprint"
    );
    assert!(!keeps(2, 1, &at, Vec2::new(124.0, 100.0)), "touching is clear");
}

/// The reach is `tuning::construction::BUILD_REACH_SU` from the footprint's edge, edge to edge: a Recruit against the
/// footprint is within it, as is one 8 su off, but not one a box further out, and one on the footprint is.
#[test]
fn the_reach_is_eight_space_units_from_the_footprints_edge() {
    let at = storage_at(4, 4);
    // The footprint spans 256..384 su on each axis; a Recruit's box is 24 su wide.
    let recruit = |x: f32| Shape::rect(Vec2::new(x, 320.0), Vec2::splat(12.0));
    assert!(within_reach(&recruit(384.0 + 12.0 + 1.0 / 64.0), &at), "against it");
    assert!(within_reach(&recruit(384.0 + 12.0 + 8.0), &at), "8 su off");
    assert!(!within_reach(&recruit(384.0 + 12.0 + 8.5), &at), "beyond");
    assert!(!within_reach(&recruit(384.0 + 36.0), &at), "a box further out");
    assert!(within_reach(&recruit(320.0), &at), "on the footprint");
}
