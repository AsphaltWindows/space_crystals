//! What a click on the map resolves to, and how the pointer previews it (`systems/controls/interface.md >
//! CursorTarget`, `systems/controls/interface.md > RightClickResolution`,
//! `systems/controls/interface.md > AwaitingTarget`, `systems/controls/hud.md >
//! PointerDisplayType`), as plain data. Unit tested here; the systems apply it to clicks (`systems.rs`) and to the
//! pointer ([`super::cursor`]).
//!
//! - **Right-click** (design changes 0062 and 0063): each selected object resolves it for itself, by its own
//!   template and capabilities ([`object_right_click`]), whichever group is active. A unit gets its default command
//!   on what is under the cursor ([`unit_right_click`]): the sim's one rule, `sc_sim::orders::resolution::resolve`,
//!   the rally point's too (design change 0136), on what the unit can do and what the player knows of the target
//!   (`Sim::aimed`: an object it sees, or one it remembers, design change 0139). A unit that can't carry out the
//!   command moves to the location or to the object instead, following a unit; so does a Peacekeeper (Shielded),
//!   whose type takes no Attack (design changes 0074 and 0202); a Drone gathers a patch, delivers its load to its own
//!   Egress Node,
//!   and moves to anything else, following a unit (`factions/colonists/units/drone.md > Right-Click Resolution`;
//!   an empty Drone moves to the node, design change 0140). The orders go out grouped, one per distinct order
//!   ([`group_orders`]). A producing
//!   structure sets its rally point on what was clicked ([`rally_target`], `systems/controls/interface.md >
//!   Production Building Default Right-Click`): the ground, any object, or itself, which clears it (design change
//!   0086).
//! - **What a click is on** ([`MapTarget`], `systems/controls/interface.md > CursorTarget`): an object the
//!   player may select, or a record of one it remembers, which it may target but not select (design change 0139); else
//!   the ground.
//! - **Left-click while a target is awaited** ([`target_resolution`]): the PendingCommand's order for the target
//!   ([`target_order`]); Attack takes any destructible object, own and neutral ones too (design changes 0059 and
//!   0064). It goes to every unit of the command's recipients, every selected unit for an Attack (design change
//!   0397), except that a unit that can't attack moves to the object instead of an explicit Attack on it
//!   ([`unit_target_order`], design change 0398; a Peacekeeper whose shield is on its way down takes the Attack, and
//!   attacks once the shield is down, design change 0202), and so does (M6-S6a) one whose weapon can't hit the
//!   object's domain (design change 0461), but for a unit that can't move; an Attack none of them can carry out is an
//!   invalid target ([`target_orders`]); an AttackMove goes to all of them (design change 0076),
//!   which plays out as a move for those that can't attack. Set Rally Point takes the ground or any object, as the
//!   right-click does; the Drones' Set Drop-off an own Egress Node, which becomes the AssignedDropOff of every one of
//!   them (an ability, `sc_sim::Ability::SetDropOff`; slice M3-J2); the Agents' Drop Off an own Tunnel in operation, a
//!   Drop Off order (slice M4-J1); the Recruits' Assist Construction an own site built from outside, an Assist order
//!   (slice M5-J1); the Recruits' and the Agents' Repair a damaged own object that one of them repairs, a Repair order
//!   (slice M5-N8); (M5-N5) a Shrine's cast a point the player sees now, the ground under the cursor, a cast there;
//!   (M5-K) a Rocketman's Target Lock an enemy unit or structure the player sees now, a Target Lock order.
//! - **The pointer** ([`pointer_display`]): in DefaultState the one thing the selected objects would all do on a
//!   right-click, or Mixed when they would do different things (a rally point shows Move:
//!   `systems/controls/hud.md > Resolution Rules > DefaultState`, "Selection is a production building:
//!   Move (rally point)"; a Gather GatherResources, a Drop Off ReturnResources, an Enter Enter); while a target is
//!   awaited, the pending command's look over a valid target, else Inactive (`resolution/pointer.rs`).

