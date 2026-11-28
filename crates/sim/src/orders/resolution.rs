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
//!   resources"), but to a unit whose type is one way (`network::one_way`;
//!   `factions/syndicate/units/prism.md > One way only`: it "can't get back into the Tunnel Network"), which Moves to it
//!   as to any other object. So does **an own building that holds units** of the unit's kind, in operation (slice M5-F1,
//!   `crate::holding`; `factions/cults/units/recruit.md > Right-Click Resolution`: "Own Armory → Enter Armory";
//!   `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only"), whether or not its pool has
//!   room, which the unit finds out as it gets there (design change 0347 point 2,
//!   `factions/cults/structures/README.md > Entering pool buildings`: "a unit arriving when the building is full has
//!   its Enter **end** and idles beside the building"; design change 0378: "If the building fills up later while the
//!   Recruits are walking over, the normal Full-building rule applies"), but for one full and damaged that the unit's
//!   type repairs, which it Repairs (below). (M6-CM) Not for an illusion at a building that refuses illusions
//!   ([`Aimed::refuses_illusions`]: the Altar, `factions/cults/units/disciple.md > Spells > Mirage`: "Illusions can't
//!   be augmented at the Altar themselves", `crate::mirage`), which it follows, an Enter there being certain to fail.
//!   (M6-ST1) So does **an own carrier** that loads now and takes the unit's base in ([`Aimed::carrier`],
//!   `crate::carry`; `factions/syndicate/units/subterranean_transport.md > Placeholder stats`: "**Loading**:
//!   right-click the transport with infantry selected (Enter); it must be above ground"), by the same rule as a
//!   building.
//! - **An own building under construction** that the unit's type builds from outside gives Assist (slice M5-E;
//!   `factions/cults/units/recruit.md > Right-Click Resolution`: "In-progress own building → Assist Construction";
//!   `OrderKind::Assist`), after Enter: an Armory under construction takes no one in (design change 0347 point 4:
//!   "a building under construction **can't be entered**; a right-click on it Assists").
//! - **A damaged own building or vehicle** that the unit's type repairs gives Repair (slice M5-N8, `crate::repair`;
//!   `factions/cults/units/recruit.md > Right-Click Resolution`: "Damaged own building or vehicle (completed) → Repair,
//!   **unless the building can be entered** by the selection: then **Enter** wins";
//!   `factions/syndicate/units/agent.md > Right-Click Resolution`: "Own Tunnel (not carrying resources) → Enter (also
//!   when damaged: Enter wins over Repair)", "Damaged own Laser Turret → Repair"; `OrderKind::Repair`), after Enter and
//!   Assist. (M5-S2b) **Unless it is full**: Enter gives way to it on a building that holds units with no room left at
//!   the moment of the click ([`Aimed::full`]; design change 0378, answering Q0140, `factions/cults/units/recruit.md >
//!   Right-Click Resolution`: "**Except when it is full** [...]: a damaged enterable building that is **full at the
//!   moment of the click** resolves to **Repair** (the pointer shows Repair). This applies to every building that
//!   holds units").
//! - (M6-GH1) **An own Airfield** in operation gives an aircraft that rearms at one (`crate::aircraft`) a Rebase to it
//!   (`systems/mechanics/aircraft_landing.md > Assigned base`: "Input: select the aircraft and **right-click the other
//!   Airfield**"), whether or not it has room, which the gate judges (`Reject::AirfieldFull`); its own Airfield a
//!   Return to Base (`systems/mechanics/aircraft_landing.md > Assigned base`: "Right-clicking its own Airfield
//!   does the same").
//! - **A Supply Chopper** (`crate::supply_run`; `factions/gdo/units/supply_chopper.md >
//!   ObjectInterfaceState[SupplyChopper]`): a Supply Delivery Station gives Pick Up Supplies ("Right-click
//!   SupplyDeliveryStation: issues PickUpSupplies command"); an own Supply Tower in operation Drop Off Supplies while
//!   it carries Supplies, Attach to Tower while it doesn't ("Right-click own SupplyTower (when carrying supplies):
//!   issues DropOffSupplies command", "(when not carrying supplies): issues AttachToTower command"), but for a tower
//!   another chopper is attached to, which takes none of them (the gate's `Reject::TowerTaken`): a Move to it, as for
//!   "other friendly/neutral Object".
//! - **An enemy** (another player's object) that the unit can attack gives Attack: it has an attack whose
//!   TargetDomain allows the object's domain (`systems/combat.md > ValidTarget` 3; [`Abilities::attacks`], by
//!   `combat::can_hit`, the gate's and the client's check too), and the object is destructible.
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
//! row of the table: slice M5-N8 added the repairers' Repair after Enter and Assist.

