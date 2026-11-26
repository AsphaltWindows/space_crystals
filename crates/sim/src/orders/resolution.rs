//! A unit's default command against a target — `systems/controls/interface.md > RightClickResolution` (design
//! changes 0062, 0063 and 0074), the one rule behind both the right-click and the rally point (design change 0136:
//! `systems/controls/interface.md > What a rally point makes a new unit do`: "A unit that spawns with a rally
//! point set **always** gets its **own default right-click command** resolved against the rally target [...]. No
//! per-building lists override this").
//!
//! [`resolve`] is the rule, on what the unit can do ([`Abilities`]) and what its owner knows of the target ([`Aim`]):
//! the sim builds both from the game (`Sim::default_order`, by the owner's [`crate::targets::TargetView`]: a target it
//! sees or remembers, design change 0139), the client from what it shows (`command_ui::resolution`), so the click,
//! its pointer and a rally point all come out of the same function.
//!
//! - **The ground**: Move there (`systems/controls/interface.md > BasicCombatUnitInterfaceState >
//!   RightClickResolution`, `factions/colonists/units/drone.md > Right-Click Resolution`: "Ground → Move").
//! - **A worker** (the Drone, `factions/colonists/units/drone.md > Right-Click Resolution`): a Space Crystals patch it
//!   can gather gives Gather ("Space Crystal patch → Gather"; a patch under a mining structure is no patch to it,
//!   question Q0037 point 11); its owner's drop-off that takes loads gives Drop Off while it carries a load ("Own
//!   Egress Node (carrying resources) → Drop off resources"); without a load, a Move there (design change 0140). A
//!   worker that gathers Supplies (the Agent, `factions/syndicate/units/agent.md > Right-Click Resolution`: "Supply
//!   Delivery Station → Gather supplies"; slice M4-I) gets Gather on a Supply Delivery Station too, whatever it carries
//!   (design change 0256 point 4: it "goes straight to the station").
//! - **An own Tunnel** whose tier lets the unit through gives Enter (`systems/controls/commands.md > Enter`: "any
//!   Syndicate unit, when the target Tunnel's tier is sufficient for the unit's base category";
//!   `factions/syndicate/units/agent.md > Right-Click Resolution`: "Own Tunnel (not carrying resources) → Enter";
//!   slice M4-C, `crate::network`), after a worker's Drop Off there ("Own Tunnel (carrying resources) → Drop off
//!   resources"). So does **an own building that holds units** of the unit's kind, in operation (slice M5-F1,
//!   `crate::holding`; `factions/cults/units/recruit.md > Right-Click Resolution`: "Own Armory → Enter Armory";
//!   `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only"), whether or not its pool has
//!   room, which the unit finds out as it gets there (Placeholder (Q0104 point 2)).
//! - **An own building under construction** that the unit's type builds from outside gives Assist (slice M5-E;
//!   `factions/cults/units/recruit.md > Right-Click Resolution`: "In-progress own building → Assist Construction";
//!   `OrderKind::Assist`), after Enter: an Armory under construction takes no one in.
//! - **An enemy** (another player's object) that the unit can attack gives Attack: it has an attack whose
//!   TargetDomain allows the object's domain (`systems/combat.md > ValidTarget` 3), and the object is destructible.
//! - **Everything else** gives a Move to the object (`systems/controls/command_behaviors.md > MovingToObject`), which
//!   follows a unit and moves to anything else: an own or neutral object (design change 0059: "Right-click never"
//!   attacks them), and an enemy the unit can't attack (design change 0062: "Units that can't carry out the resolved
//!   command [...] **move to or follow the target instead**"), an unarmed Drone's included ("Enemy or other object →
//!   Move to it: **Follow** if it is a unit, Move to its location otherwise", design change 0136), or one its type
//!   takes no Attack order against, a Peacekeeper (Shielded) (design change 0074; design change 0202: "While raising
//!   (already Shielded): no Attack; a right-click follows"). A Peacekeeper whose shield is on its way down attacks
//!   ("accepted and held, and the attack starts once the shield is fully down").
//!
//! The Peacekeeper, the Guard, the Sabre and the Rifle Legionnaire resolve by the BasicCombatUnit table, the Drone by
//! its own; both are this one rule (`tests/default_order.rs` has the table of unit kinds against targets). The rules
//! on an object are data, tried in their order ([`RULES`], [`Rule`]; slice M5-J1), so that a new default order is a
//! row of the table: slice N8 adds the Recruit's Repair after Enter.

use glam::Vec2;

use crate::combat::domain_allows;
use crate::construction::site::assists;
use crate::defs::{Domain, ObjectKind, TargetDomain, Transit};
use crate::economy::gathering::worker_def;
use crate::ids::ObjectId;
use crate::network;
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::riot_shield;

/// What a unit can do, as its default command reads it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Abilities {
    /// Its type: what a building that holds units takes in (`Aimed::accepts`).
    pub kind: ObjectKind,
    /// The TargetDomain of its attack (`systems/combat.md > AttackAttributes`); `None` for a unit without one.
    pub attack: Option<TargetDomain>,
    /// Its type takes Attack orders: not a Peacekeeper (Shielded), whose shield is up or on its way up
    /// (`riot_shield::takes_attack_orders`; design change 0202: "which orders the unit accepts follows its new type
    /// from the press"). A Peacekeeper whose shield is on its way down takes them, and attacks once it is down.
    pub may_attack: bool,
    /// It gathers (`UnitDef::worker`): the Drone.
    pub worker: bool,
    /// A worker that carries a load (`economy::gathering::Worker::load`).
    pub carrying: bool,
    /// A worker of a type that gathers Supplies (`WorkerDef::supplies`: the Agent; slice M4-I).
    pub gathers_supplies: bool,
    /// The class of its base for a Tunnel's transit (`factions/syndicate/tunnels.md > Tunnel Network > Transit Tier
    /// Requirements`, `Transit::of`).
    pub transit: Transit,
}

