//! What a click on the map resolves to, and how the pointer previews it (`systems/controls/selection_and_hud.md >
//! CursorTarget`, `> RightClickResolution`, `> AwaitingTarget`, `> PointerDisplayType`), as plain data. Unit tested
//! here; the systems apply it to clicks (`systems.rs`) and to the pointer ([`super::cursor`]).
//!
//! - **Right-click** (design changes 0062 and 0063): each selected object resolves it for itself, by its own
//!   template and capabilities ([`object_right_click`]), whichever group is active. A unit gets an order
//!   ([`unit_right_click`]): one that can't carry out the resolved command moves to the location or to the object
//!   instead, following a unit ([`move_to`]); so does a unit whose Riot Shield keeps it from attacking
//!   ([`order_without_attack`], design change 0074). The orders go out grouped, one per distinct order
//!   ([`group_orders`]). A producing structure sets its rally point on what was clicked ([`rally_target`],
//!   `> Production Building Default Right-Click`): the ground, any object, or itself, which clears it (design change
//!   0086).
//! - **Left-click while a target is awaited** ([`target_resolution`]): the PendingCommand's order for the target
//!   ([`target_order`]); Attack takes any destructible object, own and neutral ones too (design changes 0059 and
//!   0064). It goes to every unit of the command's recipients, except that an explicit Attack leaves out the units
//!   whose Riot Shield keeps them from attacking ([`unit_target_order`], design change 0075); an AttackMove goes to
//!   all of them (design change 0076). Set Rally Point takes the ground or any object, as the right-click does.
//! - **The pointer** ([`pointer_display`]): in DefaultState the one thing the selected objects would all do on a
//!   right-click, or Mixed when they would do different things (a rally point shows Move: `> Resolution Rules >
//!   DefaultState`, "Selection is a production building: Move (rally point)"); while a target is awaited, the
//!   pending command's look over a valid target, else Inactive.

use std::collections::BTreeSet;

use bevy::math::Vec2;
use sc_sim::combat::{domain_allows, object_domain};
use sc_sim::defs::{Domain, TargetDomain};
use sc_sim::{ObjectId, ObjectKind, OrderKind, Sim, Target};

use super::interface::{InterfaceState, InterfaceTemplate, Modes, PanelCommand};
use crate::selection::{Relation, Selectables, Selection};

/// `> CursorTarget`: what is under the cursor when it clicks.
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
    /// What is under the view point `view`, whose ground point is `ground`: the object a click there picks
    /// (`Selectables::at`, as for selecting), else the ground.
    pub fn at(objects: &Selectables, view: Vec2, ground: Vec2) -> Self {
        match objects.at(view) {
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
/// the cursor, and what an attack would need to know of the object there.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MapTarget {
    pub target: CursorTarget,
    /// The ground point under the cursor: where a Move to the location goes.
    pub ground: Vec2,
    /// The object can be attacked: its type is destructible (`systems/controls/commands.md > Attack`: "a specific
    /// destructible object instance"). False over the ground.
    pub destructible: bool,
    /// The domain the object is targeted in (`sc_sim::combat::object_domain`; Ground for the ground, and for a
    /// structure the player remembers).
    pub domain: Domain,
}

impl MapTarget {
    /// What is under the view point `view`, whose ground point is `ground`: the object a click there picks among
    /// `objects` ([`CursorTarget::at`]), as `sim` has it, else the ground.
    pub fn at(sim: &Sim, objects: &Selectables, view: Vec2, ground: Vec2) -> Self {
        let target = CursorTarget::at(objects, view, ground);
        let kind = target.object().and_then(|id| objects.get(id)).map(|o| o.kind);
        let domain = target
            .object()
            .and_then(|id| sim.object(id))
            .map_or(Domain::Ground, object_domain);
        Self {
            target,
            ground,
            destructible: kind.is_some_and(|kind| kind.def().destructible),
            domain,
        }
    }

    /// The ground at `point`.
    pub fn ground(point: Vec2) -> Self {
        Self {
            target: CursorTarget::Ground(point),
            ground: point,
            destructible: false,
            domain: Domain::Ground,
        }
    }
}

/// A selected object as a right-click resolves it for itself (`> RightClickResolution`: "Each selected unit resolves
/// the right-click for itself, against its own capabilities, not the ActiveGroup's"): a unit, or a producing
/// structure, whose right-click sets its rally point.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnitCapabilities {
    /// Its type's ObjectInterfaceState template, which resolves the right-click; none for a type without commands,
    /// which a right-click leaves alone (design change 0062: "Buildings in the selection aren't affected"). A
    /// producing structure has its own (`> Production Building Default Right-Click`): **placeholder reading**, 0062
    /// is read as the buildings without a right-click of their own, so a Barracks selected with units sets its rally
    /// point as the units take their orders.
    pub template: Option<InterfaceTemplate>,
    /// The TargetDomain of its type's attack (`systems/combat.md > AttackAttributes`); none without an attack.
    pub attack: Option<TargetDomain>,
    /// Its mode lets it attack now: not while its Riot Shield is up or on its way ([`Modes::can_attack`]).
    pub may_attack: bool,
}

