//! What a player knows of the objects its orders target — `core/vision.md > Explored > Targeting remembered objects`
//! (design change 0139, which answers question Q0017 point 2).
//!
//! "Remembered objects (structures, resource patches and other objects shown in their last-known state in Explored
//! areas) **can be targeted** by right-click and by targeted commands (Attack, Gather, Move to object, rally points),
//! even though they are not selectable. The order goes to the object's **last-known position**. [...] Nothing about
//! the real state leaks: the cursor, the command and any rally marker treat the remembered object as present until
//! the player sees otherwise."
//!
//! One view ([`TargetView`]) serves everything that resolves such a target: the gate (`Sim::validate`), the default
//! right-click command (`orders::resolution`, `Sim::default_order`), the behaviours of the orders that name an object
//! (`orders::world::World::known`, each tick), the rally points (`production::rally`) and the removal path, which
//! leaves alone what names an object its holder's owner still remembers (`Sim::remove_objects`). It is built from
//! the player's sight and its last-known records (`vision::LastKnown`) only, so it reads nothing of the truth about
//! an object the player doesn't see: a remembered object destroyed in the fog is Remembered exactly as one that still
//! stands, until the player sees its place.

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::object::{Placement, ResourceState};
use crate::vision::{LastKnown, PlayerVision};

/// What a player knows of an object one of its orders or rally points targets (design change 0139).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TargetView {
    /// The player sees it now, and its own objects always: the object as it is.
    Visible,
    /// The player doesn't see it, but remembers it: a structure or a resource it has seen, as its record has it
    /// (`core/vision.md > Explored`). The order goes to `position`, the last-known one, the centre of `footprint`,
    /// and treats it as present there until the player sees the place. Whether it still exists, nothing says.
    Remembered { position: Vec2, footprint: Placement },
    /// Neither: as far as the player knows there is nothing to go to. An object it saw gone (its record was dropped),
    /// a unit out of its sight (units aren't remembered: a Follow ends when its target leaves sight, design change
    /// 0034), or one it never saw.
    Gone,
}

impl TargetView {
    /// Whether the player knows of the object: it sees or remembers it (the gate's rule for an object target,
    /// design change 0139).
    pub fn is_known(&self) -> bool {
        !matches!(self, TargetView::Gone)
    }
}

/// An object of the sim as [`view`] reads it: its owner and the box it covers (`combat::object_box`, space units).
/// Only whether the player sees it is read of it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Live {
    pub owner: Option<PlayerId>,
    pub lo: Vec2,
    pub hi: Vec2,
}

/// The [`TargetView`] `player`, whose vision is `sight`, has of object `id`, which is `live` if it exists:
/// - **Visible** if it exists and is the player's own, or a tile it covers is Visible to the player;
/// - else **Remembered** if the player has a record of it out of its sight now (`PlayerVision::remembered_record`:
///   no tile under the record is Visible), whether or not the object exists;
/// - else **Gone**. An object that doesn't exist on a tile the player sees is gone; the record of one that no
///   longer stands where the record has it goes in phase 5f (`vision::memory`), and so does its Remembered.
///
/// A player without a vision (not in the game) knows nothing but its own objects.
pub(crate) fn view(sight: Option<&PlayerVision>, player: PlayerId, id: ObjectId, live: Option<Live>) -> TargetView {
    if let Some(live) = live
        && (live.owner == Some(player) || sight.is_some_and(|sight| sight.sees_box(live.lo, live.hi)))
    {
        return TargetView::Visible;
    }
    match sight.and_then(|sight| sight.remembered_record(id)) {
        Some(record) => TargetView::Remembered {
            position: record.pos(),
            footprint: record.placement,
        },
        None => TargetView::Gone,
    }
}

/// Question Q0037 point 11 and design change 0137 ("the patch then counts as gone for the Drones") for `record`, an
/// object the player whose vision is `sight` remembers out of its sight, judged by what the player knows (design
/// change 0139): whether it is a Space Crystals patch with no mining structure on it that the player knows of. That is
/// none of its records of the others' structures (`PlayerVision::records_on`), and none of its own (`own_cover`: a
/// player always knows its own objects). The gate (`Sim::aimed`) and the workers (`orders::world::World::known_patch`)
/// both go by it.
pub(crate) fn remembered_gatherable(sight: &PlayerVision, record: &LastKnown, own_cover: bool) -> bool {
    let is_patch = matches!(record.resource, Some(ResourceState::CrystalPatch { .. }));
    let mines = |kind: ObjectKind| kind.def().structure.as_ref().is_some_and(|s| s.mining.is_some());
    is_patch
        && !own_cover
        && !sight
            .records_on(record.placement)
            .any(|r| r.id != record.id && mines(r.kind))
}
