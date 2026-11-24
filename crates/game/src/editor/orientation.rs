//! Turning the structure about to be placed (slice M2-I; `core/entities.md > Structure Instance`: "Rotation - 0 | 90
//! | 180 | 270", "FlipHorizontal", "FlipVertical"). `core/map_editor.md` gives the editor no keys for it, so the
//! Place and Layout tools take those of the game's placement ghost (`factions/gdo/structures.md > DeploymentCenter >
//! ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`): "R rotates the ghost 90 degrees clockwise, Shift+R
//! rotates 90 degrees counter-clockwise. Non-square buildings swap footprint dimensions when rotated to 90 or 270. F
//! flips the ghost horizontally, Shift+F flips vertically". The flips are as seen on the map, whatever the turn
//! already is. Only structures turn; units and map resources are placed as they are.

use sc_sim::{BuildingOrientation, Category, ObjectKind, Rotation};
use serde::{Deserialize, Serialize};

/// A quarter turn or a flip of the structure being placed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Turn {
    /// R: a quarter turn clockwise.
    Clockwise,
    /// Shift+R: a quarter turn counter-clockwise.
    CounterClockwise,
    /// F: mirrored left and right.
    FlipHorizontal,
    /// Shift+F: mirrored top and bottom.
    FlipVertical,
}

impl Turn {
    pub const ALL: [Turn; 4] = [
        Turn::Clockwise,
        Turn::CounterClockwise,
        Turn::FlipHorizontal,
        Turn::FlipVertical,
    ];

    /// The button's label, with its key.
    pub fn label(self) -> &'static str {
        match self {
            Turn::Clockwise => "Turn R",
            Turn::CounterClockwise => "Back Shift+R",
            Turn::FlipHorizontal => "Flip F",
            Turn::FlipVertical => "Flip Shift+F",
        }
    }

    /// `orientation` turned this way. A building is flipped in its own frame first, then rotated clockwise
    /// (`sc_sim::Orientation::of_building`), so a turn adds to its rotation, and a flip as seen mirrors its rotation
    /// too: mirroring after a turn by `a` is mirroring before a turn by `-a`.
    pub fn apply(self, orientation: BuildingOrientation) -> BuildingOrientation {
        let quarters = quarters(orientation.rotation);
        let mut turned = orientation;
        match self {
            Turn::Clockwise => turned.rotation = rotation(quarters + 1),
            Turn::CounterClockwise => turned.rotation = rotation(quarters + 3),
            Turn::FlipHorizontal => {
                turned.rotation = rotation(4 - quarters);
                turned.flip_h = !turned.flip_h;
            }
            Turn::FlipVertical => {
                turned.rotation = rotation(4 - quarters);
                turned.flip_v = !turned.flip_v;
            }
        }
        turned
    }
}

/// Clockwise quarter turns of `rotation`, 0 to 3.
fn quarters(rotation: Rotation) -> u8 {
    match rotation {
        Rotation::R0 => 0,
        Rotation::R90 => 1,
        Rotation::R180 => 2,
        Rotation::R270 => 3,
    }
}

/// The rotation of `quarters` clockwise quarter turns, whole turns left out.
fn rotation(quarters: u8) -> Rotation {
    match quarters % 4 {
        0 => Rotation::R0,
        1 => Rotation::R90,
        2 => Rotation::R180,
        _ => Rotation::R270,
    }
}

/// Whether objects of `kind` are turned when placed: structures (`core/entities.md > Structure Type`).
pub fn turns(kind: ObjectKind) -> bool {
    kind.def().category == Category::Structure
}

/// What an orientation is, in words: "unturned", "turned 90", "turned 180, flipped left/right".
pub fn describe(orientation: BuildingOrientation) -> String {
    let mut parts = Vec::new();
    if orientation.rotation != Rotation::R0 {
        parts.push(format!("turned {}", u32::from(quarters(orientation.rotation)) * 90));
    }
    if orientation.flip_h {
        parts.push("flipped left/right".to_string());
    }
    if orientation.flip_v {
        parts.push("flipped top/bottom".to_string());
    }
    if parts.is_empty() {
        "unturned".into()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc_sim::Orientation;

    fn all_orientations() -> Vec<BuildingOrientation> {
        let mut all = Vec::new();
        for q in 0..4 {
            for flip_h in [false, true] {
                for flip_v in [false, true] {
                    all.push(BuildingOrientation {
                        rotation: rotation(q),
                        flip_h,
                        flip_v,
                    });
                }
            }
        }
        all
    }

    fn of(o: BuildingOrientation) -> Orientation {
        Orientation::of_building(o.rotation, o.flip_h, o.flip_v)
    }

    /// Each turn is the matching map orientation applied after the building's own, as seen: clockwise turns
    /// east into south (y grows southwards), and the flips mirror x and y.
    #[test]
    fn turns_follow_the_sims_orientations_as_seen() {
        assert_eq!(Orientation::ROTATE_90.apply((1, 0)), (0, 1), "clockwise on the map");
        let seen = [
            (Turn::Clockwise, Orientation::ROTATE_90),
            (Turn::CounterClockwise, Orientation::ROTATE_270),
            (Turn::FlipHorizontal, Orientation::MIRROR_X),
            (Turn::FlipVertical, Orientation::MIRROR_Y),
        ];
        for o in all_orientations() {
            for (turn, map) in seen {
                assert_eq!(of(turn.apply(o)), map.after(of(o)), "{turn:?} of {o:?}");
            }
        }
    }

    #[test]
    fn turns_undo_each_other_and_four_quarters_are_a_whole_turn() {
        for o in all_orientations() {
            assert_eq!(Turn::CounterClockwise.apply(Turn::Clockwise.apply(o)), o);
            assert_eq!(Turn::FlipHorizontal.apply(Turn::FlipHorizontal.apply(o)), o);
            assert_eq!(Turn::FlipVertical.apply(Turn::FlipVertical.apply(o)), o);
            let whole = (0..4).fold(o, |o, _| Turn::Clockwise.apply(o));
            assert_eq!(whole, o);
        }
        let quarter = Turn::Clockwise.apply(BuildingOrientation::default());
        assert!(quarter.swaps_axes());
        assert_eq!(describe(quarter), "turned 90");
        assert_eq!(describe(BuildingOrientation::default()), "unturned");
        assert_eq!(
            describe(Turn::FlipHorizontal.apply(quarter)),
            "turned 270, flipped left/right"
        );
        assert!(turns(ObjectKind::Barracks) && !turns(ObjectKind::CrystalPatch) && !turns(ObjectKind::Peacekeeper));
    }
}
