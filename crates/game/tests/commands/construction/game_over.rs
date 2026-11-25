//! No construction once the game is over or the player defeated (design change 0101): nothing is constructed
//! or placed, and the placement ends.

use super::*;

/// Design change 0101 (`core/game_rules.md > End of Game`): once the game is over nobody gives commands. Player 2
/// loses their last structure while player 1 places a Generator: the placement ends with its ghost, its overlay and
/// its keys, the command panel goes, and neither the keys nor clicks construct or place anything.
#[test]
fn once_the_game_is_over_nothing_is_constructed_or_placed() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    assert!(ghost(&app).is_some() && overlay(&mut app).1 && placement_keys(&app));
    let given = constructions(&app).len();

    let remove = DebugCommand::Remove {
        id: ObjectId(ENEMY_GENERATOR),
    };
    submit(&mut app, P1, Command::Debug(remove));
    updates(&mut app, 2);
    let outcome = app.world().resource::<SimRes>().outcome().cloned();
    assert!(outcome.is_some_and(|o| o.is_winner(P1)));
    assert_eq!(interface(&app), InterfaceState::DefaultState, "placement ends");
    assert_eq!(ghost(&app), None);
    assert!(!overlay(&mut app).1, "no overlay");
    assert!(!placement_keys(&app), "R and F are no placement keys");
    assert!(!ghost_is_cursor(&app), "the pointer is back");
    assert_eq!(buttons(&mut app), [], "no command panel");

    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    click(&mut app, middle(OPEN, (2, 2)), &[]);
    tap(&mut app, &[], KeyX);
    assert_eq!(constructions(&app).len(), given);
    assert_eq!(constructor(&app, DC), Constructor::Ready { kind: GENERATOR });
    assert_eq!(structure_at(&app, GENERATOR, OPEN), None);
}

/// Design change 0101 (`core/game_rules.md > On Defeat`): a player defeated while the game goes on gives no commands
/// either (slice M2-J2's check of the placement keys, which the game reports outside the command input). Player 1
/// loses their structures while placing a Generator, with players 2 and 3 still in the game: the placement ends with
/// its ghost, its overlay, its keys and its pointer, and R, F and the build menu's keys do nothing.
#[test]
fn a_player_defeated_while_placing_places_nothing() {
    let mut scenario = construction_scenario();
    scenario.map.slots.push(sc_sim::Slot::new(TilePos::new(56, 4)));
    scenario.map.spawns.push(Spawn::new(
        GENERATOR,
        Some(SlotId(3)),
        SpawnAt::Tile(TilePos::new(56, 2)),
    ));
    scenario.setup = sc_sim::GameSetup::default_for(&scenario.map);
    let mut app = game_of(scenario);
    set_crystals(&mut app, 1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    assert!(ghost(&app).is_some() && overlay(&mut app).1 && placement_keys(&app) && ghost_is_cursor(&app));
    let given = constructions(&app).len();

    for id in [DC, EF] {
        submit(&mut app, P1, Command::Debug(DebugCommand::Remove { id: ObjectId(id) }));
    }
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.player(P1).unwrap().is_defeated() && sim.outcome().is_none());
    assert_eq!(interface(&app), InterfaceState::DefaultState, "placement ends");
    assert_eq!(ghost(&app), None);
    assert!(!overlay(&mut app).1, "no overlay");
    assert!(!placement_keys(&app), "R and F are no placement keys");
    assert!(!ghost_is_cursor(&app), "the pointer is back");
    assert_eq!(buttons(&mut app), [], "no command panel");
    for key in [KeyR, KeyF, KeyQ, KeyQ] {
        tap(&mut app, &[], key);
        assert!(!placement_keys(&app));
    }
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(constructions(&app).len(), given);
}
