//! What a click on the map resolves to, and how the pointer previews it (`systems/controls/selection_and_hud.md >
//! CursorTarget`, `> RightClickResolution`, `> AwaitingTarget`, `> PointerDisplayType`), as plain data. Unit tested
//! here; the systems apply it to clicks (`systems.rs`) and to the pointer ([`super::cursor`]).
//!
//! - **Right-click** (design changes 0062 and 0063): each selected unit resolves it for itself, by its own template
//!   and capabilities ([`unit_right_click`]), whichever group is active. A unit that can't carry out the resolved
//!   command moves to the location or to the object instead, following a unit ([`move_to`]); a unit whose Riot
//!   Shield keeps it from attacking moves to the clicked location ([`order_without_attack`], question Q0020 point 1).
//!   The orders go out grouped, one per distinct order ([`group_orders`]).
//! - **Left-click while a target is awaited**: the PendingCommand's order for the target ([`target_order`]); Attack
//!   takes any destructible object, own and neutral ones too (design changes 0059 and 0064).
//! - **The pointer** ([`pointer_display`]): in DefaultState the one thing the selected units would all do on a
//!   right-click, or Mixed when they would do different things; while a target is awaited, the pending command's
//!   look over a valid target, else Inactive.

use std::collections::BTreeSet;

use bevy::math::Vec2;
use sc_sim::combat::{domain_allows, object_domain};
use sc_sim::defs::{Domain, TargetDomain};
use sc_sim::{ObjectId, ObjectKind, OrderKind, Sim, Target};

use super::interface::{InterfaceState, InterfaceTemplate, Modes, UnitCommand};
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
/// the right-click for itself, against its own capabilities, not the ActiveGroup's").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnitCapabilities {
    /// Its type's ObjectInterfaceState template, which resolves the right-click; none for a type without commands,
    /// which a right-click leaves alone (design change 0062: "Buildings in the selection aren't affected").
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