use glam::Vec2;

use crate::construction::site::assists;
use crate::defs::{CarrierDef, Domain, ObjectKind, TargetDomain, Transit};
use crate::economy::gathering::worker_def;
use crate::ids::ObjectId;
use crate::network;
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::repair::repairs;
use crate::{aircraft, carry, combat};

/// What a unit can do, as its default command reads it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Abilities {
    /// Its type: what a building that holds units takes in (`Aimed::accepts`).
    pub kind: ObjectKind,
    /// The TargetDomain of its attack (`systems/combat.md > AttackAttributes`); `None` for a unit without one.
    pub attack: Option<TargetDomain>,
    /// Its type takes Attack orders (`combat::takes_attack_orders`): not a Peacekeeper (Shielded), whose shield is up or
    /// on its way up (`riot_shield::takes_attack_orders`; design change 0202: "which orders the unit accepts follows
    /// its new type from the press"), nor (M5-N4a) a unit without a weapon. A Peacekeeper whose shield is on its way
    /// down takes them, and attacks once it is down.
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
    /// (M6-GH1) If it is an aircraft that rearms at an Airfield (`crate::aircraft`), the Airfield it is assigned to:
    /// `Some(None)` while it is unassigned. `None` for every other unit.
    pub aircraft: Option<Option<ObjectId>>,
    /// (M6-CM) It is an illusion (`crate::mirage`), which a building that refuses illusions doesn't take in
    /// ([`Aimed::refuses_illusions`]). Only its owner, whose unit it is, resolves its default command.
    pub illusion: bool,
    /// If it is a Supply Chopper (`crate::supply_run`), what it carries and the tower it is attached to.
    /// `None` for every other unit.
    pub chopper: Option<Chopper>,
}

/// A Supply Chopper's part of what it can do (`Abilities::chopper`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Chopper {
    /// It carries Supplies (`supply_run::carried`).
    pub carrying: bool,
    /// The Supply Tower it is attached to (`supply_run::attached_to`).
    pub attached: Option<ObjectId>,
}

impl Abilities {
    /// (M6-S6a) Whether the unit can attack a target in `domain`, as its owner knows it ([`Aimed::domain`]): it may
    /// attack now, and its weapon can hit that domain (`combat::can_hit`, design change 0461).
    pub fn attacks(&self, domain: Domain) -> bool {
        self.may_attack && combat::can_hit(self.attack, domain)
    }

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
            // (M6-GH1) Not an aircraft out of ammo (`crate::aircraft::armed`).
            may_attack: combat::takes_attack_orders(o.kind) && aircraft::armed(o),
            worker: o.worker.is_some(),
            carrying: o.worker.is_some_and(|worker| worker.load > 0),
            gathers_supplies: o.worker.is_some() && worker_def(o.kind).is_some_and(|def| def.supplies.is_some()),
            aircraft: o.aircraft.map(|state| state.base),
            illusion: crate::mirage::is_illusion(o),
            chopper: o.chopper.as_ref().map(|state| Chopper {
                carrying: state.supplies() > 0,
                attached: state.attached,
            }),
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
    /// (M5-S2b) It is such a building ([`Aimed::accepts`]) with no room left now: its capacity taken by everyone
    /// inside, reserved units included, as its pool counts it at the door (`Sim::room`; design change 0378, answering
    /// Q0140: "fullness, counting reserved units, is checked at click time"). (M6-ST1) Or such a carrier
    /// ([`Aimed::carrier`]) with no place free. `false` for anything else.
    pub full: bool,
    /// (M5-E) Its kind, if it is a site of the unit's owner's that its builders build from outside
    /// (`Object::is_built_from_outside`): a Cults building under construction, which the units that assist it take an
    /// Assist on (`construction::site::assists`). `None` for anything else.
    pub site: Option<ObjectKind>,
    /// (M5-N8) Its kind, if it is an object of the unit's owner's that a repair may restore (`repair::damage`:
    /// complete, below its MaxHP), which the units whose type repairs that kind take a Repair on (`repair::repairs`).
    /// `None` for anything else.
    pub repair: Option<ObjectKind>,
    /// (M6-GH1) It is an Airfield of the unit's owner's in operation (`crate::aircraft::is_airfield`), which the
    /// aircraft that rearm at one are assigned to. `false` for anything else.
    pub airfield: bool,
    /// (M6-CM) It is such a building ([`Aimed::accepts`]) that doesn't take illusions in (`mirage::refuses_illusions`:
    /// one that augments the units inside, the Altar). `false` for anything else.
    pub refuses_illusions: bool,
    /// (M6-ST1) Its hold, if it is a carrier of the unit's owner's that loads now (`crate::carry`, `carry::loads`: the
    /// Subterranean Transport above ground), which the units of a base it takes board (`carry::takes`). `None` for
    /// anything else.
    pub carrier: Option<&'static CarrierDef>,
    /// If it is a Supply Tower of the unit's owner's in operation (`supply_run::is_tower`), the chopper
    /// attached to it, `Some(None)` for none. `None` for anything else.
    pub supply_tower: Option<Option<ObjectId>>,
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
    /// (M5-S2b) Not a full building that [`Rule::Repair`] would give the unit, damaged (design change 0378). (M6-ST1)
    /// Or an own carrier that loads now and takes the unit's base in, by the same rule.
    Enter,
    /// An own site that the unit's type builds from outside: Assist.
    Assist,
    /// (M5-N8) A damaged own building or vehicle that the unit's type repairs: Repair.
    Repair,
    /// (M6-GH1) An own Airfield, for an aircraft that rearms at one: Return to Base on its own, Rebase on another.
    Rebase,
    /// A Supply Delivery Station, for a Supply Chopper: Pick Up Supplies; an own Supply Tower with no other
    /// chopper attached: Drop Off Supplies while it carries Supplies, Attach to Tower while it doesn't.
    SupplyRun,
    /// An enemy the unit can attack, its type taking Attack orders: Attack.
    Attack,
}

