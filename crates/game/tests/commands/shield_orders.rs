//! What a raised Riot Shield does to the orders a Peacekeeper takes (`factions/gdo/units.md > Peacekeeper > Riot
//! Shield`): a right-click on an enemy has it follow a unit or move to a structure instead of attacking (design
//! change 0074) while the others of the selection attack; an explicit Attack on an object leaves it out, on its
//! orders (0075); an AttackMove takes it along, walking on without firing (0076). A Peacekeeper with its shield up is
//! a Peacekeeper (Shielded) (design change 0173), whose own panel shows no Attack. Guards stand in for the design's
//! Rocketmen and tanks, which M1 doesn't have.

use bevy::prelude::*;
use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS;
use sc_sim::{Ability, Command, ObjectId, OrderKind, PlayerId, Scenario, ShieldMode, SimEvent, Spawn, Target};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelCommand, PointerDisplay};
use space_crystals::hud::command_panel::CommandButton;

use super::*;
use KeyCode::{KeyA, KeyR};

const RAISE: Ability = Ability::RiotShield { raise: true };

// The units of a [`field`], by id.
/// Player 1's Peacekeepers and Guards.
const PKS: [u32; 4] = [1, 2, 3, 4];
const GUARDS: [u32; 4] = [5, 6, 7, 8];
const OWN: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
/// Player 2's first unit.
const ENEMY: u32 = 9;

