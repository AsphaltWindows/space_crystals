//! What a click on the map resolves to, and how the pointer previews it (`systems/controls/selection_and_hud.md >
//! CursorTarget`, `systems/controls/selection_and_hud.md > RightClickResolution`,
//! `systems/controls/selection_and_hud.md > AwaitingTarget`, `systems/controls/selection_and_hud.md >
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
//!   structure sets its rally point on what was clicked ([`rally_target`], `systems/controls/selection_and_hud.md >
//!   Production Building Default Right-Click`): the ground, any object, or itself, which clears it (design change
//!   0086).
//! - **What a click is on** ([`MapTarget`], `systems/controls/selection_and_hud.md > CursorTarget`): an object the
//!   player may select, or a record of one it remembers, which it may target but not select (design change 0139); else
//!   the ground.
//! - **Left-click while a target is awaited** ([`target_resolution`]): the PendingCommand's order for the target
//!   ([`target_order`]); Attack takes any destructible object, own and neutral ones too (design changes 0059 and
//!   0064). It goes to every unit of the command's recipients, except that an explicit Attack leaves out the units
//!   whose type takes none, the Peacekeepers (Shielded) ([`unit_target_order`], design changes 0075 and 0202: a
//!   Peacekeeper whose shield is on its way down takes it, and attacks once the shield is down); an AttackMove goes to
//!   all of them (design change 0076). Set Rally Point takes the ground or any object, as the right-click does; the
//!   Drones' Set Drop-off an own Egress Node, which becomes the AssignedDropOff of every one of them (an ability,
//!   `sc_sim::Ability::SetDropOff`; slice M3-J2); the Agents' Drop Off an own Tunnel in operation, a Drop Off order
//!   (slice M4-J1).
//! - **The pointer** ([`pointer_display`]): in DefaultState the one thing the selected objects would all do on a
//!   right-click, or Mixed when they would do different things (a rally point shows Move:
//!   `systems/controls/selection_and_hud.md > Resolution Rules > DefaultState`, "Selection is a production building:
//!   Move (rally point)"; a Gather GatherResources, a Drop Off ReturnResources, an Enter Enter); while a target is
//!   awaited, the pending command's look over a valid target, else Inactive.

use std::collections::BTreeSet;

use bevy::math::Vec2;
use sc_sim::defs::{Domain, Transit};
use sc_sim::orders::resolution::{Abilities, Aim, Aimed, resolve};
use sc_sim::{Ability, ObjectId, ObjectKind, OrderKind, PlayerId, Sim, Target};

use super::interface::{InterfaceState, InterfaceTemplate, Modes, PanelCommand};
use crate::selection::{Relation, Selectables, Selection};

/// `systems/controls/selection_and_hud.md > CursorTarget`: what is under the cursor when it clicks.
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
/// the cursor, and the object there as the player knows it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MapTarget {
    pub target: CursorTarget,
    /// The ground point under the cursor: where a Move to the location goes.
    pub ground: Vec2,
    /// The object under the cursor as the player knows it (`Sim::aimed`): one it sees, as it is, or one it remembers,
    /// by its record (design change 0139). `None` over the ground, and over an object the player knows nothing of,
    /// which only the fog's debug toggle shows.
    pub aimed: Option<Aimed>,
}

impl MapTarget {
    /// What is under the view point `view`, whose ground point is `ground`: the object a click there targets among
    /// `objects` ([`CursorTarget::at`]), as `player` knows it in `sim`, else the ground.
    pub fn at(sim: &Sim, player: PlayerId, objects: &Selectables, view: Vec2, ground: Vec2) -> Self {
        let target = CursorTarget::at(objects, view, ground);
        Self {
            target,
            ground,
            aimed: target.object().and_then(|id| sim.aimed(player, id)),
        }
    }

    /// The ground at `point`.
    pub fn ground(point: Vec2) -> Self {
        Self {
            target: CursorTarget::Ground(point),
            ground: point,
            aimed: None,
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

/// A selected object as a right-click resolves it for itself (`systems/controls/selection_and_hud.md >
/// RightClickResolution`: "Each selected unit resolves the right-click for itself, against its own capabilities, not
/// the ActiveGroup's"): a unit, or a producing structure, whose right-click sets its rally point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnitCapabilities {
    /// Its type's ObjectInterfaceState template; none for a type without commands and for an object that transforms
    /// (slice M3-J1). A producing structure's right-click sets its rally point
    /// (`systems/controls/selection_and_hud.md > Production Building Default Right-Click`): **placeholder reading**,
    /// design change 0062 ("Buildings in the selection aren't affected") is read as the buildings without a right-click
    /// of their own, so a Barracks selected with units sets its rally point as the units take their orders. A
    /// constructor's does nothing.
    pub template: Option<InterfaceTemplate>,
    /// What a unit can do, as its default command reads it (`sc_sim::orders::resolution::Abilities`); none for an
    /// object that isn't a unit.
    pub abilities: Option<Abilities>,
}

impl UnitCapabilities {
    /// Those of the selected object `id`, of type `kind`, whose mode and load are in `modes`. An object that
    /// transforms resolves nothing (`factions/colonists/economy.md > Beacon Deployment > While transforming: only
    /// Cancel`, design change 0116: "no rally point"): a portal under Warp In sets no rally point (slice M3-J1).
    pub fn of(id: ObjectId, kind: ObjectKind, modes: &Modes) -> Self {
        let transforming = modes.transforming(id);
        Self {
            template: InterfaceTemplate::of(kind).filter(|_| !transforming),
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
                }),
        }
    }

