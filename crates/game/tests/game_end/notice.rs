//! The defeat notice (slice M5-S1; `core/game_rules.md > On Defeat`: "**Everyone learns of it** (user, 2026-11-26;
//! Q0090): when a player is defeated while the game goes on, **all players** are told", design change 0357;
//! `space_crystals::hud::notices`): every other player sees it, each in their own view (the debug player switch, F2),
//! for `FeedbackTuning::notice_ticks`, a line for each defeat; the defeated player sees their results screen; and the
//! defeat that ends the game is told by the banner alone.

use space_crystals::hud::notices::{DefeatNoticeText, DefeatNoticeView, defeat_text};

use super::*;

const P4: PlayerId = PlayerId(4);

/// A four-player game on a flat 32 × 16 map: each player's Generator, 1 to 4, in a corner of their own: (1, 1),
/// (28, 1), (1, 13) and (28, 13).
fn four_way() -> Scenario {
    let mut scenario = ScenarioBuilder::new(&vec![".".repeat(32); 16].join("\n"))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Generator, Some(P2), tile(28, 1))
        .spawn(ObjectKind::Generator, Some(P3), tile(1, 13))
        .spawn(ObjectKind::Generator, Some(P4), tile(28, 13))
        .build();
    scenario.map.slots.push(Slot::new(TilePos::new(4, 12)));
    scenario.map.slots.push(Slot::new(TilePos::new(26, 12)));
    scenario.setup = GameSetup::default_for(&scenario.map);
    scenario
}

/// The defeat notice's text while it shows.
fn notice(app: &mut App) -> Option<String> {
    let world = app.world_mut();
    let shown = world
        .query_filtered::<&Visibility, With<DefeatNoticeView>>()
        .single(world)
        .is_ok_and(|v| *v != Visibility::Hidden);
    let text = world
        .query_filtered::<&Text, With<DefeatNoticeText>>()
        .single(world)
        .ok()?
        .0
        .clone();
    shown.then_some(text)
}

/// Taps F2, the debug player switch, for a frame: the local player it shows then.
fn switch_player(app: &mut App) -> PlayerId {
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F2);
    app.update();
    app.world().resource::<LocalPlayer>().0
}

/// The plan's acceptance (addendum S1, design change 0357): in the three-player game, player 3 loses their Generator,
/// their only structure, and is defeated while the game goes on. Player 1 is told from that tick; F2 shows player 2
/// told alike, then player 3 their results screen and no notice, and player 1 again. The notice goes for both after
/// `FeedbackTuning::notice_ticks`.
#[test]
fn every_other_player_is_told_of_a_defeat() {
    let mut app = game_as(AppConfig::new(three_way()), P1);
    run_until(&mut app, 3);
    assert_eq!(notice(&mut app), None);
    remove(&mut app, 3);
    let defeat = tick(&app);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.player(P3).unwrap().is_defeated());
    assert_eq!(sim.outcome(), None, "the game goes on");
    let told = Some(defeat_text("Player 3"));
    assert_eq!(notice(&mut app), told, "player 1, in the tick of the defeat");
    assert_eq!(switch_player(&mut app), P2);
    assert_eq!(notice(&mut app), told, "player 2");
    assert_eq!(switch_player(&mut app), P3);
    assert_eq!(notice(&mut app), None, "the defeated player");
    assert!(results_screen(&mut app).is_some());
    assert_eq!(switch_player(&mut app), P1);
    assert_eq!(notice(&mut app), told);
    run_until(
        &mut app,
        defeat + space_crystals::tuning::feedback::BUILTIN.notice_ticks - 1,
    );
    assert_eq!(notice(&mut app), told);
    app.update();
    assert_eq!(
        tick(&app),
        defeat + space_crystals::tuning::feedback::BUILTIN.notice_ticks
    );
    assert_eq!(notice(&mut app), None, "gone after notice_ticks");
    assert_eq!(switch_player(&mut app), P2);
    assert_eq!(notice(&mut app), None);
}

/// Two defeats while the game goes on, players 3 and 4 a few ticks apart: player 1 is told of both, a line each in
/// the order they fell, and so is player 2. Then player 2's defeat ends the game, player 1 winning: the banner tells it
/// (`core/game_rules.md > End of Game`: "Every player sees Victory or Defeat"), and no notice shows any more.
#[test]
fn each_defeat_is_a_line_and_the_last_one_the_banners() {
    let mut app = game_as(AppConfig::new(four_way()), P1);
    run_until(&mut app, 3);
    remove(&mut app, 3);
    run_until(&mut app, 8);
    remove(&mut app, 4);
    let both = Some(format!("{}\n{}", defeat_text("Player 3"), defeat_text("Player 4")));
    assert_eq!(notice(&mut app), both);
    assert_eq!(switch_player(&mut app), P2);
    assert_eq!(notice(&mut app), both, "player 2");
    assert_eq!(switch_player(&mut app), P3);
    assert_eq!(
        notice(&mut app),
        None,
        "player 3 left the game before player 4's defeat"
    );
    switch_player(&mut app);
    assert_eq!(switch_player(&mut app), P1);

    remove(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    assert_eq!(sim.outcome().map(|end| end.winners.clone()), Some(vec![P1]));
    assert_eq!(notice(&mut app), None);
    assert_eq!(banner(&mut app), shown("VICTORY", "The game is over"));
}
