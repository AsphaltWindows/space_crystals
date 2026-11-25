//! A press of R sets the Peacekeepers' target state (design change 0198, `factions/gdo/units/peacekeeper.md > Riot
//! Shield > Two unit types`): their type switches at the press, and a second press reaches them all, so however fast
//! R is pressed they end in the same state. Which orders a unit takes follows its type from the press, and the sim
//! carries them out once the shield is done (design change 0202, `> During the toggle > Orders given during the
//! toggle`): a Peacekeeper whose shield is on its way down takes an Attack, by the panel or a right-click, and attacks
//! once its shield is down; one whose shield is on its way up takes none, and a right-click has it follow.

use sc_sim::combat::AttackPhase;
use sc_sim::defs::RIOT_SHIELD_TOGGLE_TICKS as T;
use sc_sim::{Command, Tick};
use space_crystals::sim_driver::SimControl;

use super::*;

/// How far up the Riot Shield of unit `id` is (`RiotShield::up`), as the sim has it.
fn up(app: &App, id: u32) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id)).unwrap().riot_shield.unwrap().up
}

/// Player 1's units `units` attack object `target`, given in the next tick.
fn attack(app: &mut App, units: &[u32], target: u32) {
    let units = units.iter().map(|&n| ObjectId(n)).collect();
    let kind = OrderKind::Attack(ObjectId(target));
    submit(
        app,
        PlayerId(1),
        Command::Order {
            units,
            kind,
            queue: false,
        },
    );
}

/// The ticks in which each of `units` lands an attack (`SimEvent::Attacked`) over the next `n` ticks, and the tick
/// each one's shield is first all the way down.
fn attacks_and_downs(app: &mut App, units: &[u32], n: usize) -> (Vec<Vec<Tick>>, Vec<Option<Tick>>) {
    let mut attacks = vec![Vec::new(); units.len()];
    let mut down = vec![None; units.len()];
    for _ in 0..n {
        app.update();
        let sim = app.world().resource::<SimRes>();
        for event in sim.events() {
            if let SimEvent::Attacked { attacker, .. } = event
                && let Some(i) = units.iter().position(|&n| ObjectId(n) == *attacker)
            {
                attacks[i].push(sim.tick());
            }
        }
        for (i, &unit) in units.iter().enumerate() {
            if down[i].is_none() && up(app, unit) == 0 {
                down[i] = Some(app.world().resource::<SimRes>().tick());
            }
        }
    }
    (attacks, down)
}

/// Player 1's Peacekeepers 1 and 2 of [`game`], their shields raised, and player 2's Peacekeeper (Shielded) between
/// them and Peacekeeper 5, in their range (4 grid units), its shield up: it never fires back. The two are selected.
fn shielded_pair() -> (App, u32) {
    let mut app = game();
    let enemy = spawn(&mut app, SHIELDED, Some(PlayerId(2)), Vec2::new(1000.0, 1000.0));
    select_box(&mut app, &[1, 2]);
    tap(&mut app, &[], KeyR);
    toggle_time(&mut app);
    assert_eq!([state(&app, 1), state(&app, 2)], [(SHIELDED, ShieldMode::Raised); 2]);
    (app, enemy)
}

