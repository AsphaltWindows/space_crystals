//! The rules of a layout (`core/map_editor.md > Starting Layouts > Layout validation`, design change 0097):
//! its faction's placement rules minus sight, a structure over one of its units reported once, a plate onto a
//! patch (pre-placed ones too), and the layouts of a game's players kept apart (`Scenario::validate`).

use super::*;

/// Design change 0097 (`core/map_editor.md > Starting Layouts > Layout validation`): a layout follows its faction's
/// placement rules minus sight, on any map. The GDO's Extraction Facility and plates need a tile in the build area
/// of the Deployment Center in the same layout, not of one pre-placed for the slot; a structure can't cover a unit;
/// starting units can't stand on impassable ground; and there is no other check: a Deployment Center that sees none
/// of its plates' patches is fine. Layouts of different factions may overlap.
#[test]
fn each_layout_follows_its_factions_placement_rules_without_sight() {
    // 40 × 12, a cliff at (30, 9).
    let mut rows = vec![".".repeat(40); 12];
    rows[9] = format!("{}c{}", ".".repeat(30), ".".repeat(9));
    let mut s = ScenarioBuilder::new(&rows.join("\n"))
        .spawn(PATCH, None, tile(13, 1)) // 0
        .spawn(PATCH, None, tile(14, 1)) // 1
        .spawn(PK, None, SpawnAt::Point(TilePos::new(7, 7).center())) // 2
        .build();
    fn gdo(s: &mut Scenario) -> &mut Vec<LayoutObject> {
        s.map.slots[0].layouts.get_mut(Faction::Gdo)
    }
    // The Deployment Center's area reaches x 16: the Extraction Facility needs one tile there. The patches lie 10
    // grid units from the Deployment Center, out of its sight of 6.
    *gdo(&mut s) = vec![laid(DC, 1, 1), laid(EF, 16, 1), laid(PLATE, 13, 1), laid(PLATE, 14, 1)];
    assert_eq!(s.validate(), []);
    let refused = |index, kind, tiles, whole| ScenarioProblem::LayoutPlacement {
        slot: SlotId(1),
        faction: Faction::Gdo,
        index,
        kind,
        check: PlacementCheck { tiles, whole },
    };
    gdo(&mut s)[1] = laid(EF, 17, 1);
    let outside = refused(1, EF, vec![], vec![WholeProblem::OutsideBuildArea]);
    assert_eq!(s.map.validate(), std::slice::from_ref(&outside));
    assert_eq!(
        outside.to_string(),
        "slot 1 Gdo layout object 1 (ExtractionFacility): no tile is in the build area"
    );
    // Over the Peacekeeper: the Extraction Facility from (6, 6) to (8, 8).
    gdo(&mut s)[1] = laid(EF, 6, 6);
    let over_unit = refused(1, EF, vec![(TilePos::new(7, 7), TileProblem::Unit)], vec![]);
    assert_eq!(s.map.validate(), std::slice::from_ref(&over_unit));
    assert_eq!(
        over_unit.to_string(),
        "slot 1 Gdo layout object 1 (ExtractionFacility): under a ground unit at tile (7, 7)"
    );
    // A starting unit on the cliff.
    gdo(&mut s)[1] = laid(EF, 16, 1);
    gdo(&mut s).push(unit_at(PK, TilePos::new(30, 9).center()));
    let blocked = ScenarioProblem::InvalidLayoutObject {
        slot: SlotId(1),
        faction: Faction::Gdo,
        index: 4,
        kind: PK,
        problem: SpawnProblem::Blocked,
    };
    assert_eq!(s.map.validate(), [blocked]);
    gdo(&mut s).pop();
    // Another faction's layout over the GDO's.
    *s.map.slots[0].layouts.get_mut(Faction::Syndicate) = vec![laid(DC, 2, 2)];
    assert_eq!(s.validate(), []);

    // Only the area of the layout's own Deployment Center counts.
    s.map.slots[0].layouts.get_mut(Faction::Syndicate).clear();
    s.map.spawns.push(Spawn::new(DC, Some(SlotId(1)), tile(1, 1)));
    *gdo(&mut s) = vec![laid(PLATE, 13, 1)];
    assert_eq!(
        s.map.validate(),
        [refused(0, PLATE, vec![], vec![WholeProblem::OutsideBuildArea])]
    );
}

