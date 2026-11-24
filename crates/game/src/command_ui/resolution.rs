//! What a click on the map resolves to, and how the pointer previews it (`systems/controls/selection_and_hud.md >
//! CursorTarget`, `> RightClickResolution`, `> AwaitingTarget`, `> PointerDisplayType`), as plain data. Unit tested
//! here; the systems apply it to clicks (`systems.rs`) and to the pointer ([`super::cursor`]).

use bevy::math::Vec2;
use sc_sim::{ObjectId, OrderKind, Target};

use super::interface::{InterfaceState, InterfaceTemplate, UnitCommand};
use crate::selection::{Relation, Selectables};

/// `> CursorTarget`: what is under the cursor when it clicks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CursorTarget {
    /// The ground, at this point.
    Ground(Vec2),
    EnemyObject(ObjectId),
    /// The player's own objects and their allies'.
    FriendlyObject(ObjectId),
    /// Unowned objects: the map resources.
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
}

/// `> RightClickResolution` from DefaultState, by the ActiveGroup's template: the command and its order.
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

/// `> BasicCombatUnitInterfaceState > AwaitingTarget[..] resolution`: the order a left-click on `target` gives while
/// `pending` awaits it, if the target is valid. Reverse and AttackGround have no order in the sim yet.
pub fn target_order(pending: UnitCommand, target: CursorTarget) -> Option<OrderKind> {
    use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
    match (pending, target) {
        (UnitCommand::Attack, EnemyObject(id)) => Some(OrderKind::Attack(id)),
        (UnitCommand::Attack, Ground(point)) => Some(OrderKind::AttackMove(point)),
        (UnitCommand::Move, Ground(point)) => Some(OrderKind::Move(Target::Point(point))),
        (UnitCommand::Move, EnemyObject(id) | FriendlyObject(id) | NeutralObject(id)) => {
            Some(OrderKind::Move(Target::Object(id)))
        }
        (UnitCommand::Patrol, Ground(point)) => Some(OrderKind::Patrol(point)),
        _ => None,
    }
}

/// `factions/gdo/units.md > Peacekeeper > Riot Shield`: "Right-click on an enemy while the shield is up issues a
/// Move order to that location instead of an Attack". The order a unit that can't attack now gets for a click at
/// `ground` that gives `kind` to the others: a Move to the clicked point for an Attack, else the same. Placeholder:
/// a left-click on an enemy in AwaitingTarget[Attack] too, which the design doesn't mention.
pub fn order_without_attack(kind: OrderKind, ground: Vec2) -> OrderKind {
    match kind {
        OrderKind::Attack(_) => OrderKind::Move(Target::Point(ground)),
        other => other,
    }
}

/// `> PointerDisplayType > Types` that M1 can show. GatherResources, ReturnResources and Enter wait for gatherers
/// and enterable objects.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PointerDisplay {
    /// Nothing will happen on a click.
    Inactive,
    Move,
    Attack,
    AttackGround,
    Patrol,
}

