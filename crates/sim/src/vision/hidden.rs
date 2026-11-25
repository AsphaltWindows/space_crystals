//! Hidden objects (plan decision T32): what an owned underground object keeps from the other players
//! (`factions/syndicate/structures.md > Tunnel Expansions`: "Expansions are invisible to enemies without detection and
//! can be walked over by surface units").
//!
//! - **The rule** ([`hidden_from`]): an object of an underground structure type (`StructureDef::underground`, the
//!   Hideout so far) that has an owner is hidden from every other player. Its owner always knows it, and a neutral one
//!   is hidden from nobody (`factions/syndicate/tunnels.md > Losing Tunnels`: "Neutral expansions are **no longer
//!   hidden** (visible to everyone, no detection needed)"). It is a rule over objects, not over structure types: the
//!   DrillUnits underground (design change 0219: "hidden from other players unless revealed by a detector") will add an
//!   arm for units, and detection a "revealed" term, here.
//! - **Who asks it**: whether a player sees an object ([`PlayerVision::object_box_state`]: never Visible while hidden
//!   from the player), which every question of the kind comes to: the targets of the behaviours
//!   (`orders::world::World::sees`), of combat (`combat::phases`: `systems/combat.md > ValidTarget` 2) and of the
//!   command gate; the records (`super::memory`: none is taken of a hidden object); and the client's `Sight`. The
//!   snapshots that look at objects without holding them, combat's targets and the behaviours' views, take each
//!   object's [`Hiding`] with them. `Sim::hidden_from` asks it for tools and tests.
//! - **Who doesn't**: damage resolution (`combat::damage`). Area damage still hits hidden objects
//!   (`systems/combat.md > AoEDamage`, design change 0262, keeping 0218: "Syndicate expansions ... take their share
//!   of any blast over them, even unseen"); a hit on the tick's list lands whatever its target's owner sees.
//!
//! [`PlayerVision::object_box_state`]: super::PlayerVision::object_box_state

use crate::ids::PlayerId;
use crate::object::Object;
use crate::sim::Sim;

/// Whom an object is hidden from, by the rule of [`hidden_from`]: every player but its owner, or nobody. Taken from the
/// object ([`Hiding::of`]) by the snapshots that look at objects without holding them, so that they ask what the
/// objects themselves would.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Hiding {
    /// The owner of a hidden object, the one player it isn't hidden from; `None` while it is hidden from nobody.
    owner: Option<PlayerId>,
}

impl Hiding {
    /// Whom `object` is hidden from now.
    pub fn of(object: &Object) -> Self {
        let underground = (object.def().structure.as_ref()).is_some_and(|structure| structure.underground);
        Self {
            owner: object.owner.filter(|_| underground),
        }
    }

    /// Whether it keeps its object from `player`.
    pub fn hides_from(self, player: PlayerId) -> bool {
        self.owner.is_some_and(|owner| owner != player)
    }
}

/// Plan decision T32: whether `object` is hidden from `player` (module docs): an owned underground structure, from
/// every player but its owner.
pub fn hidden_from(object: &Object, player: PlayerId) -> bool {
    Hiding::of(object).hides_from(player)
}

impl Sim {
    /// Plan decision T32: whether `object` is hidden from `player` ([`hidden_from`]: an owned underground structure,
    /// from every player but its owner), the one rule that vision, the records, targeting and the client follow, and
    /// damage doesn't (design changes 0218 and 0262).
    pub fn hidden_from(&self, object: &Object, player: PlayerId) -> bool {
        hidden_from(object, player)
    }
}