use bevy::math::Vec2;
use sc_sim::defs::{Domain, Transit};
use sc_sim::orders::resolution::{Abilities, Aim, Aimed, resolve};
use sc_sim::production::takes_rally;
use sc_sim::repair::repairs;
use sc_sim::{
    Ability, AbilityArea, AreaAbility, ObjectId, ObjectKind, OrderKind, PlayerId, Sim, Target, TargetView,
    vehicle_deploy,
};

use super::interface::{InterfaceTemplate, Modes, PanelCommand};
use crate::selection::{Relation, Selectables, Selection};

mod pointer;

pub use pointer::{PointerDisplay, pointer_display, right_click_display};

/// `systems/controls/interface.md > CursorTarget`: what is under the cursor when it clicks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CursorTarget {
    /// The ground, at this point.
    Ground(Vec2),
    EnemyObject(ObjectId),
    /// The player's own objects and their allies'.
    FriendlyObject(ObjectId),
    /// Unowned objects: the map resources and neutral units.
    NeutralObject(ObjectId),
}

impl CursorTarget {
    /// What is under the view point `view`, whose ground point is `ground`: the object a click there targets
    /// (`Selectables::target_at`: one the player may select, or a record it may only target), else the ground.
    pub fn at(objects: &Selectables, view: Vec2, ground: Vec2) -> Self {
        match objects.target_at(view) {
            Some(object) => match object.relation {
                Relation::Enemy => CursorTarget::EnemyObject(object.id),
                Relation::Own | Relation::Ally => CursorTarget::FriendlyObject(object.id),
                Relation::Neutral => CursorTarget::NeutralObject(object.id),
            },
            None => CursorTarget::Ground(ground),
        }
    }

    /// The object under the cursor; none over the ground.
    pub fn object(self) -> Option<ObjectId> {
        match self {
            CursorTarget::Ground(_) => None,
            CursorTarget::EnemyObject(id) | CursorTarget::FriendlyObject(id) | CursorTarget::NeutralObject(id) => {
                Some(id)
            }
        }
    }
}

/// What a click, or the pointer, is on, as the resolution needs it: the [`CursorTarget`], the ground point under
/// the cursor, the object there as the player knows it, and whether the player sees the point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MapTarget {
    pub target: CursorTarget,
    /// The ground point under the cursor: where a Move to the location goes.
    pub ground: Vec2,
    /// The object under the cursor as the player knows it (`Sim::aimed`): one it sees, as it is, or one it remembers,
    /// by its record (design change 0139). `None` over the ground, and over an object the player knows nothing of,
    /// which only the fog's debug toggle shows.
    pub aimed: Option<Aimed>,
    /// (M5-N5) Whether the player sees the ground point now, as the gate judges a cast's point (`Sim::sees_point`: on
    /// the map, on a tile Visible to them as the last tick left their vision; `factions/cults/structures/shrine.md >
    /// Abilities`: "cast on an area anywhere the player currently has vision").
    pub seen: bool,
    /// (M6-GR) Whether the ground point is on the map, seen or not, as the gate judges a Scan's point
    /// (`sc_sim::AbilityArea::AnyPoint`; `factions/gdo/structures/radar_station.md > Placeholder stats`: "left-click
    /// any point on the map").
    pub on_map: bool,
    /// (M5-K) Whether the player sees the object under the cursor now (`Sim::target_view`: Visible), not only
    /// remembers it: a Target Lock's target must be seen (`factions/gdo/units/rocketman.md > Target Lock`: "Locking
    /// succeeds only if the target stays within LockRange and visible for the whole duration"). `false` over the
    /// ground.
    pub visible: bool,
}

impl MapTarget {
    /// What is under the view point `view`, whose ground point is `ground`: the object a click there targets among
    /// `objects` ([`CursorTarget::at`]), as `player` knows it in `sim`, else the ground.
    pub fn at(sim: &Sim, player: PlayerId, objects: &Selectables, view: Vec2, ground: Vec2) -> Self {
        Self::of(sim, player, CursorTarget::at(objects, view, ground), ground)
    }

