//! Rally points — `factions/gdo/structures.md > Barracks > RallyPoint behavior`,
//! `systems/controls/selection_and_hud.md > Production Building Default Right-Click`, `> Rally points on enemy
//! objects` (design changes 0086 and 0088).
//!
//! "When a unit finishes production and spawns, if RallyPoint is set, it is issued the default right-click command
//! resolved against the RallyPoint target. Rally on ground -> Move to location, rally on enemy -> Attack, rally on
//! friendly/neutral object -> Move to object. If RallyPoint is None, the unit spawns with no command."

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::economy::gathering::gatherable;
use crate::hash::{StateHash, StateHasher};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::vision::Vision;

/// `BarracksInstanceState > RallyPoint: Coordinates | ObjectInstance` (`None` is no `Rally` at all).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Rally {
    /// A ground point, space units.
    Point(Vec2),
    /// An object, and where its owner last saw it: the ground point an enemy's rally turns into when the enemy
    /// leaves the owner's sight (design change 0088).
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

/// `> RallyPoint behavior`: the order a unit of `unit`'s kind gets as it leaves a producer of `owner` with this
/// rally point: the default right-click command on the rally's target. Ground: Move there. An enemy object:
/// Attack, if the unit can attack and the object can be destroyed, else Move to it. Any other object: Move to it
/// (Follow, for a unit). `None` if the object is gone.
///
/// A worker (`is_worker`) rallied on a Space Crystals patch it can gather gets Gather, its default right-click
/// there (`factions/colonists/units.md > Drone > Right-Click Resolution`; question Q0037 point 9, its proposal:
/// `factions/colonists/structures.md > Robo Portal > RallyPoint behavior` also lists "Move to object" for it).
pub(crate) fn order_for(
    rally: &Rally,
    owner: Option<PlayerId>,
    can_attack: bool,
    is_worker: bool,
    objects: &BTreeMap<ObjectId, Object>,
) -> Option<OrderKind> {
    match *rally {
        Rally::Point(p) => Some(OrderKind::Move(Target::Point(p))),
        Rally::Object { id, .. } => {
            let target = objects.get(&id)?;
            Some(if is_worker && gatherable(objects, id) {
                OrderKind::Gather(id)
            } else if is_enemy(owner, target) && can_attack && target.def().destructible {
                OrderKind::Attack(id)
            } else {
                OrderKind::Move(Target::Object(id))
            })
        }
    }
}

/// `systems/controls/selection_and_hud.md > Rally points on enemy objects` (design change 0088), at the end of
/// each tick, on the vision the tick leaves: "A rally point on an enemy object only follows that object while the
/// owner can see it. As soon as the enemy object leaves the owner's sight, the rally point turns into a ground
/// rally point at its last-seen position." While the owner sees the object, the rally notes where it is.
///
/// So when an object is removed, a rally still on it is one whose owner saw it at the end of the tick before, or
/// one on an own or neutral object: those reset to None (`Production::forget`), and "an enemy dying unseen in the
/// fog doesn't change the rally point", which is a ground point by then.
pub(crate) fn follow(objects: &mut BTreeMap<ObjectId, Object>, vision: &Vision) {
    let mut changed: Vec<(ObjectId, Rally)> = Vec::new();
    for o in objects.values() {
        let Some(Rally::Object { id, at }) = o.production.as_ref().and_then(|p| p.rally) else {
            continue;
        };
        let Some(target) = objects.get(&id) else {
            continue;
        };
        let seen = o.owner.is_some_and(|owner| vision.sees(owner, target));
        let rally = if seen {
            Rally::Object { id, at: target.pos }
        } else if is_enemy(o.owner, target) {
            Rally::Point(at)
        } else {
            // An own or neutral object stays the rally out of sight too (a remembered Supply Delivery Station).
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
