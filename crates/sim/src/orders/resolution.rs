//! A unit's default command against a target — `systems/controls/selection_and_hud.md > RightClickResolution` (design
//! changes 0062, 0063 and 0074), the one rule behind both the right-click and the rally point (design change 0136:
//! `> What a rally point makes a new unit do`: "A unit that spawns with a rally point set **always** gets its **own
//! default right-click command** resolved against the rally target [...]. No per-building lists override this").
//!
//! [`resolve`] is the rule, on what the unit can do ([`Abilities`]) and what its owner knows of the target ([`Aim`]):
//! the sim builds both from the game (`Sim::default_order`, by the owner's [`crate::targets::TargetView`]: a target it
//! sees or remembers, design change 0139), the client from what it shows (`command_ui::resolution`), so the click,
//! its pointer and a rally point all come out of the same function.
//!
//! - **The ground**: Move there (`> BasicCombatUnitInterfaceState > RightClickResolution`, `factions/colonists/units.md
//!   > Drone > Right-Click Resolution`: "Ground → Move").
//! - **A worker** (the Drone, `> Drone > Right-Click Resolution`): a Space Crystals patch it can gather gives Gather
//!   ("Space Crystal patch → Gather"; a patch under a mining structure is no patch to it, question Q0037 point 11); its
//!   owner's drop-off that takes loads gives Drop Off while it carries a load ("Own Egress Node (carrying resources) →
//!   Drop off resources"); without a load, a Move there (design change 0140).
//! - **An enemy** (another player's object) that the unit can attack gives Attack: it has an attack whose
//!   TargetDomain allows the object's domain (`systems/combat.md > ValidTarget` 3), and the object is destructible.
//! - **Everything else** gives a Move to the object (`systems/controls/behaviors.md > MovingToObject`), which follows
//!   a unit and moves to anything else: an own or neutral object (design change 0059: "Right-click never" attacks
//!   them), and an enemy the unit can't attack (design change 0062: "Units that can't carry out the resolved command
//!   [...] **move to or follow the target instead**"), an unarmed Drone's included ("Enemy or other object → Move
//!   to it: **Follow** if it is a unit, Move to its location otherwise", design change 0136), or one it may not
//!   attack now, its Riot Shield up (design change 0074).
//!
//! The Peacekeeper, the Guard, the Sabre and the Rifle Legionnaire resolve by the BasicCombatUnit table, the Drone by
//! its own; both are this one rule (`tests/default_order.rs` has the table of unit kinds against targets).

use glam::Vec2;

use crate::combat::domain_allows;
use crate::defs::{Domain, TargetDomain};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::riot_shield;

/// What a unit can do, as its default command reads it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Abilities {
    /// The TargetDomain of its attack (`systems/combat.md > AttackAttributes`); `None` for a unit without one.
    pub attack: Option<TargetDomain>,
    /// Its mode lets it attack now: its Riot Shield isn't up, nor on its way up or down
    /// (`riot_shield::allows_attack`).
    pub may_attack: bool,
    /// It gathers (`UnitDef::worker`): the Drone.
    pub worker: bool,
    /// A worker that carries a load (`economy::gathering::Worker::load`).
    pub carrying: bool,
}

impl Abilities {
    /// What `o` can do, if it takes orders (a unit, `Object::orders`).
    pub fn of(o: &Object) -> Option<Self> {
        o.orders.as_ref()?;
        Some(Self {
            attack: o
                .def()
                .unit
                .as_ref()
                .and_then(|unit| unit.attack.as_ref())
                .map(|attack| attack.target_domain),
            may_attack: riot_shield::allows_attack(o),
            worker: o.worker.is_some(),
            carrying: o.worker.is_some_and(|worker| worker.load > 0),
        })
    }
}

/// An object a unit's default command is resolved against, as the unit's owner knows it (design change 0139: one it
/// sees, or one it remembers, by its record).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aimed {
    pub id: ObjectId,
    /// It belongs to another player than the unit's owner (no teams yet, so no allies; nobody's objects are nobody's
    /// enemies).
    pub enemy: bool,
    /// Its type is destructible (`systems/controls/commands.md > Attack`: "a specific destructible object instance").
    pub destructible: bool,
    /// The domain it is targeted in (`combat::object_domain`): a unit's base's, Ground for anything else.
    pub domain: Domain,
    /// It is a Space Crystals patch a worker can gather: no mining structure on it, as far as the owner knows
    /// (question Q0037 point 11).
    pub gatherable: bool,
    /// It is a drop-off of the unit's owner's that takes loads now (`StructureDef::drop_off`: an Egress Node, warped
    /// in: design change 0116).
    pub drop_off: bool,
}

/// What a unit's default command is resolved against.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Aim {
    /// The ground, at this point (space units).
    Ground(Vec2),
    Object(Aimed),
}

/// `> RightClickResolution` for one unit (module docs): the order a unit that can do `unit` gets against `aim`. Every
/// unit can move, so there is always one.
pub fn resolve(unit: Abilities, aim: Aim) -> OrderKind {
    let target = match aim {
        Aim::Ground(point) => return OrderKind::Move(Target::Point(point)),
        Aim::Object(target) => target,
    };
    if unit.worker && target.gatherable {
        return OrderKind::Gather(target.id);
    }
    if unit.worker && unit.carrying && target.drop_off {
        return OrderKind::DropOff(target.id);
    }
    let can_attack = target.destructible && unit.attack.is_some_and(|domain| domain_allows(domain, target.domain));
    if target.enemy && can_attack && unit.may_attack {
        return OrderKind::Attack(target.id);
    }
    OrderKind::Move(Target::Object(target.id))
}