    /// (M5-MM) The ground at `ground` alone, never an object there: what a click on the minimap is on
    /// (`systems/controls/hud.md > Minimap > Using it`, the designer's placeholder: "the minimap targets only ground,
    /// never an object"), as `player` sees it in `sim`.
    pub fn on_ground(sim: &Sim, player: PlayerId, ground: Vec2) -> Self {
        Self::of(sim, player, CursorTarget::Ground(ground), ground)
    }

    fn of(sim: &Sim, player: PlayerId, target: CursorTarget, ground: Vec2) -> Self {
        Self {
            target,
            ground,
            aimed: target.object().and_then(|id| sim.aimed(player, id)),
            seen: sim.sees_point(player, ground),
            on_map: sim.map().contains_point(ground),
            visible: (target.object()).is_some_and(|id| matches!(sim.target_view(player, id), TargetView::Visible)),
        }
    }

    /// The ground at `point`, which the player sees.
    pub fn ground(point: Vec2) -> Self {
        Self {
            target: CursorTarget::Ground(point),
            ground: point,
            aimed: None,
            seen: true,
            on_map: true,
            visible: false,
        }
    }

    /// The object can be attacked: its type is destructible (`systems/controls/commands.md > Attack`: "a specific
    /// destructible object instance"). False over the ground.
    pub fn destructible(&self) -> bool {
        self.aimed.is_some_and(|aimed| aimed.destructible)
    }

    /// The domain the object is targeted in (`sc_sim::combat::object_domain`): Ground over the ground, and for a
    /// structure the player remembers.
    pub fn domain(&self) -> Domain {
        self.aimed.map_or(Domain::Ground, |aimed| aimed.domain)
    }
}

/// A selected object as a right-click resolves it for itself (`systems/controls/interface.md >
/// RightClickResolution`: "Each selected unit resolves the right-click for itself, against its own capabilities, not
/// the ActiveGroup's"): a unit, or a producing structure, whose right-click sets its rally point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnitCapabilities {
    /// Its type's ObjectInterfaceState template; none for a type without commands and for an object that transforms
    /// (slice M3-J1). A producing structure's right-click sets its rally point
    /// (`systems/controls/interface.md > Production Building Default Right-Click`): **placeholder reading**,
    /// design change 0062 ("Buildings in the selection aren't affected") is read as the buildings without a right-click
    /// of their own, so a Barracks selected with units sets its rally point as the units take their orders. A
    /// constructor's does nothing.
    pub template: Option<InterfaceTemplate>,
    /// What a unit can do, as its default command reads it (`sc_sim::orders::resolution::Abilities`); none for an
    /// object that isn't a unit.
    pub abilities: Option<Abilities>,
}

impl UnitCapabilities {
    /// Those of the selected object `id`, of type `kind`, whose mode and load are in `modes`. An object that isn't in
    /// operation yet resolves nothing (`factions/colonists/economy.md > Beacon Deployment > While transforming: only
    /// Cancel`, design change 0116; slice M3-J1), but (M5-S6) a building that will produce or release units, whose
    /// right-click sets its rally point "as on the finished building" (design change 0396,
    /// `sc_sim::production::takes_rally`). (M6-GA) A unit that deploys resolves as the type it becomes
    /// ([`Modes::becomes`]): it holds the order, the first thing it tries once its deploy is done
    /// (`factions/gdo/units/artillery_rig.md > Modes`), so that the mobile Rig, deploying, holds an Attack on the enemy
    /// right-clicked.
    pub fn of(id: ObjectId, kind: ObjectKind, modes: &Modes) -> Self {
        let kind = modes.becomes(id, kind);
        let transforming = modes.transforming(id);
        Self {
            template: InterfaceTemplate::of(kind).filter(|_| !transforming || takes_rally(kind)),
            abilities: (kind.def().unit.as_ref())
                .filter(|_| !transforming)
                .map(|unit| Abilities {
                    kind,
                    attack: unit.attack.map(|attack| attack.target_domain),
                    may_attack: modes.can_attack(id),
                    worker: unit.worker.is_some(),
                    carrying: modes.carries(id),
                    gathers_supplies: unit.worker.is_some_and(|worker| worker.supplies.is_some()),
                    transit: Transit::of(unit.base),
                    aircraft: modes.aircraft(id),
                    illusion: modes.illusion(id),
                }),
        }
    }

