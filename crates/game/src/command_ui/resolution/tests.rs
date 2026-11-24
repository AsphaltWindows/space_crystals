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

/// An enemy ground unit, an enemy structure (none exists in M1: a destructible enemy object of the Ground
/// domain, a structure's), an own unit, a neutral one, a crystal patch and an enemy aircraft.
fn enemy() -> MapTarget {
    on(CursorTarget::EnemyObject(id(6)), true, Domain::Ground)
}
fn enemy_structure() -> MapTarget {
    on(CursorTarget::EnemyObject(id(5)), true, Domain::Ground)
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
/// A producing structure, a Barracks.
const PRODUCER: UnitCapabilities = UnitCapabilities {
    template: Some(InterfaceTemplate::Barracks),
    ..BUILDING
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
    assert_eq!(left(PanelCommand::Attack, enemy()), Some(OrderKind::Attack(id(6))));
    assert_eq!(left(PanelCommand::Attack, ground), Some(OrderKind::AttackMove(P)));
    // Force-attack (0059, 0064): own and neutral destructible objects too, never an indestructible one.
    assert_eq!(left(PanelCommand::Attack, own()), Some(OrderKind::Attack(id(2))));
    assert_eq!(
        left(PanelCommand::Attack, neutral_unit()),
        Some(OrderKind::Attack(id(9)))
    );
    assert_eq!(left(PanelCommand::Attack, patch()), None);
    assert_eq!(
        left(PanelCommand::Move, ground),
        Some(OrderKind::Move(Target::Point(P)))
    );
    assert_eq!(left(PanelCommand::Move, enemy()), Some(object(6)));
    assert_eq!(left(PanelCommand::Move, patch()), Some(object(8)));
    assert_eq!(left(PanelCommand::Patrol, ground), Some(OrderKind::Patrol(P)));
    assert_eq!(left(PanelCommand::Patrol, enemy()), None);
    assert_eq!(left(PanelCommand::AttackGround, ground), None, "no sim order yet");
}

/// The Riot Shield's two rules. A right-click that would attack sends a shielded unit after the target instead
/// (design change 0074): it follows a unit and moves to a structure. An explicit Attack leaves it out (0075), on
/// any object, own ones too (force-attack); every other order of a targeting click, an AttackMove included
/// (0076), goes to it as to the others.
#[test]
fn a_raised_shield_follows_on_a_right_click_and_is_left_out_of_an_explicit_attack() {
    assert_eq!(order_without_attack(&enemy()), object(6));
    assert_eq!(order_without_attack(&enemy_structure()), object(5));
    for kind in [OrderKind::Attack(id(6)), OrderKind::Attack(id(2))] {
        assert_eq!(unit_target_order(SHIELDED, kind), None, "{kind:?}");
        assert_eq!(unit_target_order(ARMED, kind), Some(kind));
    }
    let everyone = [
        OrderKind::AttackMove(P),
        OrderKind::Move(Target::Point(P)),
        object(6),
        OrderKind::Patrol(P),
        OrderKind::Hold,
        OrderKind::Stop,
    ];
    for kind in everyone {
        for unit in [ARMED, SHIELDED, UNARMED] {
            assert_eq!(unit_target_order(unit, kind), Some(kind), "{unit:?} {kind:?}");
        }
    }
}

/// Design changes 0062, 0063 and 0074: every unit resolves a right-click for itself. On an enemy, a unit that
/// can attack it attacks; one that can't (no attack, or not in the target's domain) follows it, and so does one
/// whose shield is up: it follows an enemy unit, and moves to an enemy structure. The ground, own and neutral
/// objects give everyone a Move. An object without commands does nothing.
#[test]
fn each_unit_resolves_a_right_click_for_itself() {
    let attack = OrderKind::Attack(id(6));
    let spot = OrderKind::Move(Target::Point(P));
    let resolved = |unit, target: MapTarget| unit_right_click(unit, &target);
    assert_eq!(resolved(ARMED, enemy()), Some(attack));
    assert_eq!(resolved(UNARMED, enemy()), Some(object(6)), "follows");
    assert_eq!(resolved(SHIELDED, enemy()), Some(object(6)), "follows");
    assert_eq!(resolved(ARMED, enemy_structure()), Some(OrderKind::Attack(id(5))));
    assert_eq!(resolved(SHIELDED, enemy_structure()), Some(object(5)), "moves to it");
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
        "shields up: a right-click follows"
    );
    let awaiting = InterfaceState::awaiting;
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Attack), &unit, &ground),
        D::Attack
    );
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Attack), &unit, &enemy()),
        D::Attack
    );
    // 0064: force-attack on own and neutral destructible objects; an indestructible one is no target.
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Attack), &unit, &own()),
        D::Attack
    );
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Attack), &unit, &neutral_unit()),
        D::Attack
    );
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Attack), &unit, &patch()),
        D::Inactive
    );
    assert_eq!(pointer_display(awaiting(PanelCommand::Move), &unit, &enemy()), D::Move);
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Patrol), &unit, &ground),
        D::Patrol
    );
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Patrol), &unit, &patch()),
        D::Inactive
    );
    assert_eq!(
        pointer_display(awaiting(PanelCommand::AttackGround), &unit, &ground),
        D::AttackGround
    );
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Reverse), &unit, &ground),
        D::Move
    );
    assert_eq!(
        pointer_display(awaiting(PanelCommand::Reverse), &unit, &enemy()),
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

/// `> Production Building Default Right-Click` (design change 0086) and `factions/gdo/structures.md > Barracks >
/// ObjectInterfaceState[Barracks]`: a producer's right-click, and a left-click while Set Rally Point awaits its
/// target, set its rally point on what is under the cursor: the ground or any object (the producer itself too, which
/// the sim takes as clearing it). It gives no order. The pointer shows Move (`> Resolution Rules`: "Selection is a
/// production building: Move (rally point)"; "AwaitingTarget[SetRallyPoint] over ground or object: Move"), and
/// Mixed beside units that would attack.
#[test]
fn a_producer_rallies_on_whatever_is_clicked() {
    use PointerDisplay as D;
    let ground = MapTarget::ground(P);
    let rally = |target| Some(Resolved::Rally(target));
    let awaiting = InterfaceState::awaiting(PanelCommand::SetRallyPoint);
    for (clicked, target) in [
        (ground, Target::Point(P)),
        (enemy(), Target::Object(id(6))),
        (own(), Target::Object(id(2))),
        (patch(), Target::Object(id(8))),
        (aircraft(), Target::Object(id(7))),
    ] {
        assert_eq!(object_right_click(PRODUCER, &clicked), rally(target));
        assert_eq!(target_resolution(PanelCommand::SetRallyPoint, &clicked), rally(target));
        assert_eq!(
            pointer_display(InterfaceState::DefaultState, &[PRODUCER], &clicked),
            D::Move
        );
        assert_eq!(pointer_display(awaiting, &[PRODUCER], &clicked), D::Move);
    }
    assert_eq!(unit_right_click(PRODUCER, &ground), None, "no order");
    assert_eq!(
        object_right_click(ARMED, &enemy()),
        Some(Resolved::Order(OrderKind::Attack(id(6))))
    );
    assert_eq!(object_right_click(BUILDING, &ground), None);
    assert_eq!(right_click_display(&[ARMED, PRODUCER], &enemy()), D::Mixed);
    assert_eq!(right_click_display(&[ARMED, PRODUCER], &ground), D::Move);
    assert_eq!(
        target_resolution(PanelCommand::Attack, &enemy()),
        Some(Resolved::Order(OrderKind::Attack(id(6))))
    );
    assert_eq!(target_resolution(PanelCommand::Patrol, &enemy()), None);
}