/// A flat 64 × 48 map: player 1's Peacekeepers (ids 1 to 4) at `pks`, its Guards (5 to 8) at `guards`, and player
/// 2's `enemies` (ids from 9), each of its kind and at its point.
fn field(pks: [Vec2; 4], guards: [Vec2; 4], enemies: &[(ObjectKind, Vec2)]) -> Scenario {
    let mut scenario = scenarios::empty(64, 48);
    let unit = |kind, slot, at| Spawn::new(kind, Some(SlotId(slot)), SpawnAt::Point(at));
    scenario
        .map
        .spawns
        .extend(pks.map(|at| unit(ObjectKind::Peacekeeper, 1, at)));
    scenario
        .map
        .spawns
        .extend(guards.map(|at| unit(ObjectKind::Guard, 1, at)));
    scenario
        .map
        .spawns
        .extend(enemies.iter().map(|&(kind, at)| unit(kind, 2, at)));
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// Four points in a square with sides of `side`, its north-west corner at `corner`.
fn square(corner: Vec2, side: f32) -> [Vec2; 4] {
    [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE].map(|at| corner + at * side)
}

fn attack(id: u32) -> OrderKind {
    OrderKind::Attack(ObjectId(id))
}

fn follow(id: u32) -> OrderKind {
    OrderKind::Move(Target::Object(ObjectId(id)))
}

/// How far up the Riot Shield of Peacekeeper `id` is, as the sim has it.
fn shield(app: &App, id: u32) -> ShieldMode {
    let sim = app.world().resource::<SimRes>();
    sc_sim::riot_shield::mode(sim.object(ObjectId(id)).unwrap()).expect("a Peacekeeper")
}

/// Selects player 1's Peacekeepers `ids` and raises their shields (R); they are up when it returns.
fn raise_shields(app: &mut App, ids: &[u32]) {
    select_units(app, ids);
    tap(app, &[], KeyR);
    updates(app, RIOT_SHIELD_TOGGLE_TICKS as usize + 2);
    assert!(ids.iter().all(|&id| shield(app, id) == ShieldMode::Raised));
}

/// The units among `ids` that fired in `events`, in id order.
fn fired(events: &[SimEvent], ids: &[u32]) -> Vec<u32> {
    let mut shooters: Vec<u32> = events
        .iter()
        .filter_map(|event| match event {
            SimEvent::Attacked { attacker, .. } if ids.contains(&attacker.0) => Some(attacker.0),
            _ => None,
        })
        .collect();
    shooters.sort_unstable();
    shooters.dedup();
    shooters
}

/// The look of the Attack button (A).
fn attack_button(app: &mut App) -> Option<ButtonLook> {
    let mut query = app.world_mut().query::<&CommandButton>();
    query
        .iter(app.world())
        .find(|b| b.slot == slot(1, 0))
        .map(|b| b.shown.look)
}

/// 0074 Verification 1: four shielded Peacekeepers and four Guards; an enemy Peacekeeper, its own shield up (slow
/// and hard to kill, like the design's tank), walks away. A right-click on it (the Mixed pointer: they would do
/// different things) has the Guards attack it and the Peacekeepers follow it; they catch up once it stops and stay
/// by it, while the Guards, from farther away, come up and fire at it.
#[test]
fn shielded_peacekeepers_follow_an_enemy_that_the_guards_attack() {
    let enemy = [(ObjectKind::Peacekeeper, Vec2::new(1650.0, 840.0))];
    let mut app = game_of(field(
        square(Vec2::new(1500.0, 820.0), 40.0),
        square(Vec2::new(760.0, 820.0), 40.0),
        &enemy,
    ));
    submit(
        &mut app,
        PlayerId(2),
        Command::Ability {
            units: vec![ObjectId(ENEMY)],
            ability: RAISE,
        },
    );
    raise_shields(&mut app, &PKS);
    assert_eq!(shield(&app, ENEMY), ShieldMode::Raised);
    let away = Command::Order {
        units: vec![ObjectId(ENEMY)],
        kind: OrderKind::Move(Target::Point(Vec2::new(1750.0, 840.0))),
        queue: false,
    };
    submit(&mut app, PlayerId(2), away);
    updates(&mut app, 2);
    select_units(&mut app, &OWN);

    let walking = pos(&app, ENEMY);
    assert!(walking.x > 1650.0, "walking away");
    assert_eq!(pointer_over(&mut app, walking), Some(PointerDisplay::Mixed));
    let walking = pos(&app, ENEMY);
    right_click(&mut app, walking, &[]);
    assert_eq!(
        orders(&app)[1..],
        [given(&PKS, follow(ENEMY), false), given(&GUARDS, attack(ENEMY), false)],
        "after player 2's walk"
    );

    let events = run_collecting(&mut app, 120);
    let enemy_at = pos(&app, ENEMY);
    assert!(enemy_at.x > 1740.0, "it stopped where it walked to: {enemy_at}");
    for id in PKS {
        let distance = pos(&app, id).distance(enemy_at);
        assert!(distance < 160.0, "Peacekeeper {id} stays by it: {distance}");
        assert_eq!(
            object_orders(&app, id).current.expect("following").order.kind,
            follow(ENEMY)
        );
    }
    assert!(fired(&events, &GUARDS).is_empty(), "the Guards are still on their way");

    let events = run_collecting(&mut app, 200);
    assert!(!fired(&events, &GUARDS).is_empty(), "the Guards attack it");
    assert!(fired(&events, &PKS).is_empty());
}

/// 0074 Verification 2, against the neutral Supply Delivery Station: M1 has no enemy structure (none can be
/// destroyed or owned yet), so the rule's enemy structure is unit-tested (`resolution.rs`). Shielded Peacekeepers
/// right-clicked onto it move to it and stop nearby, without attacking.
#[test]
fn shielded_peacekeepers_move_to_a_structure_and_stop_by_it() {
    let mut app = game();
    let units = [
        spawn(
            &mut app,
            ObjectKind::Peacekeeper,
            Some(PlayerId(1)),
            Vec2::new(1500.0, 900.0),
        ),
        spawn(
            &mut app,
            ObjectKind::Peacekeeper,
            Some(PlayerId(1)),
            Vec2::new(1560.0, 900.0),
        ),
    ];
    raise_shields(&mut app, &units);
    assert_eq!(pointer_over(&mut app, at(STATION)), Some(PointerDisplay::Move));
    right_click(&mut app, at(STATION), &[]);
    assert_eq!(orders(&app), [given(&units, follow(STATION), false)]);

    let events = run_collecting(&mut app, 300);
    for id in units {
        assert_eq!(object_orders(&app, id).current, None, "{id} arrived");
        // Within 3 grid units of the 2 × 2 station, edge to edge (`behaviors.md > MovingToObject`, design change
        // 0169; 2 before it).
        let distance = pos(&app, id).distance(at(STATION));
        assert!(distance < 64.0 + 192.0 + 24.0, "{id} by the station: {distance}");
    }
    assert!(fired(&events, &units).is_empty());
}

/// 0074 Verification 3: with their shields lowered again, the Peacekeepers attack an enemy they are right-clicked
/// onto, as before: the Attack pointer, the Attack order, and their shots.
#[test]
fn lowered_shields_attack_on_a_right_click_again() {
    let mut app = game();
    raise_shields(&mut app, &[1, 2]);
    tap(&mut app, &[], KeyR);
    updates(&mut app, RIOT_SHIELD_TOGGLE_TICKS as usize + 2);
    assert_eq!([shield(&app, 1), shield(&app, 2)], [ShieldMode::Lowered; 2]);
    assert_eq!(pointer_over(&mut app, at(ENEMY_PK)), Some(PointerDisplay::Attack));
    right_click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(orders(&app), [given(&[1, 2], attack(ENEMY_PK), false)]);
    let events = run_collecting(&mut app, 400);
    assert!(!fired(&events, &[1, 2]).is_empty(), "they attack");
}

/// 0075 Verification 1: four shielded Peacekeepers and four Guards walk somewhere; Attack (A) and a click on an
/// enemy send the Guards to attack it. The Peacekeepers are left out: they carry on with their move, the same
/// order, and get where it sends them.
#[test]
fn an_attack_leaves_shielded_peacekeepers_on_their_way() {
    let enemy = [(ObjectKind::Peacekeeper, Vec2::new(1250.0, 680.0))];
    let mut app = game_of(field(
        square(Vec2::new(1000.0, 760.0), 50.0),
        square(Vec2::new(1000.0, 900.0), 50.0),
        &enemy,
    ));
    raise_shields(&mut app, &PKS);
    select_units(&mut app, &OWN);
    let somewhere = Vec2::new(1300.0, 1150.0);
    right_click(&mut app, somewhere, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 1);
    assert_eq!(given_orders[0].units, OWN);
    assert!(near(point_of(given_orders[0].kind), somewhere));
    let moves = PKS.map(|id| object_orders(&app, id).current.expect("walking").order);

    tap(&mut app, &[], KeyA);
    let enemy_at = pos(&app, ENEMY);
    click(&mut app, enemy_at, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app)[1..], [given(&GUARDS, attack(ENEMY), false)]);
    for (id, walk) in PKS.iter().zip(&moves) {
        assert_eq!(&object_orders(&app, *id).current.expect("walking").order, walk, "{id}");
    }

    let events = run_collecting(&mut app, 300);
    for (id, walk) in PKS.iter().zip(&moves) {
        assert_eq!(object_orders(&app, *id).current, None, "{id} arrived");
        let distance = pos(&app, *id).distance(walk.destination().unwrap());
        assert!(distance < 30.0, "{id} where its move sent it: {distance}");
    }
    assert!(!fired(&events, &GUARDS).is_empty(), "the Guards attack");
}

