mod gathering;

use sc_sim::defs::TargetDomain;

use super::*;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

const P: Vec2 = Vec2::new(300.0, 200.0);

/// `target` under the cursor at [`P`], as the player knows it: an object destructible or not, in `domain`, an enemy
/// if it is under the cursor as one. Neither a patch to gather nor a drop-off ([`patch`], [`own_node`]).
fn on(target: CursorTarget, destructible: bool, domain: Domain) -> MapTarget {
    let id = target.object().expect("an object");
    MapTarget {
        target,
        ground: P,
        aimed: Some(Aimed {
            id,
            enemy: matches!(target, CursorTarget::EnemyObject(_)),
            destructible,
            domain,
            gatherable: false,
            station: false,
            drop_off: false,
            transit: &[],
        }),
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
/// A Space Crystals patch, free to gather.
fn patch() -> MapTarget {
    let mut patch = on(CursorTarget::NeutralObject(id(8)), false, Domain::Ground);
    patch.aimed = patch.aimed.map(|aimed| Aimed {
        gatherable: true,
        ..aimed
    });
    patch
}
fn aircraft() -> MapTarget {
    on(CursorTarget::EnemyObject(id(7)), true, Domain::Air)
}
/// The player's own Egress Node, which takes loads.
fn own_node() -> MapTarget {
    let mut node = on(CursorTarget::FriendlyObject(id(3)), true, Domain::Ground);
    node.aimed = node.aimed.map(|aimed| Aimed {
        drop_off: true,
        ..aimed
    });
    node
}

/// A unit of the BasicCombatUnit template with an attack in `attack`, whose mode lets it attack or not.
const fn combat_unit(attack: Option<TargetDomain>, may_attack: bool) -> UnitCapabilities {
    UnitCapabilities {
        template: Some(InterfaceTemplate::BasicCombatUnit),
        abilities: Some(Abilities {
            attack,
            may_attack,
            worker: false,
            carrying: false,
            gathers_supplies: false,
            transit: Transit::Infantry,
        }),
    }
}

/// A combat unit with a ground attack, the Peacekeeper's and the Guard's.
const ARMED: UnitCapabilities = combat_unit(Some(TargetDomain::Ground), true);
/// The same with its Riot Shield up.
const SHIELDED: UnitCapabilities = combat_unit(Some(TargetDomain::Ground), false);
/// One without an attack, like the design's Supply Chopper (none exists yet).
const UNARMED: UnitCapabilities = combat_unit(None, true);
/// One that attacks anything.
const UNIVERSAL: UnitCapabilities = combat_unit(Some(TargetDomain::Universal), true);
/// A Drone: a worker without an attack, of the Drone's template (slice M3-J1); and one with a load.
const DRONE: UnitCapabilities = UnitCapabilities {
    template: Some(InterfaceTemplate::Drone),
    abilities: Some(Abilities {
        attack: None,
        may_attack: true,
        worker: true,
        carrying: false,
        gathers_supplies: false,
        transit: Transit::Vehicles,
    }),
};
const LOADED_DRONE: UnitCapabilities = UnitCapabilities {
    template: Some(InterfaceTemplate::Drone),
    abilities: Some(Abilities {
        attack: None,
        may_attack: true,
        worker: true,
        carrying: true,
        gathers_supplies: false,
        transit: Transit::Vehicles,
    }),
};
/// An object without commands: a building.
const BUILDING: UnitCapabilities = UnitCapabilities {
    template: None,
    abilities: None,
};
/// A producing structure, a Barracks.
const PRODUCER: UnitCapabilities = UnitCapabilities {
    template: Some(InterfaceTemplate::Production),
    abilities: None,
};

fn object(n: u32) -> OrderKind {
    OrderKind::Move(Target::Object(id(n)))
}

/// `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState > RightClickResolution` and
/// `systems/controls/selection_and_hud.md > AwaitingTarget[..] resolution`.
#[test]
fn clicks_resolve_by_what_is_under_the_cursor() {
    let right = |target: MapTarget| unit_right_click(ARMED, &target);
    assert_eq!(right(MapTarget::ground(P)), Some(OrderKind::Move(Target::Point(P))));
    assert_eq!(right(enemy()), Some(OrderKind::Attack(id(6))));
    assert_eq!(right(own()), Some(object(2)));
    assert_eq!(right(patch()), Some(object(8)));

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
    // Slice M4-O: the ground only, as `systems/controls/selection_and_hud.md > AwaitingTarget[AttackGround]
    // resolution` has it.
    assert_eq!(
        left(PanelCommand::AttackGround, ground),
        Some(OrderKind::AttackGround(P))
    );
    assert_eq!(left(PanelCommand::AttackGround, enemy()), None);
    assert_eq!(left(PanelCommand::AttackGround, own()), None);
}

/// The Riot Shield's two rules. A right-click that would attack sends a shielded unit after the target instead
/// (design change 0074): it follows a unit and moves to a structure. An explicit Attack leaves it out (0075), on
/// any object, own ones too (force-attack); every other order of a targeting click, an AttackMove included
/// (0076), goes to it as to the others.
#[test]
fn a_raised_shield_follows_on_a_right_click_and_is_left_out_of_an_explicit_attack() {
    assert_eq!(unit_right_click(SHIELDED, &enemy()), Some(object(6)));
    assert_eq!(unit_right_click(SHIELDED, &enemy_structure()), Some(object(5)));
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

/// `factions/colonists/units/drone.md > Right-Click Resolution`, by the sim's one rule (design change 0136): a
/// patch gives Gather, the player's own node Drop Off while the Drone carries a load and a Move there without one
/// (design change 0140), an enemy a Move to it (unarmed: it follows a unit), the ground a Move. A target the player
/// remembers resolves as one it sees, by what it knows (design change 0139); one it knows nothing of (only the fog's
/// debug toggle shows it) gives a Move to it, which the sim refuses.
#[test]
fn a_drone_resolves_a_right_click_by_its_own_table() {
    let resolved = |unit, target: MapTarget| unit_right_click(unit, &target);
    for drone in [DRONE, LOADED_DRONE] {
        assert_eq!(resolved(drone, patch()), Some(OrderKind::Gather(id(8))));
        assert_eq!(resolved(drone, enemy()), Some(object(6)), "follows");
        assert_eq!(resolved(drone, enemy_structure()), Some(object(5)));
        assert_eq!(
            resolved(drone, MapTarget::ground(P)),
            Some(OrderKind::Move(Target::Point(P)))
        );
        assert_eq!(
            object_right_click(drone, &patch()),
            Some(Resolved::Order(OrderKind::Gather(id(8))))
        );
    }
    assert_eq!(resolved(LOADED_DRONE, own_node()), Some(OrderKind::DropOff(id(3))));
    assert_eq!(resolved(DRONE, own_node()), Some(object(3)));
    assert_eq!(resolved(ARMED, own_node()), Some(object(3)));
    let unknown = MapTarget { aimed: None, ..enemy() };
    assert_eq!(resolved(ARMED, unknown), Some(object(6)));
    assert_eq!(resolved(DRONE, unknown), Some(object(6)));
    assert_eq!(right_click_display(&[DRONE], &patch()), PointerDisplay::GatherResources);
}

/// Slice M3-J1's objects without a right-click: the Beacon, and an object that transforms (a Beacon deploying, a
/// portal under Warp In: no rally point, design change 0116), which gets neither a template nor abilities.
#[test]
fn the_beacon_and_a_transforming_object_resolve_nothing() {
    let modes = Modes::default().with_transforming([id(2)]);
    let beacon = UnitCapabilities::of(id(3), ObjectKind::Beacon, &modes);
    assert_eq!(beacon.template, Some(InterfaceTemplate::Beacon));
    assert_eq!(object_right_click(beacon, &enemy()), None);
    for kind in [ObjectKind::RoboPortal, ObjectKind::Beacon] {
        let transforming = UnitCapabilities::of(id(2), kind, &modes);
        assert_eq!(
            (transforming.template, transforming.abilities),
            (None, None),
            "{kind:?}"
        );
        assert_eq!(object_right_click(transforming, &patch()), None, "{kind:?}");
        assert_ne!(UnitCapabilities::of(id(3), kind, &modes).template, None, "{kind:?}");
    }
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

/// `systems/controls/selection_and_hud.md > PointerDisplayType > Resolution Rules`.
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

/// `systems/controls/selection_and_hud.md > Mixed Right-Click Preview` (design change 0063): the pointer shows what the
/// units would do, or Mixed when they would do different things. Moving to a spot and following are both Move.
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

/// `systems/controls/selection_and_hud.md > Production Building Default Right-Click` (design change 0086) and
/// `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`: a producer's right-click, and a left-click
/// while Set Rally Point awaits its target, set its rally point on what is under the cursor: the ground or any object
/// (the producer itself too, which the sim takes as clearing it). It gives no order. The pointer shows Move
/// (`systems/controls/selection_and_hud.md > Resolution Rules`: "Selection is a production building: Move (rally
/// point)"; "AwaitingTarget[SetRallyPoint] over ground or object: Move"), and Mixed beside units that would attack.
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
