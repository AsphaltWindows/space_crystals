//! (A1) A computer player's player takes no commands from the client (`core/game_rules.md > Computer players
//! (playtest tool)`, `space_crystals::ai_players`): while the local player is a computer's, the command flow's input is
//! dropped, mouse, keys and the panel alike, though the selection still works, and the top bar says "(AI)" after the
//! player's name. The debug player switch (F2) still changes the local player, and a human player's commands go as ever.

use sc_ai::Profile;
use space_crystals::ai_players::{AiPlayers, AiSlot};

use super::*;
use crate::hud::notes;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// [`game`]'s game with a computer playing player 1, passive: it has no buildings to defend and attacks nothing, so its
/// only orders send its idle units to its staging point ([`client_orders`]).
fn computer_game() -> App {
    let mut config = AppConfig::new(scenario());
    config.display = DisplayMode::Headless;
    config.qa.no_fog = true;
    config.ai = vec![AiSlot {
        player: P1,
        profile: Profile::Passive,
    }];
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAutoTargeting { on: false }),
    );
    app.update();
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    updates(&mut app, 2);
    app
}

/// The orders given but player 1's computer's own: its Attack Moves to its staging point (`sc_ai::Army`).
fn client_orders(app: &App) -> Vec<Given> {
    let staging = (app.world().resource::<AiPlayers>().get(P1))
        .and_then(|ai| ai.army().staging())
        .expect("player 1's computer, with a staging point");
    let computers = |o: &Given| o.player == P1 && o.kind == OrderKind::AttackMove(staging);
    orders(app).into_iter().filter(|o| !computers(o)).collect()
}

#[test]
fn a_computer_players_player_takes_no_commands_from_the_client() {
    let mut app = computer_game();
    assert_eq!(local_player(&app), P1);
    assert_eq!(notes(&mut app), ["(AI)", "Fog off (F3)"]);
    select_mixed(&mut app);
    let away = Vec2::new(1000.0, 1000.0);
    right_click(&mut app, away, &[]);
    tap(&mut app, &[], KeyCode::KeyE);
    click_button(&mut app, slot(2, 1), &[]);
    updates(&mut app, 4);
    assert_eq!(
        client_orders(&app),
        [],
        "a right-click, a hotkey and a button: nothing for the computer's player"
    );
    assert_eq!(selected(&app), [1, 2, 3, 4], "the selection still works");

    tap(&mut app, &[], KeyCode::F2);
    assert_eq!(local_player(&app), P2);
    assert_eq!(notes(&mut app), ["Fog off (F3)"], "player 2 is a human's");
    // Player 2 has a camera of their own, at their start (slice M3-J3): back to this test's view.
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    select_box(&mut app, &[ENEMY_PK]);
    right_click(&mut app, away, &[]);
    let given = client_orders(&app);
    assert_eq!(given.len(), 1, "{given:?}");
    let order = &given[0];
    assert_eq!((order.player, order.units.as_slice()), (P2, [ENEMY_PK].as_slice()));
    assert!(
        matches!(order.kind, OrderKind::Move(_)) && near(point_of(order.kind), away),
        "{order:?}"
    );

    tap(&mut app, &[], KeyCode::F2);
    assert_eq!(local_player(&app), P1);
    assert_eq!(notes(&mut app), ["(AI)", "Fog off (F3)"]);
    tap(&mut app, &[], KeyCode::KeyX);
    updates(&mut app, 2);
    assert_eq!(
        client_orders(&app).len(),
        1,
        "back on the computer's player: its keys give nothing again"
    );
}