/// A layout structure over one of its layout's units is one problem, the structure's placement problem
/// (`TileProblem::Unit`), whichever of the two comes first in the layout: the unit-start rule doesn't report the
/// unit as blocked by it as well. A unit on impassable ground, or over a pre-placed structure, is still the unit's
/// problem.
#[test]
fn a_layout_structure_over_a_layout_unit_is_reported_once() {
    let mut s = ScenarioBuilder::new(&flat(24, 12))
        .spawn(ObjectKind::Generator, None, tile(16, 8)) // 0: (16, 8) to (17, 9)
        .build();
    fn gdo(s: &mut Scenario) -> &mut Vec<LayoutObject> {
        s.map.slots[0].layouts.get_mut(Faction::Gdo)
    }
    let unit = unit_at(PK, TilePos::new(7, 7).center());
    let over_unit = |index| ScenarioProblem::LayoutPlacement {
        slot: SlotId(1),
        faction: Faction::Gdo,
        index,
        kind: EF,
        check: PlacementCheck {
            tiles: vec![(TilePos::new(7, 7), TileProblem::Unit)],
            whole: vec![],
        },
    };
    // The Extraction Facility from (6, 6) to (8, 8), before the unit and after it.
    *gdo(&mut s) = vec![laid(DC, 1, 1), laid(EF, 6, 6), unit.clone()];
    assert_eq!(s.map.validate(), [over_unit(1)]);
    *gdo(&mut s) = vec![laid(DC, 1, 1), unit.clone(), laid(EF, 6, 6)];
    assert_eq!(s.map.validate(), [over_unit(2)]);
    // A unit over the pre-placed Generator is the unit's problem, after a layout structure as well.
    *gdo(&mut s) = vec![
        laid(DC, 1, 1),
        laid(EF, 6, 6),
        unit_at(PK, TilePos::new(16, 8).center()),
    ];
    let blocked = ScenarioProblem::InvalidLayoutObject {
        slot: SlotId(1),
        faction: Faction::Gdo,
        index: 2,
        kind: PK,
        problem: SpawnProblem::Blocked,
    };
    assert_eq!(s.map.validate(), [blocked]);
}

/// A layout's plate goes exactly onto a patch (`factions/gdo/economy.md > ExtractionPlate`: "Placed onto a Space
/// Crystal Patch"): one off its patch is the plate's [`ScenarioProblem::LayoutPlacement`], by the rule a game's
/// placement uses, `WholeProblem::NotOnPatch` of the placement check that both modes share (design change 0097), so
/// the two can't drift apart. `dev_skirmish`'s layouts, plates on patches, are valid, the built-in map and its file.
#[test]
fn a_layout_plate_goes_onto_a_patch_by_the_games_rule() {
    let mut map = scenarios::empty_map(24, 12);
    map.spawns = vec![patch(1, 8), patch(2, 8)];
    *map.slots[0].layouts.get_mut(Faction::Gdo) = west_base();
    assert_eq!(map.validate(), []);
    // The second plate a tile east of its patch, on open ground in the build area.
    map.slots[0].layouts.get_mut(Faction::Gdo)[3] = laid(PLATE, 3, 8);
    let off = ScenarioProblem::LayoutPlacement {
        slot: SlotId(1),
        faction: Faction::Gdo,
        index: 3,
        kind: PLATE,
        check: PlacementCheck {
            tiles: vec![],
            whole: vec![WholeProblem::NotOnPatch],
        },
    };
    assert_eq!(map.validate(), std::slice::from_ref(&off));
    assert_eq!(
        off.to_string(),
        "slot 1 Gdo layout object 3 (ExtractionPlate): an Extraction Plate goes onto a Space Crystals patch"
    );
    // In a game from the valid layout, the placement check refuses the same tile for the same reason.
    map.slots[0].layouts.get_mut(Faction::Gdo)[3] = laid(PLATE, 2, 8);
    let sim = Sim::try_new(&Scenario::new(map)).unwrap();
    let in_game = sim.placement_check(P1, PLATE, TilePos::new(3, 8), BuildingOrientation::default());
    assert_eq!(in_game.whole, [WholeProblem::NotOnPatch]);

    assert_eq!(scenarios::dev_skirmish().validate(), []);
    let file: sc_sim::MapDef = serde_json::from_str(include_str!("../../../../maps/dev_skirmish.json")).unwrap();
    assert_eq!(file.validate(), []);
}

