//! Rally points — `factions/gdo/structures/barracks.md > RallyPoint behavior`,
//! `systems/controls/selection_and_hud.md > Production Building Default Right-Click`,
//! `systems/controls/selection_and_hud.md > Rally points on enemy objects`, `systems/controls/selection_and_hud.md >
//! What a rally point makes a new unit do` (design changes 0086, 0088, 0136 and 0139).
//!
//! - **The new unit's order** (design change 0136): "A unit that spawns with a rally point set **always** gets its
//!   **own default right-click command** resolved against the rally target [...]. No per-building lists override
//!   this." That is `Sim::rally_order`, the unit's default order (`orders::resolution`) on the target as its owner
//!   knows it: a Drone rallied on a patch gathers it and one rallied on an enemy follows it (it is unarmed), a
//!   Peacekeeper rallied on an enemy attacks it. "If RallyPoint is None, the unit spawns with no command."
//! - **On an object** ([`follow`], at the end of each tick): an enemy unit leaving its owner's sight turns the rally
//!   into a ground rally at its last-seen position (design change 0088); a structure (or a resource) stays the rally
//!   out of sight, at its last-known position, until its owner sees its place without it (design change 0139: "A
//!   rally point on an enemy structure stays on it after it leaves sight", `Sim::follow_targets`).

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::vision::Vision;

/// `BarracksInstanceState > RallyPoint: Coordinates | ObjectInstance` (`None` is no `Rally` at all).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Rally {
    /// A ground point, space units.
    Point(Vec2),
    /// An object, and where its owner last saw it: the ground point an enemy unit's rally turns into when the unit
    /// leaves the owner's sight (design change 0088), a remembered structure's last-known position (design change
    /// 0139).
    Object { id: ObjectId, at: Vec2 },
}

impl Rally {
    /// The ground point the rally shows at: the point, or where the object was last seen.
    pub fn point(&self) -> Vec2 {
        match *self {
            Rally::Point(p) | Rally::Object { at: p, .. } => p,
        }
    }
}

/// Whether `target` is an enemy of `player`: another player's object (no teams yet, so no allies; neutral objects
/// are nobody's enemy).
pub fn is_enemy(player: Option<PlayerId>, target: &Object) -> bool {
    target.owner.is_some() && target.owner != player
}

/// `systems/controls/selection_and_hud.md > Rally points on enemy objects` (design change 0088), at the end of
/// each tick, on the vision the tick leaves: "A rally point on an enemy object only follows that object while the
/// owner can see it. As soon as the enemy object leaves the owner's sight, the rally point turns into a ground
/// rally point at its last-seen position." While the owner sees the object, the rally notes where it is.
///
/// Design change 0139 narrows it to enemy units: an object the owner remembers (a structure or a resource) stays
/// the rally out of sight, at its last-known position, destroyed or not, until the owner sees its place without it
/// (`Sim::follow_targets` resets it then, before this runs). So when an object is removed, a rally still on it is
/// one whose owner saw it at the end of the tick before, or one on an own or neutral object: those reset to None
/// (`Production::forget`); one on a remembered object stays; and "an enemy dying unseen in the fog doesn't change
/// the rally point", which is a ground point by then, or still on the remembered structure.
pub(crate) fn follow(objects: &mut BTreeMap<ObjectId, Object>, vision: &Vision) {
    let mut changed: Vec<(ObjectId, Rally)> = Vec::new();
    for o in objects.values() {
        let Some(Rally::Object { id, at }) = o.production.as_ref().and_then(|p| p.rally) else {
            continue;
        };
        let Some(target) = objects.get(&id) else {
            continue;
        };
        let sight = o.owner.and_then(|owner| vision.player(owner));
        let rally = if sight.is_some_and(|sight| sight.sees(target)) {
            Rally::Object { id, at: target.pos }
        } else if is_enemy(o.owner, target) && sight.is_none_or(|sight| sight.remembered_record(id).is_none()) {
            Rally::Point(at)
        } else {
            // A remembered object stays the rally out of sight (design change 0139), and so does an own or neutral
            // one (a remembered Supply Delivery Station).
            continue;
        };
        if rally != (Rally::Object { id, at }) {
            changed.push((o.id, rally));
        }
    }
    for (producer, rally) in changed {
        if let Some(production) = objects.get_mut(&producer).and_then(|o| o.production.as_mut()) {
            production.rally = Some(rally);
        }
    }
}

impl StateHash for Rally {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Rally::Point(p) => {
                h.tag(0);
                p.hash(h);
            }
            Rally::Object { id, at } => {
                h.tag(1);
                id.hash(h);
                at.hash(h);
            }
        }
    }
}
