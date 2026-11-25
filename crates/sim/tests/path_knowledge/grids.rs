//! The players' grids: built lazily, with a player's first walking unit, and derived, so that a save with
//! stale knowledge loads equal and continues identically.

use super::*;

/// The players' grids are built lazily: a player gets them with its first unit that walks on a grid, and an order
/// in the very tick of that unit's spawn already plans on what the player knows.
#[test]
fn a_players_grids_are_built_with_its_first_walking_unit() {
    let mut t = ScenarioBuilder::new(&map(true))
        .spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)) // 1
        .spawn(PK, Some(P1), SpawnAt::Point(point(WEST))) // 2
        .start();
    t.without_auto_targeting();
    assert!(t.sim().known_nav(P1).is_some() && t.sim().known_nav(P2).is_none());
    t.run(3);
    assert_eq!(t.sim().nav_rebuilds(), 0);
    // Player 2's first unit, ordered east of its own Generator in the tick it appears.
    t.spawn_unit_at(4, PK, P2, at(20.5, 7.0));
    t.order_at(
        4,
        P2,
        &[ObjectId(3)],
        OrderKind::Move(Target::Point(point(EAST))),
        false,
    );
    t.step();
    assert!(known_blocked(t.sim(), P2, WALL, GAP));
    assert_eq!(t.sim().nav_rebuilds(), 0, "built for the order, not rebuilt");
    // A unit acts from the tick after its spawn: its first plan.
    t.step();
    assert!(
        t.mover(3).path.len() > 2,
        "around its own Generator from the start: {:?}",
        t.mover(3).path
    );
    let track = walk(&mut t, 3);
    assert_eq!((t.status(3), t.pos(3)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track) && !went_through_the_gap(&track));
}

/// Determinism: the grids are derived and rebuilt on load, the assumed resources and the versions are state. A
/// save at any point of a game with stale knowledge loads equal (state, hash and each player's grids) and
/// continues tick for tick like the game it was taken from; and the game replays to the same state.
#[test]
fn a_save_with_stale_knowledge_loads_and_continues_identically() {
    let game = || {
        let mut t = ScenarioBuilder::new(&map(true))
            .spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)) // 1
            .spawn(ObjectKind::CrystalPatch, None, tile(30, 12)) // 2
            .spawn(ObjectKind::CrystalPatch, None, tile(3, 15)) // 3
            .spawn(PK, Some(P1), SpawnAt::Point(at(21.5, 7.0))) // 4
            .spawn(PK, Some(P2), SpawnAt::Point(at(33.5, 9.0))) // 5
            // Player 2's other building, never in player 1's sight: losing the Generator in the gap doesn't defeat
            // them (plan decision T8), and the game goes on.
            .spawn(ObjectKind::Generator, Some(P2), tile(0, 0)) // 6
            .start();
        t.without_auto_targeting();
        let order = |unit: u32, to: (f32, f32)| (unit, OrderKind::Move(Target::Point(point(to))));
        // Player 1 scouts the Generator, leaves, comes back the long way and looks at the gap; player 2 crosses
        // the map past the patches. The Generator goes while nobody of player 1 looks.
        for (tick, (unit, kind)) in [
            (1, order(4, WEST)),
            (2, order(5, (5.5, 15.0))),
            (260, order(4, EAST)),
            (900, order(4, BY_THE_GAP)),
            (1000, order(4, WEST)),
        ] {
            let player = if unit == 4 { P1 } else { P2 };
            t.order_at(tick, player, &[ObjectId(unit)], kind, false);
        }
        t.debug_at(250, DebugCommand::Remove { id: ObjectId(1) });
        t.debug_at(255, DebugCommand::Remove { id: ObjectId(3) });
        t
    };
    let mut t = game();
    let mut versions = Vec::new();
    for k in [0, 5, 249, 251, 400, 899, 960, 1100] {
        t.run_until(k);
        let json = serde_json::to_string(t.sim()).unwrap();
        let mut loaded: Sim = serde_json::from_str(&json).unwrap();
        assert_eq!(&loaded, t.sim(), "tick {k}");
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {k}");
        assert_eq!(loaded.path_knowledge(), t.sim().path_knowledge(), "tick {k}");
        for player in [P1, P2] {
            assert_eq!(
                loaded.known_nav(player),
                t.sim().known_nav(player),
                "tick {k}, {player:?}"
            );
        }
        let mut twin = game();
        twin.run_until(k);
        for _ in 0..120 {
            twin.step();
            loaded.step(&twin.replay().inputs[twin.sim().tick() as usize - 1]);
            assert_eq!(
                loaded.state_hash(),
                twin.sim().state_hash(),
                "from tick {k}, at {}",
                twin.sim().tick()
            );
        }
        versions.push((version(t.sim(), P1), version(t.sim(), P2)));
    }
    assert!(
        versions
            .windows(2)
            .all(|pair| pair[0].0 <= pair[1].0 && pair[0].1 <= pair[1].1)
    );
    // Player 1 saw the Generator gone (a patch it passed only moved from assumed to remembered); player 2 saw
    // the Generator gone too, and the patch in the west gone.
    assert_eq!(*versions.last().unwrap(), (1, 2), "{versions:?}");
    t.run_until(1300);
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());

    // The knowledge is part of the save: one that doesn't fit the game is refused, never a panic.
    let mut value: serde_json::Value = serde_json::to_value(t.sim()).unwrap();
    assert!(value["path_knowledge"][0]["version"].is_u64());
    value["path_knowledge"].as_array_mut().unwrap().pop();
    let error = serde_json::from_value::<Sim>(value).unwrap_err().to_string();
    assert!(error.contains("path knowledge"), "{error}");
}