    /// Its type takes an Attack: not a Peacekeeper (Shielded) ([`Modes::can_attack`], design change 0202), nor (M5-N4a)
    /// a unit without a weapon. One that doesn't moves instead of an Attack ([`unit_target_order`]).
    pub fn may_attack(&self) -> bool {
        self.abilities.is_none_or(|abilities| abilities.may_attack)
    }

    /// (M6-S6a) It takes an Attack on a target in `domain` ([`MapTarget::domain`]), and its weapon can hit that domain:
    /// the sim's check, the gate's (`sc_sim::orders::resolution::Abilities::attacks`, `sc_sim::combat::can_hit`; design
    /// change 0461). One that can't moves to the target of an Attack instead ([`unit_target_order`]).
    pub fn attacks(&self, domain: Domain) -> bool {
        self.abilities.is_none_or(|abilities| abilities.attacks(domain))
    }
}

/// The selected objects with their capabilities, in id order.
pub fn capabilities(selection: &Selection, modes: &Modes) -> Vec<(ObjectId, UnitCapabilities)> {
    selection
        .members()
        .map(|(id, kind)| (id, UnitCapabilities::of(id, kind, modes)))
        .collect()
}

/// What a click gives one selected object: a unit's order, a producer's rally point, or a unit's ability on the
/// target.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Resolved {
    /// A unit's order (`Command::Order`).
    Order(OrderKind),
    /// A producer's rally point (`ProductionCommand::SetRally`): the ground or an object, the producer itself to
    /// clear it (design change 0086).
    Rally(Target),
    /// A unit's ability on the target (`Command::Ability`): the Drones' Set Drop-off, on an own Egress Node (slice
    /// M3-J2). No right-click gives one.
    Ability(Ability),
    /// (M5-N5) A Shrine's cast of the ability at the point (`Command::Cast`). No right-click gives one.
    Cast(AreaAbility, Vec2),
}

/// `systems/controls/interface.md > RightClickResolution` for one selected object: a unit's order
/// ([`unit_right_click`]), or, for a producing structure, its rally point on what is under the cursor
/// (`systems/controls/interface.md > Production Building Default Right-Click`: "Right-click Ground: sets rally
/// point to that location; Right-click Object: sets rally point to that object; Right-click the building itself: clears
/// the rally point"). Nothing for a constructor (it produces no units), nor for any other object that isn't a unit.
pub fn object_right_click(object: UnitCapabilities, target: &MapTarget) -> Option<Resolved> {
    match object.template {
        // A pool building's too (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory] > Right-click
        // resolution`: "Right-click Ground: issues SetRallyPoint command to that location", slice M5-F2), and the
        // Altar's (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`, slice M5-N1b).
        Some(InterfaceTemplate::Production | InterfaceTemplate::PoolProducer | InterfaceTemplate::Altar) => {
            Some(Resolved::Rally(rally_target(target)))
        }
        // A constructor produces no units, and the Beacon, the Egress Node and the Tunnel have no right-click, nor has
        // the Garrison (`factions/cults/structures/garrison.md > Placeholder stats > Panel`: "Right-click resolution:
        // none (it has no rally point)", slice M5-N2), nor the Temple, which researches and has no rally point either
        // (slice M5-N3; its panel has no C), nor (M5-N5) the Shrine, which casts and has none either, nor (M6-GR)
        // the Radar Station, which scans, nor the Defense System, a constructor.
        Some(
            InterfaceTemplate::DeploymentCenter
            | InterfaceTemplate::ExtractionFacility
            | InterfaceTemplate::RadarStation
            | InterfaceTemplate::DefenseSystem
            | InterfaceTemplate::Beacon
            | InterfaceTemplate::EgressNode
            | InterfaceTemplate::Tunnel
            | InterfaceTemplate::Garrison
            | InterfaceTemplate::Temple
            | InterfaceTemplate::Shrine,
        ) => None,
        // A unit's default command, by the sim's one rule: the BasicCombatUnit's table, the Drone's own
        // (`factions/colonists/units/drone.md > Right-Click Resolution`; slices M3-R and M3-J2) and the Agent's
        // (`factions/syndicate/units/agent.md > Right-Click Resolution`, slice M4-J1) and the Recruit's
        // (`factions/cults/units/recruit.md > Right-Click Resolution`, slice M5-J1), and the Field Engineer's
        // (`factions/gdo/units/field_engineer.md > Placeholder stats`: "Right-click a damaged own building or vehicle:
        // Repair", slice M5-N9), and (M6-GD) the operator's drone's, and (M6-ST1) the Subterranean Transport's. Without
        // a template, an object that transforms, which resolves nothing (`unit_right_click`: it has no abilities then).
        Some(
            InterfaceTemplate::BasicCombatUnit
            | InterfaceTemplate::Drone
            | InterfaceTemplate::Agent
            | InterfaceTemplate::Recruit
            | InterfaceTemplate::FieldEngineer
            | InterfaceTemplate::OperatorDrone
            | InterfaceTemplate::Transport,
        )
        | None => unit_right_click(object, target).map(Resolved::Order),
    }
}

