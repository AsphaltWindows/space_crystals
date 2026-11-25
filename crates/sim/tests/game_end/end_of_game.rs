//! The end of the game (`core/game_rules.md > End of Game`): a 1v1 ends with the first defeat, three players
//! with the second, and players defeated in the same tick draw (design change 0163).

use super::*;

/// Design change 0101's verification 3 (the sim's half): in a 1v1, the defeat of one player ends the game in that
/// tick, with the outcome in the hashed state, and from then on the sim accepts no command at all. It still steps if
/// asked; nobody is defeated after the end.
#[test]
fn a_1v1_ends_with_the_first_defeat() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1)) // 2
        .spawn(PK, Some(P2), SpawnAt::Point(centre(20, 8))) // 3
        .start();
    let hash = t.run_until(3).sim().state_hash();
    t.debug_at(4, remove(2)).run_until(4);
    let outcome = Outcome {
        tick: 4,
        winners: vec![P1],
    };
    assert_eq!(t.sim().outcome(), Some(&outcome));
    assert!(outcome.is_winner(P1) && !outcome.is_winner(P2));
    // In order: the defeat, then the end.
    let tick_events: Vec<&SimEvent> = t
        .sim()
        .events()
        .iter()
        .filter(|e| matches!(e, SimEvent::PlayerDefeated { .. } | SimEvent::GameOver { .. }))
        .collect();
    assert_eq!(tick_events, [&defeated(P2), &game_over(&[P1])]);
    assert_ne!(t.sim().state_hash(), hash);

    let command = |player, command| sc_sim::PlayerCommand { player, command };
    for player in [P1, P2] {
        for c in [
            Command::Debug(remove(1)),
            Command::Order {
                units: vec![ObjectId(3)],
                kind: OrderKind::Stop,
                queue: false,
            },
        ] {
            assert_eq!(t.sim().validate(&command(player, c)), Err(Reject::GameOver));
        }
    }
    // Player 1's last structure goes after the end: refused, and no defeat would count anyway.
    t.debug_at(5, remove(1)).run_until(40);
    assert!(t.sim().object(ObjectId(1)).is_some());
    assert_eq!(t.sim().tick(), 40);
    assert_eq!(t.sim().outcome(), Some(&outcome));
    assert_eq!(game_events(&t), [(4, defeated(P2)), (4, game_over(&[P1]))]);
}

/// `> End of Game` with three players, every player their own side (no teams): the first defeat leaves two
/// standing and the game goes on; the second ends it, the last player standing winning. The defeated player's
/// commands are refused in between, the others' are not.
#[test]
fn three_players_end_with_the_second_defeat() {
    let mut t = three_way();
    t.debug_at(3, remove(3)).run_until(5);
    assert_eq!(defeated_at(&t, P3), Some(3));
    assert_eq!(t.sim().outcome(), None);
    assert!(t.sim().allies(P3).is_empty(), "no teams: no allies");
    t.debug_at(6, remove(2)).run_until(8);
    assert_eq!(defeated_at(&t, P2), Some(6));
    assert_eq!(
        t.sim().outcome(),
        Some(&Outcome {
            tick: 6,
            winners: vec![P1],
        })
    );
    assert_eq!(
        game_events(&t),
        [(3, defeated(P3)), (6, defeated(P2)), (6, game_over(&[P1]))]
    );
}

/// Design change 0163 (`> End of Game > Draw`: "if the last remaining players or teams are all defeated in the same
/// tick [...] the game ends as a draw. Nobody wins"), its Verification: two players lose their last structures in
/// the same tick. The game ends in that tick without a winner, a draw.
#[test]
fn players_defeated_in_the_same_tick_draw() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1)) // 2
        .start();
    t.debug_at(4, remove(2));
    t.debug_at(4, remove(1));
    t.run_until(4);
    assert_eq!(
        game_events(&t),
        [(4, defeated(P1)), (4, defeated(P2)), (4, game_over(&[]))]
    );
    let outcome = t.sim().outcome().unwrap();
    assert!(outcome.is_draw() && outcome.tick == 4);
    assert!(!outcome.is_winner(P1) && !outcome.is_winner(P2));
}

/// Design change 0163 with three players: player 3 is defeated first and the game goes on; then the last two fall in
/// one tick, and the game is a draw, which player 3 sees too (`> Draw`: "every player sees 'Draw'"): nobody won.
#[test]
fn the_last_two_of_three_defeated_in_the_same_tick_draw() {
    let mut t = three_way();
    t.debug_at(3, remove(3)).run_until(5);
    assert_eq!(t.sim().outcome(), None);
    t.debug_at(6, remove(1)).debug_at(6, remove(2)).run_until(8);
    let outcome = t.sim().outcome().unwrap();
    assert_eq!(
        outcome,
        &Outcome {
            tick: 6,
            winners: vec![]
        }
    );
    assert!(outcome.is_draw());
    assert!([P1, P2, P3].iter().all(|&p| !outcome.is_winner(p)));
    assert_eq!(
        game_events(&t),
        [
            (3, defeated(P3)),
            (6, defeated(P1)),
            (6, defeated(P2)),
            (6, game_over(&[]))
        ]
    );
}
