//! The start of a game: each player's objects exactly where their faction's layout at their slot puts them
//! (design change 0008), and `dev_skirmish`'s GDO bases, which see and reach their fields (0095; plan risk
//! R13).

use super::*;

/// Each player's banks: Space Crystals and Supplies.
fn banks(sim: &Sim) -> Vec<(u32, u32)> {
    sim.players().iter().map(|p| (p.crystals, p.supplies)).collect()
}

/// Design change 0008 and the plan's acceptance of the game start: each player's objects are exactly where their
/// faction's layout at their slot puts them, turned as it turns them, and none of another faction's layout comes; a
/// GDO player has 100 Space Crystals, 0 Supplies (design change 0077) and power +9, 21 used out of 30; the layout's
/// plates mine their patches from the start, and pay at the end of the first cycle. A Cults player starts with empty
/// banks, their Starting Conditions' (slice M5-B).
#[test]
fn a_game_starts_with_each_players_starting_conditions() {
    let mut s = ScenarioBuilder::new(&flat(24, 12))
        .spawn(PATCH, None, tile(1, 8)) // 1
        .spawn(PATCH, None, tile(2, 8)) // 2
        .spawn(PATCH, None, tile(21, 2)) // 3
        .spawn(PATCH, None, tile(22, 2)) // 4
        .build();
    *s.map.slots[0].layouts.get_mut(Faction::Gdo) = west_base(); // 5 to 8
    *s.map.slots[1].layouts.get_mut(Faction::Gdo) = vec![
        laid(DC, 19, 7),                  // 9: (19, 7) to (22, 10)
        turned(EF, 15, 8, Rotation::R90), // 10: (15, 8) to (17, 10)
        laid(PLATE, 21, 2),               // 11
        laid(PLATE, 22, 2),               // 12
    ];
    // A Custom map's layouts may hold any objects (`core/map_editor.md > Map Types > Custom`), the Cults' here a
    // Recruit and its Hall, whose Recruitment Area, (6, 0) to (15, 9), lies on the map, so that none of it is lost
    // (design change 0387: "The part of its 10x10 area beyond the map edge yields **no Recruitable tiles**"); nobody
    // plays the Cults.
    *s.map.slots[0].layouts.get_mut(Faction::Cults) = vec![
        unit_at(RECRUIT, point(12.0, 8.0)),
        laid(HALL, 9, 3), // (9, 3) to (12, 6)
    ];
    let sim = Sim::try_new(&s).unwrap();
    let owned: Vec<_> = sim
        .objects()
        .filter(|o| o.owner.is_some())
        .map(|o| (o.id.0, o.kind, o.owner, o.pos, o.placement.map(|p| p.rotation)))
        .collect();
    let (r0, r90) = (Some(Rotation::R0), Some(Rotation::R90));
    assert_eq!(
        owned,
        [
            (5, DC, Some(P1), point(3.0, 3.0), r0),
            (6, EF, Some(P1), point(7.5, 2.5), r0),
            (7, PLATE, Some(P1), point(1.5, 8.5), r0),
            (8, PLATE, Some(P1), point(2.5, 8.5), r0),
            (9, DC, Some(P2), point(21.0, 9.0), r0),
            (10, EF, Some(P2), point(16.5, 9.5), r90),
            (11, PLATE, Some(P2), point(21.5, 2.5), r0),
            (12, PLATE, Some(P2), point(22.5, 2.5), r0),
        ]
    );
    assert_eq!(banks(&sim), [(100, 0), (100, 0)]);
    for player in [P1, P2] {
        let power = sim.power(player);
        assert_eq!((power.generated, power.consumed), (30, 21), "{player:?}");
    }
    let patch_of = |sim: &Sim, id: u32| sim.object(ObjectId(id)).unwrap().extractor.map(|e| e.patch.0);
    let links: Vec<_> = [7, 8, 11, 12].into_iter().map(|id| patch_of(&sim, id)).collect();
    assert_eq!(links, [Some(1), Some(2), Some(3), Some(4)]);
    let mut t = TestSim::new(s.clone());
    t.run_until(47);
    assert_eq!(banks(t.sim()), [(100, 0), (100, 0)]);
    t.run_until(48);
    assert_eq!(banks(t.sim()), [(120, 0), (120, 0)], "10 from each plate");

    // Player 1 plays the Cults: their layout at slot 1, its Recruit of its Hall's lineage though listed before it
    // (`cults_start.rs`), and their Starting Conditions' banks, nothing (`factions/cults/economy.md > Starting
    // Conditions`: "**0 Space Crystals**").
    s.setup.players[0].faction = Faction::Cults;
    let sim = Sim::try_new(&s).unwrap();
    let p1: Vec<_> = sim
        .objects()
        .filter(|o| o.owner == Some(P1))
        .map(|o| (o.id.0, o.kind, o.pos, o.lineage.clone()))
        .collect();
    assert_eq!(
        p1,
        [
            (5, RECRUIT, point(12.0, 8.0), vec![ObjectId(6)]),
            (6, HALL, point(11.0, 5.0), vec![]),
        ]
    );
    assert_eq!(banks(&sim), [(0, 0), (100, 0)]);
}

/// `dev_skirmish`'s GDO bases (design change 0095; plan risk R13): each Deployment Center sees every patch of its
/// field from the start, by its own sight alone (6 grid units; the base and the field lie at level 0, so no
/// elevation modifier), and its build area holds all eight (`factions/gdo/economy.md > Opening Economy Baseline >
/// Reference starting base`), so a plate can go onto each uncovered one at once (0095's verification 1). The layouts
/// are complete, the second one the image of the first, and not outdated.
#[test]
fn dev_skirmish_bases_see_and_reach_their_fields() {
    let scenario = scenarios::dev_skirmish();
    assert_eq!(scenario.validate(), []);
    assert_eq!(scenario.map.outdated_layouts(), []);
    let r180 = |kind, x, y| turned(kind, x, y, Rotation::R180);
    assert_eq!(
        scenario.map.slots[1].layouts.get(Faction::Gdo).to_vec(),
        [
            r180(DC, 55, 39),
            r180(EF, 52, 44),
            r180(PLATE, 57, 45),
            r180(PLATE, 56, 45)
        ]
    );
    let sim = Sim::try_new(&scenario).unwrap();
    let half = scenario.map.terrain.width() as f32 * SU_PER_GU / 2.0;
    for player in [P1, P2] {
        let dc = sim.objects().find(|o| o.owner == Some(player) && o.kind == DC).unwrap();
        let field: Vec<_> = sim
            .objects()
            .filter(|o| o.kind == PATCH && (o.pos.x < half) == (player == P1))
            .collect();
        assert_eq!(field.len(), 8, "{player:?}");
        let area = sim.build_area(player);
        let plated: Vec<_> = sim.objects().filter_map(|o| o.extractor.map(|e| e.patch)).collect();
        assert_eq!(plated.len(), 4);
        for patch in field {
            let t = patch.placement.unwrap().origin;
            let distance = patch.pos.distance(dc.pos) / SU_PER_GU;
            assert!(distance <= 6.0, "{player:?} {t:?}: {distance} grid units");
            assert_eq!(sim.visibility(player, t), VisibilityState::Visible, "{player:?} {t:?}");
            assert!(area.contains(t), "{player:?} {t:?}");
            let check = sim.placement_check(player, PLATE, t, BuildingOrientation::default());
            let free = !plated.contains(&patch.id);
            assert_eq!(check.is_valid(), free, "{player:?} {t:?}: {check:?}");
        }
    }
}