/// The rally point a click on `target` sets (`systems/controls/interface.md > Production Building Default
/// Right-Click`, `systems/controls/interface.md > Standard Slot Assignments > C`): the ground point, or the
/// object, whoever owns it. Which objects the player may rally on, the sim decides (`Sim::validate`): one it sees, the
/// producer itself (which clears the rally point, design change 0086), and one it remembers, on which the rally point
/// stays while it is out of sight (design change 0139).
pub fn rally_target(target: &MapTarget) -> Target {
    match target.target.object() {
        Some(id) => Target::Object(id),
        None => Target::Point(target.ground),
    }
}

/// `systems/controls/interface.md > RightClickResolution` for one selected unit (design changes 0062, 0063 and
/// 0136): the order `unit` gets for a right-click on `target`, its default command there: the sim's rule
/// (`sc_sim::orders::resolution::resolve`, which a rally point's new unit goes by too), on what the unit can do and the
/// target as the player knows it ([`MapTarget::aimed`]). A unit that can't carry out the command it would get moves to
/// the target instead, a Peacekeeper (Shielded) too, its shield up or on its way up (design change 0074: it follows an
/// enemy unit, and moves to an enemy structure, so that in a mixed group the shield wall stays with the units that
/// attack; an explicit Attack moves it to the target too, [`unit_target_order`], design change 0397, where 0075 left it
/// out). A Peacekeeper whose shield is on its way down attacks (design change 0202: "a right-click on an enemy is
/// accepted and held"). Over an object the player knows nothing of, a Move to it ([`move_to`]), which the sim refuses.
/// Nothing for an object that isn't a unit.
pub fn unit_right_click(unit: UnitCapabilities, target: &MapTarget) -> Option<OrderKind> {
    let abilities = unit.abilities?;
    let aim = match (target.target, target.aimed) {
        (CursorTarget::Ground(point), _) => Aim::Ground(point),
        (_, Some(aimed)) => Aim::Object(aimed),
        (_, None) => return Some(move_to(target)),
    };
    Some(resolve(abilities, aim))
}

/// Design change 0062: what a unit does on a target it can't act on: it moves to the location, or to the object,
/// which follows a unit (`systems/controls/command_behaviors.md > MovingToObject`), so that the group stays together.
pub fn move_to(target: &MapTarget) -> OrderKind {
    OrderKind::Move(match target.target.object() {
        Some(id) => Target::Object(id),
        None => Target::Point(target.ground),
    })
}