/// `> RightClickResolution` by a template: the command a right-click on `target` resolves to, and its order.
pub fn right_click(template: InterfaceTemplate, target: CursorTarget) -> Option<(UnitCommand, OrderKind)> {
    match template {
        // `> BasicCombatUnitInterfaceState > RightClickResolution`. Enter (own Tunnel, Altar, Garrison, Bunker): none
        // of those exists yet.
        InterfaceTemplate::BasicCombatUnit => Some(match target {
            CursorTarget::EnemyObject(id) => (UnitCommand::Attack, OrderKind::Attack(id)),
            CursorTarget::Ground(point) => (UnitCommand::Move, OrderKind::Move(Target::Point(point))),
            CursorTarget::FriendlyObject(id) | CursorTarget::NeutralObject(id) => {
                (UnitCommand::Move, OrderKind::Move(Target::Object(id)))
            }
        }),
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
        UnitCommand::Attack if !unit.can_attack(target) => move_to(target),
        UnitCommand::Attack if !unit.may_attack => order_without_attack(kind, target.ground),
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

/// The Riot Shield's rule (question Q0020 point 1, its default until answered): the order a unit whose shield keeps
/// it from attacking gets in place of `kind`, for a click at the ground point `ground`. An Attack becomes a Move to
/// that location (`factions/gdo/units.md > Peacekeeper > Riot Shield`: "Right-click on an enemy while the shield is
/// up issues a Move order to that location instead of an Attack"), which wins over design change 0062's Follow.
/// This is the one place of that rule: for 0062's Follow, make an `Attack(id)` a `Move(Target::Object(id))` here.
/// Any other order stays as it is. A unit that couldn't attack the target anyway follows it like the others
/// ([`unit_right_click`]). Placeholder: the rule holds for a left-click in AwaitingTarget[Attack] too; the design
/// speaks of the right-click only.
pub fn order_without_attack(kind: OrderKind, ground: Vec2) -> OrderKind {
    match kind {
        OrderKind::Attack(_) => OrderKind::Move(Target::Point(ground)),
        other => other,
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
pub fn target_order(pending: UnitCommand, target: &MapTarget) -> Option<OrderKind> {
    use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
    match (pending, target.target) {
        (UnitCommand::Attack, Ground(point)) => Some(OrderKind::AttackMove(point)),
        (UnitCommand::Attack, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            target.destructible.then_some(OrderKind::Attack(id))
        }
        (UnitCommand::Move, Ground(point)) => Some(OrderKind::Move(Target::Point(point))),
        (UnitCommand::Move, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            Some(OrderKind::Move(Target::Object(id)))
        }
        (UnitCommand::Patrol, Ground(point)) => Some(OrderKind::Patrol(point)),
        _ => None,
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
            OrderKind::Move(_) => PointerDisplay::Move,
            OrderKind::Attack(_) | OrderKind::AttackMove(_) => PointerDisplay::Attack,
            OrderKind::Patrol(_) => PointerDisplay::Patrol,
            OrderKind::Hold | OrderKind::Stop => PointerDisplay::Inactive,
        }
    }
}

/// `> PointerDisplayType > Resolution Rules` over `target`, for the selected `units` (none: nothing commandable is
/// selected, `> CommandPanel`).
pub fn pointer_display(interface: InterfaceState, units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    if units.is_empty() {
        return PointerDisplay::Inactive;
    }
    match interface {
        InterfaceState::DefaultState => right_click_display(units, target),
        InterfaceState::AwaitingTarget(pending) => awaiting_display(pending, target),
    }
}

/// `> Resolution Rules > DefaultState` and `> Mixed Right-Click Preview` (design change 0063): the right-click
/// preview, from the action each of `units` would take ([`unit_right_click`]). One action shows its pointer, more
/// than one Mixed, none Inactive. An action is a PointerDisplayType, so a Move to the location and a Follow are both
/// Move.
pub fn right_click_display(units: &[UnitCapabilities], target: &MapTarget) -> PointerDisplay {
    let actions: BTreeSet<PointerDisplay> = units
        .iter()
        .filter_map(|&unit| unit_right_click(unit, target))
        .map(PointerDisplay::of_order)
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
fn awaiting_display(pending: UnitCommand, target: &MapTarget) -> PointerDisplay {
    match (pending, target.target) {
        (UnitCommand::AttackGround, CursorTarget::Ground(_)) => PointerDisplay::AttackGround,
        (UnitCommand::Reverse, CursorTarget::Ground(_)) => PointerDisplay::Move,
        _ => target_order(pending, target).map_or(PointerDisplay::Inactive, PointerDisplay::of_order),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u32) -> ObjectId {
        ObjectId(n)
    }

    const P: Vec2 = Vec2::new(300.0, 200.0);

    /// `target` under the cursor at [`P`]: an object destructible or not, in `domain`.
    fn on(target: CursorTarget, destructible: bool, domain: Domain) -> MapTarget {
        MapTarget {
            target,
            ground: P,
            destructible,
            domain,
        }
    }

    /// An enemy ground unit, an own one, a neutral one, a crystal patch and an enemy aircraft.
    fn enemy() -> MapTarget {
        on(CursorTarget::EnemyObject(id(6)), true, Domain::Ground)
    }
    fn own() -> MapTarget {
        on(CursorTarget::FriendlyObject(id(2)), true, Domain::Ground)
    }
    fn neutral_unit() -> MapTarget {
        on(CursorTarget::NeutralObject(id(9)), true, Domain::Ground)
    }
    fn patch() -> MapTarget {
        on(CursorTarget::NeutralObject(id(8)), false, Domain::Ground)
    }
    fn aircraft() -> MapTarget {
        on(CursorTarget::EnemyObject(id(7)), true, Domain::Air)
    }

    /// A combat unit with a ground attack, the Peacekeeper's and the Guard's.
    const ARMED: UnitCapabilities = UnitCapabilities {
        template: Some(InterfaceTemplate::BasicCombatUnit),
        attack: Some(TargetDomain::Ground),
        may_attack: true,
    };
    /// The same with its Riot Shield up.
    const SHIELDED: UnitCapabilities = UnitCapabilities {
        may_attack: false,
        ..ARMED
    };
    /// One without an attack, like the design's Supply Chopper (none exists yet).
    const UNARMED: UnitCapabilities = UnitCapabilities { attack: None, ..ARMED };
    /// One that attacks anything.
    const UNIVERSAL: UnitCapabilities = UnitCapabilities {
        attack: Some(TargetDomain::Universal),
        ..ARMED
    };
    /// An object without commands: a building.
    const BUILDING: UnitCapabilities = UnitCapabilities {
        template: None,
        attack: None,
        may_attack: true,
    };

    fn object(n: u32) -> OrderKind {
        OrderKind::Move(Target::Object(id(n)))
    }

    /// `> BasicCombatUnitInterfaceState > RightClickResolution` and `> AwaitingTarget[..] resolution`.
    #[test]
    fn clicks_resolve_by_what_is_under_the_cursor() {
        use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
        let template = InterfaceTemplate::BasicCombatUnit;
        let right = |target| right_click(template, target).map(|(_, kind)| kind);
        assert_eq!(right(Ground(P)), Some(OrderKind::Move(Target::Point(P))));
        assert_eq!(right(EnemyObject(id(6))), Some(OrderKind::Attack(id(6))));
        assert_eq!(right(FriendlyObject(id(2))), Some(object(2)));
        assert_eq!(right(NeutralObject(id(8))), Some(object(8)));

        let ground = MapTarget::ground(P);
        let left = |pending, target: MapTarget| target_order(pending, &target);
        assert_eq!(left(UnitCommand::Attack, enemy()), Some(OrderKind::Attack(id(6))));
        assert_eq!(left(UnitCommand::Attack, ground), Some(OrderKind::AttackMove(P)));
        // Force-attack (0059, 0064): own and neutral destructible objects too, never an indestructible one.
        assert_eq!(left(UnitCommand::Attack, own()), Some(OrderKind::Attack(id(2))));
        assert_eq!(
            left(UnitCommand::Attack, neutral_unit()),
            Some(OrderKind::Attack(id(9)))
        );
        assert_eq!(left(UnitCommand::Attack, patch()), None);
        assert_eq!(left(UnitCommand::Move, ground), Some(OrderKind::Move(Target::Point(P))));
        assert_eq!(left(UnitCommand::Move, enemy()), Some(object(6)));
        assert_eq!(left(UnitCommand::Move, patch()), Some(object(8)));
        assert_eq!(left(UnitCommand::Patrol, ground), Some(OrderKind::Patrol(P)));
        assert_eq!(left(UnitCommand::Patrol, enemy()), None);
        assert_eq!(left(UnitCommand::AttackGround, ground), None, "no sim order yet");

        // With the Riot Shield up, an Attack becomes a Move to the clicked point; the other orders stay.
        assert_eq!(
            order_without_attack(OrderKind::Attack(id(6)), P),
            OrderKind::Move(Target::Point(P))
        );
        for kind in [OrderKind::AttackMove(P), object(6), OrderKind::Stop] {
            assert_eq!(order_without_attack(kind, Vec2::ZERO), kind);
        }
    }

    /// Design changes 0062 and 0063, with question Q0020 point 1's default: every unit resolves a right-click for
    /// itself. On an enemy, a unit that can attack it attacks; one that can't (no attack, or not in the target's
    /// domain) follows it; one whose shield is up moves to the clicked spot. The ground, own and neutral objects
    /// give everyone a Move. An object without commands does nothing.
    #[test]
    fn each_unit_resolves_a_right_click_for_itself() {
        let attack = OrderKind::Attack(id(6));
        let spot = OrderKind::Move(Target::Point(P));
        let resolved = |unit, target: MapTarget| unit_right_click(unit, &target);
        assert_eq!(resolved(ARMED, enemy()), Some(attack));
        assert_eq!(resolved(UNARMED, enemy()), Some(object(6)), "follows");
        assert_eq!(resolved(SHIELDED, enemy()), Some(spot), "moves to the spot");
        assert_eq!(resolved(BUILDING, enemy()), None);
        // 0062: "ground-only Peacekeepers when the target is an aircraft" follow it.
        assert_eq!(resolved(ARMED, aircraft()), Some(object(7)));
        assert_eq!(resolved(UNIVERSAL, aircraft()), Some(OrderKind::Attack(id(7))));
        // A shielded unit that couldn't attack the target anyway follows it, like the others that can't.
        assert_eq!(resolved(SHIELDED, aircraft()), Some(object(7)));
        for unit in [ARMED, UNARMED, SHIELDED] {
            assert_eq!(resolved(unit, MapTarget::ground(P)), Some(spot));
            assert_eq!(
                resolved(unit, own()),
                Some(object(2)),
                "0059: a right-click never attacks"
            );
            assert_eq!(resolved(unit, neutral_unit()), Some(object(9)));
            assert_eq!(resolved(unit, patch()), Some(object(8)));
        }
        assert_eq!(resolved(BUILDING, MapTarget::ground(P)), None);
    }

    /// One order per distinct order, in the order each first comes, each to its units in their order.
    #[test]
    fn orders_go_out_grouped() {
        let attack = OrderKind::Attack(id(6));
        let spot = OrderKind::Move(Target::Point(P));
        let orders = [(1, spot), (2, attack), (3, object(6)), (4, attack), (5, spot)].map(|(n, kind)| (id(n), kind));
        assert_eq!(
            group_orders(orders),
            [
                (spot, vec![id(1), id(5)]),
                (attack, vec![id(2), id(4)]),
                (object(6), vec![id(3)])
            ]
        );
        assert_eq!(group_orders([]), []);
    }

    /// `> PointerDisplayType > Resolution Rules`.
    #[test]
    fn the_pointer_previews_what_a_click_would_do() {
        use PointerDisplay as D;
        let unit = [ARMED];
        let ground = MapTarget::ground(Vec2::ZERO);
        let default = InterfaceState::DefaultState;
        assert_eq!(pointer_display(default, &[], &enemy()), D::Inactive, "nothing selected");
        assert_eq!(pointer_display(default, &unit, &enemy()), D::Attack);
        assert_eq!(pointer_display(default, &unit, &ground), D::Move);
        assert_eq!(pointer_display(default, &unit, &own()), D::Move);
        assert_eq!(pointer_display(default, &unit, &patch()), D::Move);
        assert_eq!(
            pointer_display(default, &[SHIELDED], &enemy()),
            D::Move,
            "shields up: a right-click moves there"
        );
        let awaiting = InterfaceState::AwaitingTarget;
        assert_eq!(
            pointer_display(awaiting(UnitCommand::Attack), &unit, &ground),
            D::Attack
        );
        assert_eq!(
            pointer_display(awaiting(UnitCommand::Attack), &unit, &enemy()),
            D::Attack
        );
        // 0064: force-attack on own and neutral destructible objects; an indestructible one is no target.
        assert_eq!(pointer_display(awaiting(UnitCommand::Attack), &unit, &own()), D::Attack);
        assert_eq!(
            pointer_display(awaiting(UnitCommand::Attack), &unit, &neutral_unit()),
            D::Attack
        );
        assert_eq!(
            pointer_display(awaiting(UnitCommand::Attack), &unit, &patch()),
            D::Inactive
        );
        assert_eq!(pointer_display(awaiting(UnitCommand::Move), &unit, &enemy()), D::Move);
        assert_eq!(
            pointer_display(awaiting(UnitCommand::Patrol), &unit, &ground),
            D::Patrol
        );
        assert_eq!(
            pointer_display(awaiting(UnitCommand::Patrol), &unit, &patch()),
            D::Inactive
        );
        assert_eq!(
            pointer_display(awaiting(UnitCommand::AttackGround), &unit, &ground),
            D::AttackGround
        );
        assert_eq!(pointer_display(awaiting(UnitCommand::Reverse), &unit, &ground), D::Move);
        assert_eq!(
            pointer_display(awaiting(UnitCommand::Reverse), &unit, &enemy()),
            D::Inactive
        );
    }

    /// `> Mixed Right-Click Preview` (design change 0063): the pointer shows what the units would do, or Mixed when
    /// they would do different things. Moving to a spot and following are both Move.
    #[test]
    fn units_that_would_do_different_things_show_the_mixed_pointer() {
        use PointerDisplay as D;
        let preview = |units: &[UnitCapabilities], target: MapTarget| right_click_display(units, &target);
        assert_eq!(preview(&[ARMED, ARMED], enemy()), D::Attack);
        assert_eq!(preview(&[ARMED, UNARMED], enemy()), D::Mixed);
        assert_eq!(preview(&[SHIELDED, ARMED], enemy()), D::Mixed);
        assert_eq!(preview(&[SHIELDED, UNARMED], enemy()), D::Move);
        assert_eq!(preview(&[ARMED, UNARMED, BUILDING], MapTarget::ground(P)), D::Move);
        assert_eq!(
            preview(&[ARMED, BUILDING], enemy()),
            D::Attack,
            "a building does nothing"
        );
        assert_eq!(preview(&[BUILDING], enemy()), D::Inactive);
        assert_eq!(preview(&[], enemy()), D::Inactive);
    }
}