    /// Its type takes an Attack: not a Peacekeeper (Shielded) ([`Modes::can_attack`], design change 0202).
    pub fn may_attack(&self) -> bool {
        self.abilities.is_none_or(|abilities| abilities.may_attack)
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
}

/// `systems/controls/selection_and_hud.md > RightClickResolution` for one selected object: a unit's order
/// ([`unit_right_click`]), or, for a producing structure, its rally point on what is under the cursor
/// (`systems/controls/selection_and_hud.md > Production Building Default Right-Click`: "Right-click Ground: sets rally
/// point to that location; Right-click Object: sets rally point to that object; Right-click the building itself: clears
/// the rally point"). Nothing for a constructor (it produces no units), nor for any other object that isn't a unit.
pub fn object_right_click(object: UnitCapabilities, target: &MapTarget) -> Option<Resolved> {
    match object.template {
        Some(InterfaceTemplate::Production) => Some(Resolved::Rally(rally_target(target))),
        // A constructor produces no units, and the Beacon, the Egress Node and the Tunnel have no right-click.
        Some(
            InterfaceTemplate::DeploymentCenter
            | InterfaceTemplate::ExtractionFacility
            | InterfaceTemplate::Beacon
            | InterfaceTemplate::EgressNode
            | InterfaceTemplate::Tunnel,
        ) => None,
        // A unit's default command, by the sim's one rule: the BasicCombatUnit's table, the Drone's own
        // (`factions/colonists/units/drone.md > Right-Click Resolution`; slices M3-R and M3-J2) and the Agent's
        // (`factions/syndicate/units/agent.md > Right-Click Resolution`, slice M4-J1). Without a template, an object
        // that transforms, which resolves nothing (`unit_right_click`: it has no abilities then).
        Some(InterfaceTemplate::BasicCombatUnit | InterfaceTemplate::Drone | InterfaceTemplate::Agent) | None => {
            unit_right_click(object, target).map(Resolved::Order)
        }
    }
}

/// The rally point a click on `target` sets (`systems/controls/selection_and_hud.md > Production Building Default
/// Right-Click`, `systems/controls/selection_and_hud.md > Standard Slot Assignments > C`): the ground point, or the
/// object, whoever owns it. Which objects the player may rally on, the sim decides (`Sim::validate`): one it sees, the
/// producer itself (which clears the rally point, design change 0086), and one it remembers, on which the rally point
/// stays while it is out of sight (design change 0139).
pub fn rally_target(target: &MapTarget) -> Target {
    match target.target.object() {
        Some(id) => Target::Object(id),
        None => Target::Point(target.ground),
    }
}

/// `systems/controls/selection_and_hud.md > RightClickResolution` for one selected unit (design changes 0062, 0063 and
/// 0136): the order `unit` gets for a right-click on `target`, its default command there: the sim's rule
/// (`sc_sim::orders::resolution::resolve`, which a rally point's new unit goes by too), on what the unit can do and the
/// target as the player knows it ([`MapTarget::aimed`]). A unit that can't carry out the command it would get moves to
/// the target instead, a Peacekeeper (Shielded) too, its shield up or on its way up (design change 0074: it follows an
/// enemy unit, and moves to an enemy structure, so that in a mixed group the shield wall stays with the units that
/// attack; an explicit Attack leaves it out instead, [`unit_target_order`], design change 0075). A Peacekeeper whose
/// shield is on its way down attacks (design change 0202: "a right-click on an enemy is accepted and held"). Over an
/// object the player knows nothing of, a Move to it
/// ([`move_to`]), which the sim refuses. Nothing for an object that isn't a unit.
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
/// which follows a unit (`systems/controls/behaviors.md > MovingToObject`), so that the group stays together.
pub fn move_to(target: &MapTarget) -> OrderKind {
    OrderKind::Move(match target.target.object() {
        Some(id) => Target::Object(id),
        None => Target::Point(target.ground),
    })
}

/// The order one of a targeting click's recipients takes, when the click gives `kind` ([`target_order`]): design
/// change 0075 (`factions/gdo/units/peacekeeper.md > Riot Shield > Explicit Attack`): a unit whose type takes no
/// Attack (a Peacekeeper (Shielded), design change 0202) is left out of an explicit Attack, on any object, and keeps
/// its orders; the units that take it carry it out, a Peacekeeper whose shield is on its way down once it is down.
/// Every other order goes to every recipient, an AttackMove too
/// (design change 0076, `systems/controls/commands.md > AttackMove > Units that can't attack`: it plays out as a
/// move for a unit that can't attack). Unlike the right-click ([`unit_right_click`]), on purpose.
pub fn unit_target_order(unit: UnitCapabilities, kind: OrderKind) -> Option<OrderKind> {
    match kind {
        OrderKind::Attack(_) if !unit.may_attack() => None,
        _ => Some(kind),
    }
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

/// `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState > AwaitingTarget[..] resolution`: the order a
/// left-click on `target` gives while `pending` awaits it, if the target is valid. Attack takes any destructible
/// object, an own or a neutral one too (force-attack, `systems/controls/commands.md > Attack`, design changes 0059 and
/// 0064), one the player remembers too (design change 0139), and the ground, for an AttackMove. AttackGround takes the
/// ground only (`systems/controls/selection_and_hud.md > AwaitingTarget[AttackGround] resolution`: "Left-click Ground:
/// issues AttackGround command to that location"; slice M4-O), wherever the click falls on it, over an object the
/// player doesn't see too. The Agent's Drop Off takes an own drop-off in operation ([`drop_off_target`], slice M4-J1;
/// `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`: "Requires clicking an own Tunnel"). Reverse has
/// no order in the sim yet.
pub fn target_order(pending: PanelCommand, target: &MapTarget) -> Option<OrderKind> {
    use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
    match (pending, target.target) {
        (PanelCommand::Attack, Ground(point)) => Some(OrderKind::AttackMove(point)),
        (PanelCommand::Attack, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            target.destructible().then_some(OrderKind::Attack(id))
        }
        (PanelCommand::Move, Ground(point)) => Some(OrderKind::Move(Target::Point(point))),
        (PanelCommand::Move, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            Some(OrderKind::Move(Target::Object(id)))
        }
        (PanelCommand::Patrol, Ground(point)) => Some(OrderKind::Patrol(point)),
        (PanelCommand::AttackGround, Ground(point)) => Some(OrderKind::AttackGround(point)),
        (PanelCommand::DropOff, _) => drop_off_target(target).map(OrderKind::DropOff),
        _ => None,
    }
}

/// `systems/controls/selection_and_hud.md > AwaitingTarget[..] resolution` for any PendingCommand: what a left-click on
/// `target` gives, if the target is valid. A unit command's order ([`target_order`]); the rally point of Set Rally
/// Point, which takes the ground and any object (`factions/gdo/structures/barracks.md`: "Left-click ground or object
/// sets the rally point"; `systems/controls/selection_and_hud.md > Standard Slot Assignments > C`: the building itself
/// clears it); the Drones' new drop-off for Set Drop-off ([`drop_off_target`]).
pub fn target_resolution(pending: PanelCommand, target: &MapTarget) -> Option<Resolved> {
    match pending {
        PanelCommand::SetRallyPoint => Some(Resolved::Rally(rally_target(target))),
        PanelCommand::SetDropOff => drop_off_target(target).map(|node| Resolved::Ability(Ability::SetDropOff(node))),
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

/// `systems/controls/selection_and_hud.md > PointerDisplayType > Types` that the game can show so far.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerDisplay {
    /// Nothing will happen on a click.
    Inactive,
    Move,
    Attack,
    AttackGround,
    Patrol,
    /// "Unit will gather from this resource" (slice M3-J2).
    GatherResources,
    /// "Unit will return carried resources to this drop-off point" (slice M3-J2).
    ReturnResources,
    /// "Unit will enter this object" (slice M4-J1): "Cursor over own Tunnel (Syndicate unit, tier sufficient)", the
    /// sim's Enter (`sc_sim::OrderKind::Enter`): an Agent that carries nothing, or any unit the Tunnel's tier lets
    /// into the network, over an own Tunnel in operation; never over a site, which takes no Enter (design change 0237).
    Enter,
    /// `systems/controls/selection_and_hud.md > Mixed Right-Click Preview` (design change 0063): the selected units
    /// would do different things on a right-click.
    Mixed,
}

impl PointerDisplay {
    /// The PointerDisplayType of an order a click gives: Attack for an AttackMove too
    /// (`systems/controls/selection_and_hud.md > Resolution Rules > AwaitingTarget[CommandType]`: Attack over ground
    /// "will issue AttackMove"); GatherResources for a Gather ("Cursor over resource (resource gatherer selected)"),
    /// ReturnResources for a Drop Off ("Cursor over drop-off point (resource gatherer carrying resources)"), Enter for
    /// an Enter ("Cursor over own Tunnel (Syndicate unit, tier sufficient)"). No click gives Hold or Stop.
    pub fn of_order(kind: OrderKind) -> Self {
        match kind {
            // No click gives a Deploy (slice M3-J1 places it), nor a Build.
            OrderKind::Move(_) | OrderKind::Deploy { .. } | OrderKind::Build { .. } => PointerDisplay::Move,
            OrderKind::Enter(_) => PointerDisplay::Enter,
            OrderKind::Gather(_) => PointerDisplay::GatherResources,
            OrderKind::DropOff(_) => PointerDisplay::ReturnResources,
            OrderKind::Attack(_) | OrderKind::AttackMove(_) => PointerDisplay::Attack,
            OrderKind::Patrol(_) => PointerDisplay::Patrol,
            OrderKind::AttackGround(_) => PointerDisplay::AttackGround,
            OrderKind::Hold | OrderKind::Stop => PointerDisplay::Inactive,
        }
    }

    /// The PointerDisplayType of what a click gives an object: its order's, or Move for a rally point
    /// (`systems/controls/selection_and_hud.md > Resolution Rules > DefaultState`: "Selection is a production building:
    /// Move (rally point)"; `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType]`:
    /// "AwaitingTarget\[SetRallyPoint\] over ground or object: Move"). Set Drop-off over a drop-off shows
    /// ReturnResources: **placeholder**, the rules have no AwaitingTarget\[SetDropOffLocation\] row, and its target is
    /// the drop-off point the loads will return to.
    pub fn of(resolved: Resolved) -> Self {
        match resolved {
            Resolved::Order(kind) => Self::of_order(kind),
            Resolved::Rally(_) => PointerDisplay::Move,
            Resolved::Ability(Ability::SetDropOff(_)) => PointerDisplay::ReturnResources,
            Resolved::Ability(_) => PointerDisplay::Inactive,
        }
    }
}

/// `systems/controls/selection_and_hud.md > PointerDisplayType > Resolution Rules` over `target`, for the selected
/// `units` (none: nothing commandable is selected, `systems/controls/selection_and_hud.md > CommandPanel`). In a build
/// menu, or a Tunnel's EjectMenu or ExpandMenu, a right-click only goes back: Inactive (placeholder: the rules list no
/// menu). AwaitingPlacement has "No
/// pointer display type — the building ghost preview serves as the cursor" (`super::cursor` shows none): Inactive here.
pub fn pointer_display(interface: InterfaceState, units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    if units.is_empty() {
        return PointerDisplay::Inactive;
    }
    match interface {
        InterfaceState::DefaultState => right_click_display(units, target),
        InterfaceState::AwaitingTarget { pending, .. } => awaiting_display(pending, target),
        InterfaceState::BuildMenu(_)
        | InterfaceState::EjectMenu
        | InterfaceState::ExpandMenu
        | InterfaceState::AwaitingPlacement(_) => PointerDisplay::Inactive,
    }
}

/// `systems/controls/selection_and_hud.md > Resolution Rules > DefaultState` and
/// `systems/controls/selection_and_hud.md > Mixed Right-Click Preview` (design change 0063): the right-click preview,
/// from the action each of `units` would take ([`object_right_click`]). One action shows its pointer, more than one
/// Mixed, none Inactive. An action is a PointerDisplayType, so a Move to the location, a Follow and a rally point are
/// all Move.
pub fn right_click_display(units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    let actions: BTreeSet<PointerDisplay> = units
        .iter()
        .filter_map(|&unit| object_right_click(unit, target))
        .map(PointerDisplay::of)
        .collect();
    let mut actions = actions.into_iter();
    match (actions.next(), actions.next()) {
        (None, _) => PointerDisplay::Inactive,
        (Some(action), None) => action,
        (Some(_), Some(_)) => PointerDisplay::Mixed,
    }
}

/// `systems/controls/selection_and_hud.md > Resolution Rules > AwaitingTarget[CommandType]`: the pending command's look
/// over a target it can use, else Inactive. Reverse, which has no order yet, by the design's row for the ground.
fn awaiting_display(pending: PanelCommand, target: &MapTarget) -> PointerDisplay {
    match (pending, target.target) {
        (PanelCommand::Reverse, CursorTarget::Ground(_)) => PointerDisplay::Move,
        _ => target_resolution(pending, target).map_or(PointerDisplay::Inactive, PointerDisplay::of),
    }
}

#[cfg(test)]
mod tests;