/// The order one of a targeting click's recipients takes, when the click gives `kind` ([`target_order`]):
/// `systems/controls/interface.md > CommandPanel` (design changes 0397 and 0398): "an Attack (A) is issued to **every
/// selected unit**, even when some can't attack: Units that can attack carry it out: Attack on a target, or Attack
/// Move on ground. Units that can't attack [...] treat an Attack or an Attack Move as a **Move** (user, 2026-11-27): on
/// ground they move to the spot (with the group, as for Attack Move); on a target they move to it (as a Move on that
/// object)." A unit whose type takes no Attack (a unit without a weapon, as the gate judges it,
/// `sc_sim::combat::takes_attack_orders`; a Peacekeeper (Shielded), design change 0202) gets a Move to the object of an
/// explicit Attack, which follows a unit (`systems/controls/command_behaviors.md > MovingToObject`); the units that
/// take the Attack carry it out, a Peacekeeper whose shield is on its way down once it is down. Every other order goes
/// to every recipient as it is, an AttackMove too (design change 0076, `systems/controls/commands.md > AttackMove >
/// Units that can't attack`: it plays out as a move for a unit that can't attack), so that the group's units go to the
/// spot as one group. **Reading**: the Peacekeeper (Shielded) moves to the target too, though
/// `factions/gdo/units/peacekeeper.md > Riot Shield` still leaves it out of an explicit Attack (design change 0075),
/// "follow[ing] the CommandPanel rule that a command goes only to objects that can carry it out", the rule design
/// change 0397 replaced. Unlike the right-click ([`unit_right_click`]), on purpose. (M6-S6a) A unit whose weapon can't
/// hit the target, its domain `domain` ([`MapTarget::domain`]), "is treated like a unit that can't attack: it **moves
/// to the target (follows it) with the group**" (design change 0461, answering Q0186: "e.g. an anti-air Twister on a
/// ground unit or structure, Peacekeepers on a Sparrow"; [`UnitCapabilities::attacks`]).
pub fn unit_target_order(unit: UnitCapabilities, kind: OrderKind, domain: Domain) -> OrderKind {
    match kind {
        OrderKind::Attack(id) if !unit.attacks(domain) => OrderKind::Move(Target::Object(id)),
        _ => kind,
    }
}

/// (M6-S6a) The orders a targeting click of `kind` gives `units`, its command's recipients with their types, in
/// `modes` (theirs, `Modes::of`), the clicked target in `domain` ([`MapTarget::domain`]): each its order as
/// [`unit_target_order`] has it, by its capabilities ([`UnitCapabilities::of`]), but none to a unit that can't move
/// (`sc_sim::vehicle_deploy::immobile`: a deployed Artillery Rig, or one deploying into one, judged as the type it
/// becomes as its capabilities are), which follows nothing: the gate would refuse it the Move
/// (`sc_sim::Reject::CannotMove`). `None` for an Attack none of them can carry out on its target: the click is an
/// invalid target (design change 0461: "If **no** selected unit can attack the target, the click is an **invalid
/// target** [...]: ignored, with feedback, and the Attack still awaits a target", as the gate refuses it,
/// `sc_sim::Reject::CannotHit`).
pub fn target_orders(
    units: impl IntoIterator<Item = (ObjectId, ObjectKind)>,
    modes: &Modes,
    kind: OrderKind,
    domain: Domain,
) -> Option<Vec<(ObjectId, OrderKind)>> {
    let units: Vec<(ObjectId, UnitCapabilities, bool)> = (units.into_iter())
        .map(|(id, of)| {
            let mobile = !vehicle_deploy::immobile(modes.becomes(id, of));
            (id, UnitCapabilities::of(id, of, modes), mobile)
        })
        .collect();
    if matches!(kind, OrderKind::Attack(_)) && !units.iter().any(|(_, unit, _)| unit.attacks(domain)) {
        return None;
    }
    let orders = (units.into_iter())
        .map(|(id, unit, mobile)| (id, unit_target_order(unit, kind, domain), mobile))
        .filter(|&(_, order, mobile)| order == kind || mobile)
        .map(|(id, order, _)| (id, order))
        .collect();
    Some(orders)
}

/// A click's orders, one per unit in the units' order, grouped into one order per distinct order, in the order each
/// first comes: as few `Command::Order`s as possible (the brief of design change 0062). Units sent to one location
/// go as one group, which the magic box keeps together (`systems/controls/commands.md > Group Destinations`).
pub fn group_orders(orders: impl IntoIterator<Item = (ObjectId, OrderKind)>) -> Vec<(OrderKind, Vec<ObjectId>)> {
    let mut groups: Vec<(OrderKind, Vec<ObjectId>)> = Vec::new();
    for (id, kind) in orders {
        match groups.iter_mut().find(|(group, _)| *group == kind) {
            Some((_, units)) => units.push(id),
            None => groups.push((kind, vec![id])),
        }
    }
    groups
}

