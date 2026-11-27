use super::super::tests::{building, orientations};
use super::*;
use crate::defs::ObjectKind;
use crate::object::Rotation;

/// A Sparrow's half extents (`factions/cults/units/sparrow.md`: "Silhouette 40x40").
const HALF: Vec2 = Vec2::splat(20.0);

fn airstrip(rotation: Rotation, flip_h: bool) -> Runway {
    Runway::of(&building(ObjectKind::Airstrip, rotation, flip_h)).expect("a runway")
}

/// `factions/cults/structures/airstrip.md > Placeholder stats`: "Size 2x4 (a runway); SymmetryType ABAC with the short
/// sides B and C [...] a finished aircraft is placed **directly on the runway** and takes off along it toward C",
/// wherever its sides face: the aircraft appears wholly over the runway against side B, centred across it, facing C;
/// its run ends at the middle of side C, and with no rally point it circles 2 grid units past it.
#[test]
fn the_runway_runs_from_side_b_to_side_c_in_all_eight_orientations() {
    // Unturned: 4 wide and 2 tall at tile (2, 1), so x 128..384 and y 64..192; B faces east, C west.
    let cases = [
        (Rotation::R0, false, Vec2::new(384.0, 128.0), -Vec2::X),
        (Rotation::R180, false, Vec2::new(128.0, 128.0), Vec2::X),
        (Rotation::R0, true, Vec2::new(128.0, 128.0), Vec2::X),
        (Rotation::R180, true, Vec2::new(384.0, 128.0), -Vec2::X),
        // Turned on its side: 2 wide and 4 tall, so x 128..256 and y 64..320.
        (Rotation::R90, false, Vec2::new(192.0, 320.0), -Vec2::Y),
        (Rotation::R270, false, Vec2::new(192.0, 64.0), Vec2::Y),
        (Rotation::R90, true, Vec2::new(192.0, 64.0), Vec2::Y),
        (Rotation::R270, true, Vec2::new(192.0, 320.0), -Vec2::Y),
    ];
    assert_eq!(cases.len(), orientations().count());
    for (rotation, flip_h, b, along) in cases {
        let runway = airstrip(rotation, flip_h);
        let what = format!("{rotation:?}, flipped {flip_h}");
        assert_eq!(runway.along(), along, "{what}");
        assert_eq!(runway.start(HALF), b + along * 20.0, "{what}");
        // Side C, the runway's 4 tiles on from side B.
        let c = b + along * 256.0;
        assert_eq!(runway.takeoff(), Takeoff { end: c, along }, "{what}");
        assert_eq!(runway.circle_point(), c + along * 128.0, "{what}");
    }
}

/// The run is over once the aircraft's centre is past the runway's end, not before.
#[test]
fn the_run_ends_past_side_c() {
    let run = airstrip(Rotation::R0, false).takeoff();
    assert!(!run.is_over(Vec2::new(364.0, 128.0)), "at the start");
    assert!(!run.is_over(Vec2::new(128.5, 150.0)), "over the runway");
    assert!(run.is_over(Vec2::new(128.0, 128.0)), "on side C");
    assert!(run.is_over(Vec2::new(100.0, 90.0)), "past it");
}

/// Only a building whose production takes off from a runway has one: none for the Armory, whose units come out of a
/// side doorway.
#[test]
fn only_a_runway_building_has_a_runway() {
    assert_eq!(Runway::of(&building(ObjectKind::Armory, Rotation::R0, false)), None);
}
