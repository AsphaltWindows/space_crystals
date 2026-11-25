//! Tech and unit prerequisites — `core/tech_and_research.md > Tech Prerequisites` (plan decision T37, slice M4-K): one
//! rule for every faction. A structure type may declare a TechPrerequisite (`StructureDef::requires`), and an entry of
//! a producer's or a constructor's list one of its own (`Recipe::requires`: a unit's Unit Prerequisite, a "Requires"
//! clause): structure types its player must own, each at least once.
//!
//! - **What counts** ([`counts`]; `core/tech_and_research.md > Tech Prerequisites > Counting Rule`): "A structure only
//!   counts toward a TechPrerequisite once it is **fully constructed and present on the map**": an object of the
//!   player's that is in operation (`Object::is_operational`): not a site (a Syndicate expansion "must have finished
//!   construction", a worker's building alike), nor transforming (a Colonists building: "the Warp In transformation
//!   must have completed"). A GDO building ready in its constructor's slot is no object yet ("A completed building
//!   sitting in a constructor's ReadyToPlace slot does not count"). A neutral expansion is no one's (slice M4-H), so it
//!   counts for no player. For the Syndicate, "satisfied by a fully constructed structure anywhere in the Tunnel
//!   Network" (`factions/syndicate/structures/README.md > Tunnel Expansions > Expansion Requirements`): any of its own,
//!   whichever Tunnel built it.
//! - **When** (`core/tech_and_research.md > Tech Prerequisites > Check Timing`): "checked only at the moment
//!   construction is **started** (queued at the constructor, Warp In begun, expansion construction begun, etc.)", and a
//!   unit's "only when the unit is queued" (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`): by
//!   the command gate ([`Sim::check_requires`]), for a Train (`sim/producers.rs`), a constructor's Construct
//!   (`sim/construction.rs`), a Warp In (`sim/transform.rs`), an Expand (`Sim::can_expand`) and a worker's Build
//!   (`sim/builds.rs`, as the order is given and paid). A refusal names the first type missing, [`Reject::Requires`],
//!   which the panel greys the entry with ("Requires an Optics Lab"): "Options whose TechPrerequisite is not met are
//!   unavailable in the build menu".
//! - **Losing one** (`core/tech_and_research.md > Tech Prerequisites > Losing a Prerequisite`): nothing checks again,
//!   so "units already queued or in production continue and complete normally", and "Constructions already started ...
//!   continue"; only new ones are refused.

use crate::defs::{ObjectKind, Recipe};
use crate::ids::PlayerId;
use crate::object::Object;
use crate::sim::{Reject, Sim};

/// Whether `object` counts toward `player`'s prerequisites of its kind (module docs): the player's, and in operation.
pub fn counts(object: &Object, player: PlayerId) -> bool {
    object.owner == Some(player) && object.is_operational()
}

/// A type's TechPrerequisite (`StructureDef::requires`): empty for a type that has none, a unit's among them.
pub fn structure_requires(kind: ObjectKind) -> &'static [ObjectKind] {
    kind.def()
        .structure
        .as_ref()
        .map_or(&[], |structure| structure.requires)
}

/// What starting `recipe` requires: the entry's own (`Recipe::requires`), then its type's TechPrerequisite
/// ([`structure_requires`]), which a building has wherever it is built.
pub fn recipe_requires(recipe: &Recipe) -> impl Iterator<Item = ObjectKind> + use<> {
    let (own, its_type) = (recipe.requires, structure_requires(recipe.kind));
    own.iter().chain(its_type).copied()
}

impl Sim {
    /// `core/tech_and_research.md > Tech Prerequisites`: whether `player` meets the requirement `kind`, owning at least
    /// one object of the kind that counts ([`counts`]: complete, and the player's).
    pub fn requirement_met(&self, player: PlayerId, kind: ObjectKind) -> bool {
        self.objects().any(|o| o.kind == kind && counts(o, player))
    }

    /// The first of `requires` that `player` doesn't meet ([`Sim::requirement_met`]), if any.
    pub fn unmet_requirement(
        &self,
        player: PlayerId,
        requires: impl IntoIterator<Item = ObjectKind>,
    ) -> Option<ObjectKind> {
        requires.into_iter().find(|&kind| !self.requirement_met(player, kind))
    }

    /// The command gate's check of `requires` for `player` (module docs): [`Reject::Requires`] the first type it
    /// doesn't meet.
    pub(crate) fn check_requires(
        &self,
        player: PlayerId,
        requires: impl IntoIterator<Item = ObjectKind>,
    ) -> Result<(), Reject> {
        match self.unmet_requirement(player, requires) {
            Some(kind) => Err(Reject::Requires(kind)),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