/// `systems/controls/interface.md > BasicCombatUnitInterfaceState > AwaitingTarget[..] resolution`: the order a
/// left-click on `target` gives while `pending` awaits it, if the target is valid. Attack takes any destructible
/// object, an own or a neutral one too (force-attack, `systems/controls/commands.md > Attack`, design changes 0059 and
/// 0064), one the player remembers too (design change 0139), and the ground, for an AttackMove. AttackGround takes the
/// ground only (`systems/controls/interface.md > AwaitingTarget[AttackGround] resolution`: "Left-click Ground:
/// issues AttackGround command to that location"; slice M4-O), wherever the click falls on it, over an object the
/// player doesn't see too. The Agent's Drop Off takes an own drop-off in operation ([`drop_off_target`], slice M4-J1;
/// `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`: "Requires clicking an own Tunnel"). The Recruits'
/// Assist Construction takes an own site built from outside ([`assist_target`], slice M5-J1), and the Repair a damaged
/// own object ([`repair_target`], slice M5-N8). (M5-V2) Reverse takes the ground only
/// (`systems/controls/interface.md > AwaitingTarget[Reverse] resolution`: "Left-click Ground: issues Reverse command to
/// that location"). (M5-K) A Rocketman's Target Lock takes an enemy unit or structure
/// (`factions/gdo/units/rocketman.md > Placeholder stats`: "targeted: left-click an enemy unit or structure") that the
/// player sees now ([`MapTarget::visible`]): **reading**, a lock "succeeds only if the target stays within LockRange
/// and visible for the whole duration", and one on a structure the player only remembers would break at once
/// (`sc_sim::combat::target_lock`), so the click on it is an invalid target.
pub fn target_order(pending: PanelCommand, target: &MapTarget) -> Option<OrderKind> {
    use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
    match (pending, target.target) {
        (PanelCommand::Attack, Ground(point)) => Some(OrderKind::AttackMove(point)),
        (PanelCommand::Attack, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            target.destructible().then_some(OrderKind::Attack(id))
        }
        (PanelCommand::Move, Ground(point)) => Some(OrderKind::Move(Target::Point(point))),
        (PanelCommand::Reverse, Ground(point)) => Some(OrderKind::Reverse(point)),
        (PanelCommand::Move, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            Some(OrderKind::Move(Target::Object(id)))
        }
        (PanelCommand::Patrol, Ground(point)) => Some(OrderKind::Patrol(point)),
        (PanelCommand::AttackGround, Ground(point)) => Some(OrderKind::AttackGround(point)),
        (PanelCommand::DropOff, _) => drop_off_target(target).map(OrderKind::DropOff),
        (PanelCommand::Assist, _) => assist_target(target).map(OrderKind::Assist),
        (PanelCommand::Repair, _) => repair_target(target).map(OrderKind::Repair),
        (PanelCommand::TargetLock, EnemyObject(id)) => {
            (target.destructible() && target.visible).then_some(OrderKind::TargetLock(id))
        }
        _ => None,
    }
}