/// A plate pre-placed on the map, not in a layout, goes onto a patch too (`factions/gdo/economy.md >
/// ExtractionPlate`: "Placed onto a Space Crystal Patch"; M2-I's finding, fixed in slice M2-J2), by the rule of a
/// layout's plate and a game's placement (`placement::Onto`), so the three can't drift apart. One off its patch is
/// its spawn's `SpawnProblem::NotOnPatch`, and the game doesn't start: it would start the plate on nothing, and the
/// plate would never mine. On its patch it is valid, before or after the patch in the list, owned or neutral.
#[test]
fn a_pre_placed_plate_off_a_patch_is_refused() {
    let mut map = scenarios::empty_map(24, 12); // a Custom map, where player-owned objects may be pre-placed
    map.spawns = vec![patch(1, 8), Spawn::new(PLATE, Some(SlotId(1)), tile(3, 8))];
    let off = |spawn| ScenarioProblem::InvalidSpawn {
        spawn,
        kind: PLATE,
        problem: SpawnProblem::NotOnPatch,
    };
    assert_eq!(map.validate(), [off(1)]);
    assert_eq!(
        off(1).to_string(),
        "spawn 1 (ExtractionPlate): an Extraction Plate goes onto a Space Crystals patch"
    );
    assert_eq!(Sim::try_new(&Scenario::new(map.clone())).err(), Some(vec![off(1)]));

    map.spawns[1].at = tile(1, 8);
    map.spawns.swap(0, 1);
    assert_eq!(map.validate(), []);
    let sim = Sim::try_new(&Scenario::new(map.clone())).unwrap();
    assert!(
        sim.objects().any(|o| o.kind == PLATE && o.extractor.is_some()),
        "it mines its patch"
    );
    map.spawns[0].owner = None;
    assert_eq!(map.validate(), []);
    map.spawns[0].at = tile(5, 8);
    assert_eq!(map.validate(), [off(0)], "a neutral plate off its patch");
}

/// Each layout is valid on its own, but the players of one game get theirs together: one in another's way keeps the
/// game from starting (`Scenario::validate`), not the map.
#[test]
fn the_layouts_of_a_games_players_keep_apart() {
    let mut s = ScenarioBuilder::new(&flat(16, 8)).build();
    *s.map.slots[0].layouts.get_mut(Faction::Gdo) = vec![laid(DC, 2, 2)]; // (2, 2) to (5, 5)
    *s.map.slots[1].layouts.get_mut(Faction::Gdo) = vec![laid(DC, 4, 3)]; // (4, 3) to (7, 6)
    assert_eq!(s.map.validate(), []);
    let overlap = ScenarioProblem::LayoutOverlap {
        slot: SlotId(2),
        faction: Faction::Gdo,
        index: 0,
        tile: TilePos::new(4, 3),
    };
    assert_eq!(s.validate(), [overlap]);
    s.setup.players.truncate(1);
    assert_eq!(s.validate(), [], "nobody plays slot 2");
}