impl UnitCapabilities {
    /// Those of the selected object `id`, of type `kind`, whose mode is in `modes`.
    pub fn of(id: ObjectId, kind: ObjectKind, modes: &Modes) -> Self {
        Self {
            template: InterfaceTemplate::of(kind),
            attack: kind
                .def()
                .unit
                .as_ref()
                .and_then(|unit| unit.attack)
                .map(|attack| attack.target_domain),
            may_attack: modes.can_attack(id),
        }
    }

    /// Whether it could attack the object under `target`, whatever its mode: it has an attack, the object is
    /// destructible, and the attack's TargetDomain allows the object's domain (`systems/combat.md > ValidTarget` 3;
    /// the cursor reaches only objects the player sees, `> ValidTarget` 2).
    pub fn can_attack(&self, target: &MapTarget) -> bool {
        target.target.object().is_some()
            && target.destructible
            && self.attack.is_some_and(|domain| domain_allows(domain, target.domain))
    }
}

/// The selected objects with their capabilities, in id order.
pub fn capabilities(selection: &Selection, modes: &Modes) -> Vec<(ObjectId, UnitCapabilities)> {
    selection
        .members()
        .map(|(id, kind)| (id, UnitCapabilities::of(id, kind, modes)))
        .collect()
}

/// `> RightClickResolution` by a template: the command a right-click on `target` resolves to, and its order. A
/// producing structure's right-click gives no order but its rally point ([`object_right_click`]); a constructor's
/// gives nothing (it produces no units: `> Production Building Default Right-Click` isn't its).
pub fn right_click(template: InterfaceTemplate, target: CursorTarget) -> Option<(PanelCommand, OrderKind)> {
    match template {
        // `> BasicCombatUnitInterfaceState > RightClickResolution`. Enter (own Tunnel, Altar, Garrison, Bunker): none
        // of those exists yet.
        InterfaceTemplate::BasicCombatUnit => Some(match target {
            CursorTarget::EnemyObject(id) => (PanelCommand::Attack, OrderKind::Attack(id)),
            CursorTarget::Ground(point) => (PanelCommand::Move, OrderKind::Move(Target::Point(point))),
            CursorTarget::FriendlyObject(id) | CursorTarget::NeutralObject(id) => {
                (PanelCommand::Move, OrderKind::Move(Target::Object(id)))
            }
        }),
        InterfaceTemplate::Production | InterfaceTemplate::DeploymentCenter | InterfaceTemplate::ExtractionFacility => {
            None
        }
    }
}

/// What a click gives one selected object: a unit's order, or a producer's rally point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Resolved {
    /// A unit's order (`Command::Order`).
    Order(OrderKind),
    /// A producer's rally point (`ProductionCommand::SetRally`): the ground or an object, the producer itself to
    /// clear it (design change 0086).
    Rally(Target),
}

/// `> RightClickResolution` for one selected object: a unit's order ([`unit_right_click`]), or, for a producing
/// structure, its rally point on what is under the cursor (`> Production Building Default Right-Click`: "Right-click
/// Ground: sets rally point to that location; Right-click Object: sets rally point to that object; Right-click the
/// building itself: clears the rally point"). Nothing for a constructor (it produces no units), nor for an object
/// without a template.
pub fn object_right_click(object: UnitCapabilities, target: &MapTarget) -> Option<Resolved> {
    match object.template? {
        InterfaceTemplate::BasicCombatUnit => unit_right_click(object, target).map(Resolved::Order),
        InterfaceTemplate::Production => Some(Resolved::Rally(rally_target(target))),
        InterfaceTemplate::DeploymentCenter | InterfaceTemplate::ExtractionFacility => None,
    }
}