/// `systems/controls/interface.md > AwaitingTarget[..] resolution` for any PendingCommand: what a left-click on
/// `target` gives, if the target is valid. A unit command's order ([`target_order`]); the rally point of Set Rally
/// Point, which takes the ground and any object (`factions/gdo/structures/barracks.md`: "Left-click ground or object
/// sets the rally point"; `systems/controls/interface.md > Standard Slot Assignments > C`: the building itself
/// clears it); the Drones' new drop-off for Set Drop-off ([`drop_off_target`]). (M5-N5) A Shrine's cast: the ground
/// point under the cursor, over an object too, while the player sees it (`factions/cults/structures/shrine.md >
/// Placeholder stats`: "Targeting: AwaitingTarget, left-click a point in vision"; [`MapTarget::seen`]); a point out of
/// vision is an invalid target (plan §N5: "a click outside vision gives "invalid target" feedback"). A click over a
/// unit casts on the ground under it: **Placeholder (not in the design: "a point" read as the ground under the
/// cursor, whatever stands there)**. (M6-GR) A Radar Station's Scan takes any point on the map, seen or not
/// (`factions/gdo/structures/radar_station.md > Placeholder stats`: "left-click any point on the map"; the gate's
/// `sc_sim::AbilityArea::AnyPoint`): a point off it is an invalid target.
pub fn target_resolution(pending: PanelCommand, target: &MapTarget) -> Option<Resolved> {
    match pending {
        PanelCommand::SetRallyPoint => Some(Resolved::Rally(rally_target(target))),
        PanelCommand::SetDropOff => drop_off_target(target).map(|node| Resolved::Ability(Ability::SetDropOff(node))),
        PanelCommand::Cast(ability) => {
            let valid = match ability.def().area {
                AbilityArea::AnyPoint => target.on_map,
                _ => target.seen,
            };
            valid.then_some(Resolved::Cast(ability, target.ground))
        }
        _ => target_order(pending, target).map(Resolved::Order),
    }
}

/// `factions/colonists/units/drone.md > Drone ObjectInterfaceState > S: Set Drop-off Location`: "Left-click an Egress
/// Node or own refinery" (no refinery is in the game yet): the drop-off under a click while Set Drop-off awaits its
/// target, if it is one of the player's own that takes loads now (`sc_sim::orders::resolution::Aimed::drop_off`, the
/// gate's own check: not while it is warped in, nor while it is a site). The same for the Agent's Drop Off, whose
/// drop-off is an own Tunnel (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`: "Requires clicking an
/// own Tunnel"). Anything else is an invalid target (design change 0065).
pub fn drop_off_target(target: &MapTarget) -> Option<ObjectId> {
    target.aimed.filter(|aimed| aimed.drop_off).map(|aimed| aimed.id)
}

/// `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`: "**S: Assist
/// Construction** — [...] Left-click an in-progress own building to join construction" (slice M5-J1): the site under a
/// click while Assist Construction awaits its target, if it is one of the player's own that its builders build from
/// outside (`sc_sim::orders::resolution::Aimed::site`, the right-click's Assist: a Cults building under construction).
/// Which recipients build its kind, the gate decides (`Reject::NoWorkers`). Anything else is an invalid target.
pub fn assist_target(target: &MapTarget) -> Option<ObjectId> {
    target.aimed.filter(|aimed| aimed.site.is_some()).map(|aimed| aimed.id)
}

/// (M5-N8) `factions/cults/units/recruit.md > Recruit ObjectInterfaceState > DefaultState commands`: "**D: Repair** —
/// [...] Left-click a damaged own building or vehicle to repair it" (the Agent's: "a damaged own Tunnel or Laser
/// Turret"): the object under a click while Repair awaits its target, if it is one of the player's own that a repair
/// may restore (`sc_sim::orders::resolution::Aimed::repair`, the right-click's Repair: complete and below its MaxHP).
/// Which recipients repair its kind, the gate decides (`Reject::CantRepair`), and the pointer previews
/// ([`repaired_by`]). Anything else is an invalid target.
pub fn repair_target(target: &MapTarget) -> Option<ObjectId> {
    target
        .aimed
        .filter(|aimed| aimed.repair.is_some())
        .map(|aimed| aimed.id)
}

/// (M5-N8) Whether one of `units` repairs the object under `target` (`sc_sim::repair::repairs`, by its kind): a
/// damaged own object, complete ([`repair_target`]), of a kind one of their types repairs, which a Repair's click on it
/// would give that unit.
pub fn repaired_by(units: &[UnitCapabilities], target: &MapTarget) -> bool {
    let Some(kind) = target.aimed.and_then(|aimed| aimed.repair) else {
        return false;
    };
    (units.iter()).any(|unit| unit.abilities.is_some_and(|abilities| repairs(abilities.kind, kind)))
}

#[cfg(test)]
mod tests;