/// 0198's Verification: "Peacekeepers mid-shot: press R. The panel shows Lower Shield at once. Press R again before
/// the shot ends: none raise." Peacekeepers 1 and 2 attack player 2's Peacekeeper (Shielded) together; R pressed as
/// their shots land, in their Cooldown, makes them Peacekeepers (Shielded) at the press, their panel Lower Shield's,
/// their raise waiting for the end of their shot; R again the next frame lowers them all (`> Two unit types`: "Lower
/// pressed while a raise is still deferred [...] the shield never starts rising"). Their shields never leave the
/// ground, and they fight on, Peacekeepers, with their Attack (design change 0150).
#[test]
fn peacekeepers_mid_shot_r_shows_lower_shield_at_once_and_r_again_before_the_shot_ends_none_raise() {
    let mut app = game();
    let enemy = spawn(&mut app, SHIELDED, Some(PlayerId(2)), Vec2::new(1000.0, 1000.0));
    attack(&mut app, &[1, 2], enemy);
    select_box(&mut app, &[1, 2]);
    let phase = |app: &App, n: u32| {
        let sim = app.world().resource::<SimRes>();
        sim.object(ObjectId(n)).unwrap().attack.as_ref().and_then(|a| a.phase)
    };
    let firing = |app: &App| [1, 2].map(|n| matches!(phase(app, n), Some(AttackPhase::Firing { .. })));
    for _ in 0..40 {
        if firing(&app) == [true; 2] {
            break;
        }
        app.update();
    }
    assert_eq!(firing(&app), [true; 2], "both mid-shot");

    let landed = |events: &[SimEvent], n: u32| {
        (events.iter()).any(|e| matches!(e, SimEvent::Attacked { attacker, .. } if *attacker == ObjectId(n)))
    };
    input(&mut app).tap_key(KeyR);
    let shots = run_collecting(&mut app, 1);
    assert!(landed(&shots, 1) && landed(&shots, 2), "their shots land");
    let cooling = [1, 2].map(|n| phase(&app, n));
    assert_eq!(cooling, [Some(AttackPhase::Cooldown); 2], "mid-shot");
    assert_eq!(groups(&app), [(SHIELDED, vec![1, 2], true)]);
    assert_eq!(
        command(&mut app, r()),
        Some((PanelCommand::LowerShield, ButtonLook::Common))
    );
    input(&mut app).tap_key(KeyR);
    app.update();
    assert_eq!([state(&app, 1), state(&app, 2)], [(SHIELDED, ShieldMode::Deferred); 2]);
    assert_eq!(abilities(&app), [used(&[1, 2], RAISE), used(&[1, 2], LOWER)]);
    assert_eq!(groups(&app), [(PEACEKEEPER, vec![1, 2], true)]);
    for _ in 0..2 * T {
        app.update();
        assert_eq!([up(&app, 1), up(&app, 2)], [0, 0], "none raise");
        assert_eq!(
            [state(&app, 1), state(&app, 2)],
            [(PEACEKEEPER, ShieldMode::Lowered); 2]
        );
    }
    let later = run_collecting(&mut app, 20);
    assert!(landed(&later, 1) && landed(&later, 2), "they fight on");
    for n in [1, 2] {
        let current = object_orders(&app, n).current.map(|active| active.order.kind);
        assert_eq!(current, Some(OrderKind::Attack(ObjectId(enemy))), "#{n}");
    }
}

/// Design change 0202's Verification: "Press R on shielded Peacekeepers, then right-click an enemy at once. They
/// finish lowering (6 frames) and then attack, with no second click needed." At the press they are Peacekeepers, whose
/// panel shows Attack, and the pointer over the enemy previews an attack: the right-click a frame on gives both an
/// Attack ("an explicit Attack or a right-click on an enemy is accepted and held"). They attack once their shields are
/// down. So they do when the right-click comes before the sim's next tick too (the sim held meanwhile): the gate,
/// which takes the Lower first, takes the Attack after it.
#[test]
fn r_on_shielded_peacekeepers_then_a_right_click_on_an_enemy_they_attack_once_lowered() {
    for held in [false, true] {
        let (mut app, enemy) = shielded_pair();
        app.world_mut().resource_mut::<SimControl>().paused = held;
        input(&mut app).tap_key(KeyR);
        app.update();
        assert_eq!(groups(&app), [(PEACEKEEPER, vec![1, 2], true)], "held {held}");
        assert_eq!(command(&mut app, a()), Some((PanelCommand::Attack, ButtonLook::Common)));
        let at = pos(&app, enemy);
        assert_eq!(pointer_over(&mut app, at), Some(PointerDisplay::Attack));
        let screen = screen_at(&app, pos(&app, enemy));
        input(&mut app).cursor = Some(screen);
        input(&mut app).mouse.press(MouseButton::Right);
        app.update();
        input(&mut app).mouse.release(MouseButton::Right);
        app.update();
        let attack = OrderKind::Attack(ObjectId(enemy));
        assert_eq!(orders(&app), [given(&[1, 2], attack, false)], "held {held}");
        if held {
            assert_eq!(
                state(&app, 1),
                (SHIELDED, ShieldMode::Raised),
                "the sim's tick is still to come"
            );
            app.world_mut().resource_mut::<SimControl>().paused = false;
        }
        let (attacks, down) = attacks_and_downs(&mut app, &[1, 2], 3 * T as usize);
        for (n, (attacks, down)) in [1, 2].into_iter().zip(attacks.iter().zip(&down)) {
            let down = down.expect("lowered");
            assert!(!attacks.is_empty(), "held {held}: #{n} attacks with no second click");
            assert!(
                attacks[0] > down,
                "held {held}: #{n} once its shield is down: {attacks:?}, down {down}"
            );
            let current = object_orders(&app, n).current.map(|active| active.order.kind);
            assert_eq!(current, Some(attack), "held {held}: #{n}");
        }
    }
}