/// The rally point a click on `target` sets (`> Production Building Default Right-Click`, `> Standard Slot
/// Assignments > C`): the ground point, or the object, whoever owns it. Which objects the player may rally on, the
/// sim decides (`Sim::validate`): one it sees, the producer itself (which clears the rally point, design change
/// 0086), and one it remembers in the fog, which becomes a ground rally point (the sim's placeholder for Q0017).
pub fn rally_target(target: &MapTarget) -> Target {
    match target.target.object() {
        Some(id) => Target::Object(id),
        None => Target::Point(target.ground),
    }
}

/// `> RightClickResolution` for one selected unit (design changes 0062 and 0063): the order `unit` gets for a
/// right-click on `target`. Its template resolves the command ([`right_click`]); a unit that can carry it out gets
/// its order. One that can't moves to the target instead ([`move_to`]); one kept from an Attack only by its Riot
/// Shield gets [`order_without_attack`]. Every unit can move, so a Move is always carried out. Nothing for an object
/// without a template.
pub fn unit_right_click(unit: UnitCapabilities, target: &MapTarget) -> Option<OrderKind> {
    let (command, kind) = right_click(unit.template?, target.target)?;
    Some(match command {
        PanelCommand::Attack if !unit.can_attack(target) => move_to(target),
        PanelCommand::Attack if !unit.may_attack => order_without_attack(target),
        _ => kind,
    })
}

/// Design change 0062: what a unit that can't carry out a right-click's command does instead. It moves to the
/// location, or to the object, which follows a unit (`systems/controls/behaviors.md > MovingToObject`), so that the
/// group stays together.
pub fn move_to(target: &MapTarget) -> OrderKind {
    OrderKind::Move(match target.target.object() {
        Some(id) => Target::Object(id),
        None => Target::Point(target.ground),
    })
}

/// The Riot Shield's right-click rule (design change 0074, `factions/gdo/units.md > Peacekeeper > Riot Shield`:
/// "Right-click on an enemy while the shield is up"): the order a unit whose shield keeps it from attacking gets for a
/// right-click that would attack `target`. It does what any unit that can't carry out a right-click does (design
/// change 0062, [`move_to`]): it follows an enemy unit, and moves to an enemy structure, so that in a mixed group the
/// shield wall stays with the units that attack. The one place of that rule ([`unit_right_click`]). An explicit
/// Attack has its own: it leaves such a unit out ([`unit_target_order`], design change 0075).
pub fn order_without_attack(target: &MapTarget) -> OrderKind {
    move_to(target)
}

