//! The Defeat Condition (`core/game_rules.md > Defeat Condition`, plan decision T8): the last structure lost,
//! plates included, to a debug removal or in combat; never for a player who never owned one.

use super::*;

/// T8, by debug removal: player 2 loses their Generator but still has a plate, which counts as a structure
/// (`> Defeat Condition`: "including GDO Extraction Plates"); they are defeated in the tick the plate goes, with one
/// event, and in a two-player game that ends it.
#[test]
fn losing_the_last_structure_to_a_removal_defeats_the_player_plates_included() {
    let mut t = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(20, 1)) // 2
        .spawn(ObjectKind::CrystalPatch, None, tile(20, 8)) // 3
        .spawn(ObjectKind::ExtractionPlate, Some(P2), tile(20, 8)) // 4
        .start();
    t.debug_at(3, remove(2)).run_until(5);
    assert_eq!(defeated_at(&t, P2), None, "the plate is left");
    assert_eq!(t.sim().outcome(), None);
    t.debug_at(6, remove(4)).run_until(8);
    assert_eq!(defeated_at(&t, P2), Some(6));
    assert_eq!(defeated_at(&t, P1), None);
    assert!(t.sim().player(P2).unwrap().is_defeated());
    assert_eq!(game_events(&t), [(6, defeated(P2)), (6, game_over(&[P1]))]);
    // The patch under the plate had crystals left: it stays, nobody's.
    assert!(t.sim().object(ObjectId(3)).is_some());
}

/// T8, in combat: player 1's Peacekeepers destroy player 2's last structure, a plate, by themselves (design change
/// 0103: enemy structures are automatic targets). Player 2 is defeated in the tick it dies, and their Peacekeeper,
/// far away, becomes neutral in that same tick.
#[test]
fn losing_the_last_structure_in_combat_defeats_the_player() {
    let mut t = ScenarioBuilder::new(&flat(32, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 1)) // 1
        .spawn(ObjectKind::Generator, Some(P2), tile(28, 1)) // 2
        .spawn(ObjectKind::CrystalPatch, None, tile(8, 6)) // 3
        .spawn(ObjectKind::ExtractionPlate, Some(P2), tile(8, 6)) // 4
        .spawn(PK, Some(P1), SpawnAt::Point(centre(5, 5))) // 5
        .spawn(PK, Some(P1), SpawnAt::Point(centre(5, 7))) // 6
        .spawn(PK, Some(P2), SpawnAt::Point(centre(28, 9))) // 7: far from it all
        .start();
    t.debug_at(1, remove(2)).run_until(400);
    let died = t.events().iter().find_map(|(tick, e)| match e {
        SimEvent::Died { id, .. } if *id == ObjectId(4) => Some(*tick),
        _ => None,
    });
    let died = died.expect("the plate is destroyed");
    assert_eq!(defeated_at(&t, P2), Some(died));
    assert_eq!(game_events(&t), [(died, defeated(P2)), (died, game_over(&[P1]))]);
    let unit = t.sim().object(ObjectId(7)).unwrap();
    assert_eq!(unit.owner, None);
    assert_eq!(unit.hp, Some(50.0), "nobody shot at it");
}

/// T8: a player who never owned a structure, as in the M1 sandboxes, is never defeated, even with no object left;
/// and a structure that comes and goes within one tick never counted (the count is taken between ticks).
#[test]
fn players_who_never_owned_a_structure_are_never_defeated() {
    let mut t = ScenarioBuilder::new(&flat(16, 8))
        .spawn(PK, Some(P1), SpawnAt::Point(centre(4, 4))) // 1
        .spawn(PK, Some(P1), SpawnAt::Point(centre(4, 2))) // 2
        .spawn(PK, Some(P2), SpawnAt::Point(centre(7, 4))) // 3
        .start();
    t.run_until(300);
    assert!(t.sim().object(ObjectId(3)).is_none(), "player 2 has nothing left");
    // A Generator spawned and removed in one tick (it gets id 4).
    t.debug_at(301, spawn(ObjectKind::Generator, P2, tile(12, 1)));
    t.debug_at(301, remove(4));
    t.run_until(310);
    assert!(t.sim().object(ObjectId(4)).is_none());
    assert_eq!(game_events(&t), []);
    assert!(t.sim().players().iter().all(|p| p.defeated_at.is_none()));
    assert_eq!(t.sim().outcome(), None);
    let order = Command::Debug(spawn(PK, P2, SpawnAt::Point(centre(12, 6))));
    for player in [P1, P2] {
        let command = sc_sim::PlayerCommand {
            player,
            command: order.clone(),
        };
        assert_eq!(t.sim().validate(&command), Ok(()));
    }
}