/// Design change 0202: "The panel shows Attack, and pressing it must not silently skip the unit." Lowered, the two are
/// Peacekeepers whose shields are on their way down: their A awaits its target, and the click on the enemy gives both
/// the Attack, which they carry out once their shields are down.
#[test]
fn the_attack_button_while_lowering_goes_to_every_peacekeeper() {
    let (mut app, enemy) = shielded_pair();
    tap(&mut app, &[], KeyR);
    assert!(
        matches!(shield(&app, 1), ShieldMode::Lowering { .. }),
        "{:?}",
        shield(&app, 1)
    );
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::Attack));
    let at = pos(&app, enemy);
    click(&mut app, at, &[]);
    let attack = OrderKind::Attack(ObjectId(enemy));
    assert_eq!(orders(&app), [given(&[1, 2], attack, false)]);
    let (attacks, down) = attacks_and_downs(&mut app, &[1, 2], 2 * T as usize);
    for (n, (attacks, down)) in [1, 2].into_iter().zip(attacks.iter().zip(&down)) {
        assert!(!attacks.is_empty(), "#{n} attacks");
        assert!(
            down.is_some_and(|down| attacks[0] > down),
            "#{n}: {attacks:?}, down {down:?}"
        );
    }
}

/// Design change 0202: "While raising (already Shielded): no Attack; a right-click follows, as with the shield up."
/// Raised, the two are Peacekeepers (Shielded) whose shields are on their way up: their panel has no Attack and A does
/// nothing, the pointer over the enemy previews a Move, and the right-click has them follow it. They never fire.
#[test]
fn while_raising_there_is_no_attack_and_a_right_click_follows() {
    let mut app = game();
    let enemy = spawn(&mut app, SHIELDED, Some(PlayerId(2)), Vec2::new(1000.0, 1000.0));
    select_box(&mut app, &[1, 2]);
    tap(&mut app, &[], KeyR);
    assert!(
        matches!(shield(&app, 1), ShieldMode::Raising { .. }),
        "{:?}",
        shield(&app, 1)
    );
    assert_eq!(button(&mut app, a()), None, "no Attack");
    tap(&mut app, &[], KeyA);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    let at = pos(&app, enemy);
    assert_eq!(pointer_over(&mut app, at), Some(PointerDisplay::Move));
    let screen = screen_at(&app, pos(&app, enemy));
    input(&mut app).cursor = Some(screen);
    input(&mut app).mouse.press(MouseButton::Right);
    app.update();
    input(&mut app).mouse.release(MouseButton::Right);
    app.update();
    let follow = OrderKind::Move(Target::Object(ObjectId(enemy)));
    assert_eq!(orders(&app), [given(&[1, 2], follow, false)]);
    let (attacks, _) = attacks_and_downs(&mut app, &[1, 2], 2 * T as usize);
    assert_eq!(attacks, [Vec::<Tick>::new(), Vec::new()], "they never fire");
    assert_eq!([shield(&app, 1), shield(&app, 2)], [ShieldMode::Raised; 2]);
}