/// The order one of a targeting click's recipients takes, when the click gives `kind` ([`target_order`]): design
/// change 0075 (`factions/gdo/units.md > Peacekeeper > Riot Shield > Explicit Attack`): a unit whose mode keeps it
/// from attacking (its Riot Shield up or on its way) is left out of an explicit Attack, on any object, and keeps its
/// orders; the units that can attack carry it out. Every other order goes to every recipient, an AttackMove too
/// (design change 0076, `systems/controls/commands.md > AttackMove > Units that can't attack`: it plays out as a
/// move for a unit that can't attack). Unlike the right-click ([`order_without_attack`]), on purpose.
pub fn unit_target_order(unit: UnitCapabilities, kind: OrderKind) -> Option<OrderKind> {
    match kind {
        OrderKind::Attack(_) if !unit.may_attack => None,
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

/// `> BasicCombatUnitInterfaceState > AwaitingTarget[..] resolution`: the order a left-click on `target` gives
/// while `pending` awaits it, if the target is valid. Attack takes any destructible object, an own or a neutral one
/// too (force-attack, `systems/controls/commands.md > Attack`, design changes 0059 and 0064), and the ground, for an
/// AttackMove. Reverse and AttackGround have no order in the sim yet.
pub fn target_order(pending: PanelCommand, target: &MapTarget) -> Option<OrderKind> {
    use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
    match (pending, target.target) {
        (PanelCommand::Attack, Ground(point)) => Some(OrderKind::AttackMove(point)),
        (PanelCommand::Attack, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            target.destructible.then_some(OrderKind::Attack(id))
        }
        (PanelCommand::Move, Ground(point)) => Some(OrderKind::Move(Target::Point(point))),
        (PanelCommand::Move, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            Some(OrderKind::Move(Target::Object(id)))
        }
        (PanelCommand::Patrol, Ground(point)) => Some(OrderKind::Patrol(point)),
        _ => None,
    }
}

/// `> AwaitingTarget[..] resolution` for any PendingCommand: what a left-click on `target` gives, if the target is
/// valid. A unit command's order ([`target_order`]), or the rally point of Set Rally Point, which takes the ground
/// and any object (`factions/gdo/structures.md > Barracks`: "Left-click ground or object sets the rally point";
/// `> Standard Slot Assignments > C`: the building itself clears it).
pub fn target_resolution(pending: PanelCommand, target: &MapTarget) -> Option<Resolved> {
    match pending {
        PanelCommand::SetRallyPoint => Some(Resolved::Rally(rally_target(target))),
        _ => target_order(pending, target).map(Resolved::Order),
    }
}

/// `> PointerDisplayType > Types` that M1 can show. GatherResources, ReturnResources and Enter wait for gatherers
/// and enterable objects.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerDisplay {
    /// Nothing will happen on a click.
    Inactive,
    Move,
    Attack,
    AttackGround,
    Patrol,
    /// `> Mixed Right-Click Preview` (design change 0063): the selected units would do different things on a
    /// right-click.
    Mixed,
}

impl PointerDisplay {
    /// The PointerDisplayType of an order a click gives: Attack for an AttackMove too (`> Resolution Rules >
    /// AwaitingTarget[CommandType]`: Attack over ground "will issue AttackMove"). No click gives Hold or Stop.
    pub fn of_order(kind: OrderKind) -> Self {
        match kind {
            // Slice M3-J2 resolves a worker's right-clicks and gives them their pointers.
            OrderKind::Move(_) | OrderKind::Gather(_) | OrderKind::DropOff(_) => PointerDisplay::Move,
            OrderKind::Attack(_) | OrderKind::AttackMove(_) => PointerDisplay::Attack,
            OrderKind::Patrol(_) => PointerDisplay::Patrol,
            OrderKind::Hold | OrderKind::Stop => PointerDisplay::Inactive,
        }
    }

    /// The PointerDisplayType of what a click gives an object: its order's, or Move for a rally point (`> Resolution
    /// Rules > DefaultState`: "Selection is a production building: Move (rally point)"; `> AwaitingTarget[CommandType]`:
    /// "AwaitingTarget\[SetRallyPoint\] over ground or object: Move").
    pub fn of(resolved: Resolved) -> Self {
        match resolved {
            Resolved::Order(kind) => Self::of_order(kind),
            Resolved::Rally(_) => PointerDisplay::Move,
        }
    }
}

/// `> PointerDisplayType > Resolution Rules` over `target`, for the selected `units` (none: nothing commandable is
/// selected, `> CommandPanel`). In a build menu a right-click only goes back: Inactive (placeholder: the rules list
/// no build menu). AwaitingPlacement has "No pointer display type — the building ghost preview serves as the
/// cursor" (`super::cursor` shows none): Inactive here.
pub fn pointer_display(interface: InterfaceState, units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    if units.is_empty() {
        return PointerDisplay::Inactive;
    }
    match interface {
        InterfaceState::DefaultState => right_click_display(units, target),
        InterfaceState::AwaitingTarget { pending, .. } => awaiting_display(pending, target),
        InterfaceState::BuildMenu(_) | InterfaceState::AwaitingPlacement(_) => PointerDisplay::Inactive,
    }
}

/// `> Resolution Rules > DefaultState` and `> Mixed Right-Click Preview` (design change 0063): the right-click
/// preview, from the action each of `units` would take ([`object_right_click`]). One action shows its pointer, more
/// than one Mixed, none Inactive. An action is a PointerDisplayType, so a Move to the location, a Follow and a
/// rally point are all Move.
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

/// `> Resolution Rules > AwaitingTarget[CommandType]`: the pending command's look over a target it can use, else
/// Inactive. AttackGround and Reverse, which have no order yet, by the design's rows for the ground.
fn awaiting_display(pending: PanelCommand, target: &MapTarget) -> PointerDisplay {
    match (pending, target.target) {
        (PanelCommand::AttackGround, CursorTarget::Ground(_)) => PointerDisplay::AttackGround,
        (PanelCommand::Reverse, CursorTarget::Ground(_)) => PointerDisplay::Move,
        _ => target_resolution(pending, target).map_or(PointerDisplay::Inactive, PointerDisplay::of),
    }
}

#[cfg(test)]
mod tests;