impl Abilities {
    /// What `o` can do, if it takes orders (a unit, `Object::orders`).
    pub fn of(o: &Object) -> Option<Self> {
        o.orders.as_ref()?;
        Some(Self {
            kind: o.kind,
            transit: Transit::of(o.def().unit.as_ref()?.base),
            attack: o
                .def()
                .unit
                .as_ref()
                .and_then(|unit| unit.attack.as_ref())
                .map(|attack| attack.target_domain),
            may_attack: riot_shield::takes_attack_orders(o.kind),
            worker: o.worker.is_some(),
            carrying: o.worker.is_some_and(|worker| worker.load > 0),
            gathers_supplies: o.worker.is_some() && worker_def(o.kind).is_some_and(|def| def.supplies.is_some()),
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
    /// It is a Supply Delivery Station (`core/map.md > SupplyDeliveryStation`), where the workers that gather Supplies
    /// pick them up (slice M4-I).
    pub station: bool,
    /// It is a drop-off of the unit's owner's that takes loads now (`StructureDef::drop_off`: an Egress Node, warped
    /// in: design change 0116).
    pub drop_off: bool,
    /// The classes of unit it lets into the Tunnel Network: its tier's Transit if it is a Tunnel of the unit's owner's
    /// in operation (`network::is_exit`), none for anything else.
    pub transit: &'static [Transit],
    /// The kinds of unit it takes in: its type's (`HoldsDef::accepts`) if it is a building of the unit's owner's that
    /// holds units, in operation (`holding::admits`; slice M5-F1), none for anything else.
    pub accepts: &'static [ObjectKind],
    /// (M5-E) Its kind, if it is a site of the unit's owner's that its builders build from outside
    /// (`Object::is_built_from_outside`): a Cults building under construction, which the units that assist it take an
    /// Assist on (`construction::site::assists`). `None` for anything else.
    pub site: Option<ObjectKind>,
}

/// What a unit's default command is resolved against.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Aim {
    /// The ground, at this point (space units).
    Ground(Vec2),
    Object(Aimed),
}

/// `systems/controls/interface.md > RightClickResolution` for one unit (module docs): the order a unit that can do
/// `unit` gets against `aim`. Every unit can move, so there is always one: on an object, the order of the first of the
/// [`RULES`] that applies, else a Move to it.
pub fn resolve(unit: Abilities, aim: Aim) -> OrderKind {
    let target = match aim {
        Aim::Ground(point) => return OrderKind::Move(Target::Point(point)),
        Aim::Object(target) => target,
    };
    (RULES.iter())
        .find_map(|rule| rule.order(&unit, &target))
        .unwrap_or(OrderKind::Move(Target::Object(target.id)))
}

/// A rule of the resolution on an object ([`RULES`], module docs): the condition under which a unit gets one order
/// against it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rule {
    /// A worker on a patch it can gather: Gather.
    Gather,
    /// A worker that gathers Supplies on a Supply Delivery Station: Gather.
    GatherSupplies,
    /// A worker that carries a load on its owner's drop-off: Drop Off.
    DropOff,
    /// An own Tunnel whose tier lets the unit through, or an own building in operation that takes its kind in: Enter.
    Enter,
    /// An own site that the unit's type builds from outside: Assist.
    Assist,
    /// An enemy the unit can attack, its type taking Attack orders: Attack.
    Attack,
}

/// (M5-J1) The rules of the resolution on an object, in the order they are tried, as data: the first that applies
/// gives the order, and a Move to the object if none does (module docs). A new default order is a row in its place:
/// slice N8's Repair, after [`Rule::Enter`] and before [`Rule::Attack`] (`factions/cults/units/recruit.md >
/// Right-Click Resolution`, design change 0322: "Damaged own building or vehicle (completed) → Repair, **unless the
/// building can be entered** by the selection: then **Enter** wins").
pub const RULES: &[Rule] = &[
    Rule::Gather,
    Rule::GatherSupplies,
    Rule::DropOff,
    Rule::Enter,
    Rule::Assist,
    Rule::Attack,
];

impl Rule {
    /// The order a unit that can do `unit` gets against `target` by this rule, if it applies.
    pub fn order(self, unit: &Abilities, target: &Aimed) -> Option<OrderKind> {
        let id = target.id;
        match self {
            Rule::Gather => (unit.worker && target.gatherable).then_some(OrderKind::Gather(id)),
            Rule::GatherSupplies => (unit.gathers_supplies && target.station).then_some(OrderKind::Gather(id)),
            Rule::DropOff => (unit.worker && unit.carrying && target.drop_off).then_some(OrderKind::DropOff(id)),
            Rule::Enter => (network::passes(target.transit, unit.transit) || target.accepts.contains(&unit.kind))
                .then_some(OrderKind::Enter(id)),
            Rule::Assist => (target.site.is_some_and(|kind| assists(unit.kind, kind))).then_some(OrderKind::Assist(id)),
            Rule::Attack => {
                let can_attack =
                    target.destructible && unit.attack.is_some_and(|domain| domain_allows(domain, target.domain));
                (target.enemy && can_attack && unit.may_attack).then_some(OrderKind::Attack(id))
            }
        }
    }
}
