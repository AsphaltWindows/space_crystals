//! The results screen (design change 0164): a player defeated with no ally left leaves for it, also before
//! a draw, and a replay shows it; a spectator's union of visions (0102).

use super::*;

/// Design change 0164's Verification: "3-player FFA: the first defeated player is moved to a results screen; the
/// other two keep playing." Player 3 has no allies (no teams), so their defeat takes them to the results screen at
/// once: DEFEATED, their name and faction, when it happened, and that the game goes on. It covers the world view and
/// the HUD Bar, below the top bar, so they see nothing of the world, and no banner says Spectating. The game goes
/// on, and players 1 and 2 play as before: no screen, no banner, their commands given.
#[test]
fn a_player_defeated_with_no_ally_left_leaves_for_the_results_screen() {
    let mut app = game_as(AppConfig::new(three_way()), P3);
    run_until(&mut app, 3);
    assert_eq!(results_screen(&mut app), None);
    remove(&mut app, 3);
    let defeat = tick(&app);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.player(P3).unwrap().is_defeated() && sim.allies(P3).is_empty());
    assert_eq!(sim.outcome(), None);
    run_until(&mut app, defeat + 20);
    assert_eq!(
        results_screen(&mut app),
        lines([
            "DEFEATED",
            "Player 3 (GDO)",
            &format!("Defeated at 0:00 (tick {defeat})"),
            "The game goes on for the other players",
        ])
    );
    assert_eq!(banner(&mut app), None, "not spectating");
    assert_eq!(Banner::of(app.world().resource::<SimRes>(), P3), None);
    // Everything below the top bar, opaque, over the HUD.
    let world = app.world_mut();
    let (node, background, z) = world
        .query_filtered::<(&Node, &BackgroundColor, &GlobalZIndex), With<ResultsScreen>>()
        .single(world)
        .unwrap();
    assert_eq!(
        (node.top, node.bottom, node.left, node.width),
        (vh(TOP_BAR_HEIGHT_VH), px(0), px(0), percent(100))
    );
    assert_eq!((background.0.alpha(), z.0), (1.0, RESULTS_Z));
    const { assert!(RESULTS_Z > 0 && BANNER_Z > RESULTS_Z) };
    let hud_z: Vec<i32> = world
        .query_filtered::<Option<&GlobalZIndex>, With<HudRoot>>()
        .iter(world)
        .map(|z| z.map_or(0, |z| z.0))
        .collect();
    assert_eq!(hud_z, [0, 0], "the top bar and the HUD Bar lie under it");

    // The game goes on: players 1 and 2 still play, and see nothing of this.
    assert_eq!(tick(&app), defeat + 20);
    for player in [P1, P2] {
        view_as(&mut app, player);
        assert_eq!((results_screen(&mut app), banner(&mut app)), (None, None));
    }
    view_as(&mut app, P1);
    let selection = Selection::new([(ObjectId(4), PK)]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
    app.world_mut().resource_mut::<PendingCommands>().0.clear();
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::KeyX);
    app.update();
    let pending = &app.world().resource::<PendingCommands>().0;
    assert!(
        pending
            .iter()
            .any(|c| c.player == P1 && matches!(c.command, Command::Order { .. })),
        "player 1 still gives orders"
    );
    run_until(&mut app, defeat + 40);
    assert_eq!(app.world().resource::<SimRes>().outcome(), None);
    // The debug player switch back to player 3 shows their screen again.
    view_as(&mut app, P3);
    assert!(results_screen(&mut app).is_some());
}

/// Design change 0163 with a player defeated before the end: player 3 leaves for the results screen; then players
/// 1 and 2 lose their last buildings in the same tick. The game is a draw, and all three see Draw: players 1 and 2
/// over the world, player 3 over their results screen, which now says the game is over.
#[test]
fn a_player_defeated_earlier_sees_the_draw_on_the_results_screen() {
    let mut app = game_as(AppConfig::new(three_way()), P3);
    run_until(&mut app, 3);
    remove(&mut app, 3);
    let defeat = tick(&app);
    run_until(&mut app, 10);
    assert_eq!(banner(&mut app), None);
    remove_together(&mut app, &[1, 2]);
    let end = tick(&app);
    let sim = app.world().resource::<SimRes>();
    assert_eq!(sim.outcome().map(|o| (o.tick, o.is_draw())), Some((end, true)));
    assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
    assert_eq!(
        results_screen(&mut app),
        lines([
            "DEFEATED",
            "Player 3 (GDO)",
            &format!("Defeated at 0:00 (tick {defeat})"),
            "The game is over",
        ])
    );
    for player in [P1, P2] {
        view_as(&mut app, player);
        assert_eq!(banner(&mut app), shown("DRAW", "The game is over"));
        assert_eq!(results_screen(&mut app), None, "player {player:?} was there at the end");
    }
}