/// `> PointerDisplayType > Resolution Rules` over `target`, for a selection whose ActiveGroup has `template` (none:
/// nothing commandable is selected). `armed`: some unit a right-click's Attack would go to can attack now; when
/// none can (their Riot Shields are up), the right-click on an enemy moves them there ([`order_without_attack`]),
/// and the pointer shows a Move (`> DefaultState`: "Cursor over enemy object (and selection can attack)").
pub fn pointer_display(
    template: Option<InterfaceTemplate>,
    interface: InterfaceState,
    target: CursorTarget,
    armed: bool,
) -> PointerDisplay {
    let Some(template) = template else {
        return PointerDisplay::Inactive;
    };
    match interface {
        // The right-click preview.
        InterfaceState::DefaultState => match right_click(template, target) {
            Some((UnitCommand::Attack, _)) if armed => PointerDisplay::Attack,
            Some(_) => PointerDisplay::Move,
            None => PointerDisplay::Inactive,
        },
        InterfaceState::AwaitingTarget(pending) => match (pending, target) {
            (UnitCommand::Attack, CursorTarget::EnemyObject(_) | CursorTarget::Ground(_)) => PointerDisplay::Attack,
            (UnitCommand::Move, _) => PointerDisplay::Move,
            (UnitCommand::Patrol, CursorTarget::Ground(_)) => PointerDisplay::Patrol,
            (UnitCommand::AttackGround, CursorTarget::Ground(_)) => PointerDisplay::AttackGround,
            (UnitCommand::Reverse, CursorTarget::Ground(_)) => PointerDisplay::Move,
            _ => PointerDisplay::Inactive,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u32) -> ObjectId {
        ObjectId(n)
    }

    /// `> BasicCombatUnitInterfaceState > RightClickResolution` and `> AwaitingTarget[..] resolution`.
    #[test]
    fn clicks_resolve_by_what_is_under_the_cursor() {
        use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
        let p = Vec2::new(300.0, 200.0);
        let template = InterfaceTemplate::BasicCombatUnit;
        let right = |target| right_click(template, target).map(|(_, kind)| kind);
        assert_eq!(right(Ground(p)), Some(OrderKind::Move(Target::Point(p))));
        assert_eq!(right(EnemyObject(id(6))), Some(OrderKind::Attack(id(6))));
        assert_eq!(
            right(FriendlyObject(id(2))),
            Some(OrderKind::Move(Target::Object(id(2))))
        );
        assert_eq!(
            right(NeutralObject(id(8))),
            Some(OrderKind::Move(Target::Object(id(8))))
        );

        let left = target_order;
        assert_eq!(
            left(UnitCommand::Attack, EnemyObject(id(6))),
            Some(OrderKind::Attack(id(6)))
        );
        assert_eq!(left(UnitCommand::Attack, Ground(p)), Some(OrderKind::AttackMove(p)));
        assert_eq!(left(UnitCommand::Attack, FriendlyObject(id(2))), None);
        assert_eq!(left(UnitCommand::Attack, NeutralObject(id(8))), None);
        assert_eq!(
            left(UnitCommand::Move, Ground(p)),
            Some(OrderKind::Move(Target::Point(p)))
        );
        assert_eq!(
            left(UnitCommand::Move, EnemyObject(id(6))),
            Some(OrderKind::Move(Target::Object(id(6))))
        );
        assert_eq!(
            left(UnitCommand::Move, NeutralObject(id(8))),
            Some(OrderKind::Move(Target::Object(id(8))))
        );
        assert_eq!(left(UnitCommand::Patrol, Ground(p)), Some(OrderKind::Patrol(p)));
        assert_eq!(left(UnitCommand::Patrol, EnemyObject(id(6))), None);
        assert_eq!(left(UnitCommand::AttackGround, Ground(p)), None, "no sim order yet");

        // With the Riot Shield up, an Attack becomes a Move to the clicked point; the other orders stay.
        assert_eq!(
            order_without_attack(OrderKind::Attack(id(6)), p),
            OrderKind::Move(Target::Point(p))
        );
        for kind in [
            OrderKind::AttackMove(p),
            OrderKind::Move(Target::Object(id(6))),
            OrderKind::Stop,
        ] {
            assert_eq!(order_without_attack(kind, Vec2::ZERO), kind);
        }
    }

    /// `> PointerDisplayType > Resolution Rules`.
    #[test]
    fn the_pointer_previews_what_a_click_would_do() {
        use CursorTarget::{EnemyObject, FriendlyObject, Ground, NeutralObject};
        use PointerDisplay as D;
        let unit = Some(InterfaceTemplate::BasicCombatUnit);
        let ground = Ground(Vec2::ZERO);
        let default = InterfaceState::DefaultState;
        let pointer_display = |template, interface, target| pointer_display(template, interface, target, true);
        assert_eq!(
            pointer_display(None, default, EnemyObject(id(6))),
            D::Inactive,
            "nothing selected"
        );
        assert_eq!(pointer_display(unit, default, EnemyObject(id(6))), D::Attack);
        assert_eq!(pointer_display(unit, default, ground), D::Move);
        assert_eq!(pointer_display(unit, default, FriendlyObject(id(2))), D::Move);
        assert_eq!(pointer_display(unit, default, NeutralObject(id(8))), D::Move);
        assert_eq!(
            super::pointer_display(unit, default, EnemyObject(id(6)), false),
            D::Move,
            "shields up: a right-click moves there"
        );
        let awaiting = InterfaceState::AwaitingTarget;
        assert_eq!(pointer_display(unit, awaiting(UnitCommand::Attack), ground), D::Attack);
        assert_eq!(
            pointer_display(unit, awaiting(UnitCommand::Attack), FriendlyObject(id(2))),
            D::Inactive
        );
        assert_eq!(
            pointer_display(unit, awaiting(UnitCommand::Move), EnemyObject(id(6))),
            D::Move
        );
        assert_eq!(pointer_display(unit, awaiting(UnitCommand::Patrol), ground), D::Patrol);
        assert_eq!(
            pointer_display(unit, awaiting(UnitCommand::Patrol), NeutralObject(id(8))),
            D::Inactive
        );
        assert_eq!(
            pointer_display(unit, awaiting(UnitCommand::AttackGround), ground),
            D::AttackGround
        );
        assert_eq!(pointer_display(unit, awaiting(UnitCommand::Reverse), ground), D::Move);
    }
}
