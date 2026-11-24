//! The banners: the winner sees Victory and the loser Defeat, and the game stops at its last tick, with
//! nobody giving orders any more; when the last sides fall in the same tick, everyone sees Draw (design
//! change 0163).

use super::*;

/// Design change 0101's verification 3, the client's half: when player 2 loses their last building, player 1 sees
/// Victory and player 2 Defeat (the debug player switch shows each player's banner), the game stops at that tick, and
/// from then on the command panel gives no orders.
#[test]
fn the_winner_sees_victory_the_loser_defeat_and_the_game_stops() {
    let mut app = game_as(AppConfig::new(duel()), P1);
    run_until(&mut app, 3);
    assert_eq!(banner(&mut app), None);
    assert!(
        stop_peacekeeper(&mut app),
        "while the game goes on, X stops the Peacekeeper"
    );

    remove(&mut app, 2);
    let end = tick(&app);
    let sim = app.world().resource::<SimRes>();
    assert_eq!(
        sim.outcome().map(|o| (o.tick, o.winners.clone())),
        Some((end, vec![P1]))
    );
    assert_eq!(banner(&mut app), shown("VICTORY", "The game is over"));
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(tick(&app), end, "the game stopped");
    assert!(!stop_peacekeeper(&mut app), "no more orders");

    app.world_mut().resource_mut::<LocalPlayer>().0 = P2;
    app.update();
    assert_eq!(banner(&mut app), shown("DEFEAT", "The game is over"));
    // The loser owns nothing: what still shows Visible to them is the fog delay of the tiles they saw last (design
    // change 0178), in which the game stopped.
    let sim = app.world().resource::<SimRes>();
    assert!(sim.objects().all(|o| o.owner != Some(P2)));
    let vision = sim.vision().player(P2).unwrap();
    let (w, h) = (sim.map().width() as i32, sim.map().height() as i32);
    let tiles = (0..h).flat_map(|y| (0..w).map(move |x| TilePos::new(x, y)));
    let visible: Vec<TilePos> = tiles.filter(|&t| vision.state(t) == VisibilityState::Visible).collect();
    assert!(!visible.is_empty() && visible.iter().all(|&t| vision.lingers(t)));
    assert_eq!(Banner::of(sim, P1), Some(Banner::Victory));
    assert_eq!(Banner::of(sim, PlayerId(9)), None, "not in the game");
}

/// Design change 0163's Verification: "Two players destroy each other's last building in the same tick: both see
/// 'Draw'; replay saved." The game stops at that tick, and nobody gives orders any more.
#[test]
fn players_defeated_in_the_same_tick_both_see_a_draw() {
    let path = std::env::temp_dir()
        .join(format!("sc_game_end_draw_test_{}", std::process::id()))
        .join("draw.json");
    let mut config = AppConfig::new(duel());
    config.record = Some(path.clone());
    let mut app = game_as(config, P1);
    run_until(&mut app, 3);
    remove_together(&mut app, &[1, 2]);
    let end = tick(&app);
    let outcome = app.world().resource::<SimRes>().outcome().cloned().unwrap();
    assert!(outcome.is_draw() && outcome.tick == end);
    assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
    assert_eq!(
        results_screen(&mut app),
        None,
        "defeated in the last tick: no results screen"
    );
    let recorder = app.world().resource::<Recorder>();
    assert!(
        recorder.is_finished() && recorder.saved_at() == Some(end),
        "replay saved"
    );
    let replay = load_replay(&path).unwrap();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    assert_eq!(replay.run().outcome(), Some(&outcome));
    for _ in 0..20 {
        app.update();
    }
    assert_eq!(tick(&app), end, "the game stopped");
    assert!(!stop_peacekeeper(&mut app), "no more orders");

    view_as(&mut app, P2);
    assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
    let sim = app.world().resource::<SimRes>();
    assert_eq!([Banner::of(sim, P1), Banner::of(sim, P2)], [Some(Banner::Draw); 2]);
}