/// Design change 0164 in a replay: played back as player 3, the playback shows their results screen from their
/// defeat on, as the game did; the debug player switch shows the others' views, the game going on in them.
#[test]
fn a_replay_shows_the_results_screen_of_a_player_who_left() {
    let mut scenario = three_way();
    scenario.setup.config = SimConfig::DEBUG;
    let mut replay = Replay::new(scenario);
    let mut sim = replay.start();
    replay.record_checked(None, &sim);
    while sim.tick() < 40 {
        let mut inputs = TickInputs::default();
        if sim.tick() + 1 == 10 {
            inputs.commands.push(PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove { id: ObjectId(3) }),
            });
        }
        sim.step(&inputs);
        replay.record_checked(Some(inputs), &sim);
    }
    replay.checkpoint(&sim);

    let mut app = game_as(AppConfig::playback(replay, PathBuf::from("left.json")), P3);
    run_until(&mut app, 9);
    assert_eq!(results_screen(&mut app), None);
    run_until(&mut app, 12);
    assert_eq!(
        results_screen(&mut app),
        lines([
            "DEFEATED",
            "Player 3 (GDO)",
            "Defeated at 0:00 (tick 10)",
            "The game goes on for the other players",
        ])
    );
    view_as(&mut app, P1);
    assert_eq!((results_screen(&mut app), banner(&mut app)), (None, None));
}

/// Design change 0102's mechanism, through `Sight::watching` (the sim has no teams to make player 2 an ally of player
/// 3): the union of player 3's vision and player 2's shows what player 2 sees, live, and what either remembers; the
/// rest stays hidden.
#[test]
fn a_spectator_watches_through_the_union_of_the_visions() {
    let mut app = game_as(AppConfig::new(three_way()), P3);
    run_until(&mut app, 3);
    remove(&mut app, 3);
    // Past the fog delay of the tiles player 3 saw (design change 0178).
    run_until(&mut app, 8 + FOG_DELAY_TICKS);
    let sim = app.world().resource::<SimRes>();
    let alone = Sight::new(sim, P3, true);
    let union = Sight::watching(sim, P3, &[P3, P2], true);
    let object = |id: u32| sim.object(ObjectId(id)).unwrap();

    // Player 2's Peacekeeper and the patch beside it: live in the union; unseen, or remembered, alone.
    assert!(union.sees(object(5)) && !alone.sees(object(5)));
    assert!(union.sees(object(7)) && union.remembered_record(ObjectId(7)).is_none());
    assert!(!alone.sees(object(7)) && alone.remembered_record(ObjectId(7)).is_some());
    // The patch only player 3 saw: remembered either way.
    assert!(!union.sees(object(8)) && union.remembered_record(ObjectId(8)).is_some());
    let remembered: Vec<u32> = union.remembered().map(|r| r.id.0).collect();
    assert_eq!(remembered, [8]);
    // Player 1's Peacekeeper, seen by neither: hidden.
    assert!(!union.sees(object(4)) && !alone.sees(object(4)));

    // The tiles: the best state of the two visions.
    assert_eq!(union.tile(TilePos::new(24, 8)), VisibilityState::Visible);
    assert_eq!(union.tile(TilePos::new(18, 12)), VisibilityState::Explored);
    assert_eq!(union.tile(TilePos::new(8, 8)), VisibilityState::Unexplored);
    let (w, h) = (sim.map().width(), sim.map().height());
    let alpha = |sight: &Sight, x: u32, y: u32| fog_texels(sight, w, h)[((y * w + x) * 4 + 3) as usize];
    let expected = |a: f32| (a * 255.0).round() as u8;
    assert_eq!(alpha(&union, 24, 8), 0);
    assert_eq!(alpha(&union, 18, 12), expected(EXPLORED_ALPHA));
    assert_eq!(alpha(&union, 8, 8), expected(UNEXPLORED_ALPHA));
    assert_eq!(alpha(&alone, 24, 8), expected(UNEXPLORED_ALPHA));
    // Whom a sight watches is part of its revision, so the fog layer redraws when it changes.
    assert_ne!(union.revision(), alone.revision());
}
