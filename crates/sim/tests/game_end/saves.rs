//! Determinism across a defeat and the end: a game saved and loaded around them continues identically.

use super::*;

/// Determinism across a defeat and the end: player 1 attacks into player 2's corner and destroys their Peacekeepers
/// and Generator, while player 3 loses theirs to a removal. Saved and loaded around each defeat and after the end,
/// the game continues exactly as the original; the replay runs to the same state; and a save from before the end of
/// the game existed (no `outcome`, no `defeated_at`) loads as a game going on.
#[test]
fn a_save_across_a_defeat_loads_and_continues_identically() {
    let game = || {
        let mut t = three_way();
        // Four more Peacekeepers for player 1 (8 to 11), and all six attack.
        for y in [7, 9, 11, 13] {
            t.spawn_unit_at(1, PK, P1, centre(8, y));
        }
        let army: Vec<ObjectId> = [4, 5, 8, 9, 10, 11].map(ObjectId).to_vec();
        t.order_at(2, P1, &army, OrderKind::AttackMove(centre(27, 4)), false);
        t.debug_at(30, remove(3));
        t
    };
    let mut t = game();
    t.run_until(1500);
    let end = t.sim().outcome().cloned().expect("player 1 won");
    assert_eq!(end.winners, [P1]);
    let p2_fell = defeated_at(&t, P2).unwrap();
    assert!(30 < p2_fell && p2_fell == end.tick, "{p2_fell}");

    let mut t = game();
    for k in [0, 29, 30, 31, p2_fell - 1, p2_fell, p2_fell + 1, p2_fell + 50] {
        t.run_until(k);
        let json = serde_json::to_string(t.sim()).unwrap();
        let mut loaded: Sim = serde_json::from_str(&json).unwrap();
        assert_eq!(&loaded, t.sim(), "tick {k}");
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {k}");
        let mut twin = game();
        twin.run_until(k);
        for _ in 0..60 {
            twin.step();
            loaded.step(&twin.replay().inputs[twin.sim().tick() as usize - 1]);
            assert_eq!(
                loaded.state_hash(),
                twin.sim().state_hash(),
                "from tick {k}, at {}",
                twin.sim().tick()
            );
        }
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());

    // Older saves: without the outcome and the defeats, a game going on.
    let mut t = game();
    t.run_until(10);
    let mut value = serde_json::to_value(t.sim()).unwrap();
    value.as_object_mut().unwrap().remove("outcome");
    for player in value["players"].as_array_mut().unwrap() {
        player.as_object_mut().unwrap().remove("defeated_at");
    }
    let loaded: Sim = serde_json::from_value(value).unwrap();
    assert_eq!(&loaded, t.sim());
    // A winner who isn't in the game is refused.
    let mut value = serde_json::to_value(t.sim()).unwrap();
    value["outcome"] = serde_json::json!({ "tick": 3, "winners": [7] });
    let error = serde_json::from_value::<Sim>(value).unwrap_err().to_string();
    assert!(error.contains("winner 7"), "{error}");
}