/// 0075 Verifications 2 and 3: of four Peacekeepers, two shielded, Attack (A) and a click on an enemy send only
/// the other two to attack it. With only shielded Peacekeepers selected, Attack is unavailable: the Peacekeeper
/// (Shielded)'s panel shows none (design change 0173), and its hotkey does nothing.
#[test]
fn an_attack_goes_to_the_unshielded_peacekeepers_alone() {
    let mut app = game();
    let fourth = spawn(
        &mut app,
        ObjectKind::Peacekeeper,
        Some(PlayerId(1)),
        Vec2::new(1050.0, 1100.0),
    );
    raise_shields(&mut app, &[1, 2]);
    select_units(&mut app, &[1, 2, 5, fourth]);
    assert_eq!(attack_button(&mut app), Some(ButtonLook::Common));
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Attack));
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(orders(&app), [given(&[5, fourth], attack(ENEMY_PK), false)]);

    select_box(&mut app, &[1, 2]);
    assert_eq!(attack_button(&mut app), None);
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app).len(), 1);
}

/// 0076 Verification 1: four shielded Peacekeepers and four Guards, auto-targeting on; Attack (A) and a click on the
/// ground beyond two enemy Guards give the AttackMove to all eight. The Guards stop and fight; the Peacekeepers,
/// which can't attack, walk on to where it sends them and never fire.
#[test]
fn an_attack_move_takes_shielded_peacekeepers_past_the_enemies() {
    let enemies = [
        (ObjectKind::Guard, Vec2::new(1450.0, 660.0)),
        (ObjectKind::Guard, Vec2::new(1510.0, 660.0)),
    ];
    let mut app = game_of(field(
        square(Vec2::new(1000.0, 760.0), 50.0),
        square(Vec2::new(1000.0, 900.0), 50.0),
        &enemies,
    ));
    raise_shields(&mut app, &PKS);
    submit(
        &mut app,
        PlayerId(1),
        Command::Debug(DebugCommand::SetAutoTargeting { on: true }),
    );
    select_units(&mut app, &OWN);
    tap(&mut app, &[], KeyA);
    let beyond = Vec2::new(1800.0, 860.0);
    click(&mut app, beyond, &[]);
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 1, "{given_orders:?}");
    assert_eq!(given_orders[0].units, OWN);
    assert!(matches!(given_orders[0].kind, OrderKind::AttackMove(p) if near(p, beyond)));
    let destinations = PKS.map(|id| {
        let order = object_orders(&app, id).current.expect("walking").order;
        order.destination().unwrap()
    });

    let events = run_collecting(&mut app, 450);
    assert!(!fired(&events, &GUARDS).is_empty(), "the Guards fight");
    assert!(fired(&events, &PKS).is_empty(), "the Peacekeepers never fire");
    for (id, destination) in PKS.iter().zip(destinations) {
        assert_eq!(object_orders(&app, *id).current, None, "{id} arrived");
        let distance = pos(&app, *id).distance(destination);
        assert!(distance < 30.0, "{id} at its destination: {distance}");
    }
}