/// (M5-J1) The rules of the resolution on an object, in the order they are tried, as data: the first that applies
/// gives the order, and a Move to the object if none does (module docs). A new default order is a row in its place:
/// slice M5-N8's Repair, after [`Rule::Enter`] and [`Rule::Assist`] and before [`Rule::Attack`], in the design's order
/// (`factions/cults/units/recruit.md > Right-Click Resolution`, design change 0325: "Damaged own building or vehicle
/// (completed) → Repair, **unless the building can be entered** by the selection: then **Enter** wins"), which (M5-S2b)
/// a full building's Enter lets through to it ([`Rule::Enter`], design change 0378). (M6-GH1) The aircraft's Rebase,
/// on an own Airfield, which no other rule gives anything; the Supply Chopper's supply run, on a station or an
/// own Supply Tower, which no other rule gives a chopper anything.
pub const RULES: &[Rule] = &[
    Rule::Gather,
    Rule::GatherSupplies,
    Rule::DropOff,
    Rule::Enter,
    Rule::Assist,
    Rule::Repair,
    Rule::Rebase,
    Rule::SupplyRun,
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
            Rule::Enter => {
                // Design change 0378: "An Enter certain to fail is a wasted click", so a full building goes to the
                // Repair it would get. The Tunnel Network takes everyone the tier lets through.
                let repairs_instead = target.full && Rule::Repair.order(unit, target).is_some();
                // (M6-CM) Nor is an illusion's at a building that refuses illusions (`crate::mirage`).
                let refused = unit.illusion && target.refuses_illusions;
                // (M6-ST1) A carrier is boarded as a building is entered (`crate::carry`).
                let boards = target.carrier.is_some_and(|def| carry::takes(def, unit.kind));
                let pool = (target.accepts.contains(&unit.kind) || boards) && !repairs_instead && !refused;
                // Its type never going back into the Tunnel Network (`network::lets_in`), a one-way unit resolves on
                // a Tunnel as one its tier doesn't let through.
                let transit = network::passes(target.transit, unit.transit) && !network::one_way(unit.kind);
                (transit || pool).then_some(OrderKind::Enter(id))
            }
            Rule::Assist => (target.site.is_some_and(|kind| assists(unit.kind, kind))).then_some(OrderKind::Assist(id)),
            Rule::Repair => {
                (target.repair.is_some_and(|kind| repairs(unit.kind, kind))).then_some(OrderKind::Repair(id))
            }
            Rule::Rebase => match unit.aircraft {
                Some(base) if target.airfield && base == Some(id) => Some(OrderKind::ReturnToBase),
                Some(_) if target.airfield => Some(OrderKind::Rebase(id)),
                _ => None,
            },
            Rule::SupplyRun => {
                let chopper = unit.chopper?;
                if target.station {
                    return Some(OrderKind::PickUpSupplies(id));
                }
                let attached = target.supply_tower?;
                let free = attached.is_none() || chopper.attached == Some(id);
                free.then_some(match chopper.carrying {
                    true => OrderKind::DropOffSupplies(id),
                    false => OrderKind::AttachToTower(id),
                })
            }
            Rule::Attack => {
                (target.enemy && target.destructible && unit.attacks(target.domain)).then_some(OrderKind::Attack(id))
            }
        }
    }
}
