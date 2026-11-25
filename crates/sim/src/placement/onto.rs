//! The rule of what a structure goes onto (`factions/gdo/economy.md > ExtractionPlate`: "Placed onto a Space Crystal
//! Patch"), which the placement check (`super::check`) and map validation share.

use super::site::Occupant;
use crate::defs::ObjectKind;
use crate::nav::TileRect;

/// What a structure goes onto, the one statement of the rule (`factions/gdo/economy.md > ExtractionPlate`: "Placed
/// onto a Space Crystal Patch"): a structure whose def names a type to go onto (`StructureDef::onto`) goes exactly over
/// an object of that type, an Extraction Plate over a Space Crystals Patch, and nothing else goes onto anything. Shown
/// everything else on the map, in any order ([`Onto::see`]), it says whether a footprint keeps the rule
/// ([`Onto::is_kept`]). Every way a structure comes to stand asks it, so they can't drift apart:
/// [`check`](super::check()) for a game's placements and the Starting Layouts' ([`super::WholeProblem::NotOnPatch`]),
/// and map validation for the map's pre-placed objects, which follow no other placement rule
/// (`SpawnProblem::NotOnPatch`). Whether the patch has a plate already is the rule of shared tiles
/// ([`ObjectKind::may_overlap`]).
#[derive(Copy, Clone, Debug)]
pub struct Onto {
    /// The type the structure goes onto, its def's: the patch for a plate, `None` for the others.
    onto: Option<ObjectKind>,
    footprint: TileRect,
    kept: bool,
}

impl Onto {
    /// The rule for a structure of `kind` covering `footprint`, before anything is seen: kept, but by a plate.
    pub fn new(kind: ObjectKind, footprint: TileRect) -> Self {
        let onto = kind.def().structure.as_ref().and_then(|s| s.onto);
        Self {
            onto,
            footprint,
            kept: onto.is_none(),
        }
    }

    /// Takes `other` into account: a patch exactly under the plate keeps the rule.
    pub fn see(&mut self, other: &Occupant) {
        self.kept |= self.onto == Some(other.kind)
            && other
                .placement
                .is_some_and(|p| TileRect::new(p.origin, p.w, p.h) == self.footprint);
    }

    /// Whether the footprint keeps the rule, given what was seen.
    pub fn is_kept(&self) -> bool {
        self.kept
    }
}
