//! The start of a game (slice M2-F1): each player gets their faction's Starting Conditions (`factions/gdo/economy.md >
//! Starting Conditions`), the banks and the objects of their faction's Starting Layout at their slot, exactly where
//! the map puts them (design change 0008). The rules of the layouts (`core/map_editor.md > Starting Layouts`): each
//! one valid on its own with its faction's placement rules minus sight (0097); on a Skirmish Default map exactly the
//! Starting Conditions, complete at every start (0009, 0012), for the factions implemented so far (plan decision
//! T11); derived "outdated" flags. And `dev_skirmish`'s GDO bases (0095; plan risk R13). The Starting Conditions
//! table is unit-tested in `scenario/starting.rs`, the camera's start in the client.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Faction, LayoutObject, MapSettings, MapType, ObjectId, ObjectKind, PlacementCheck, PlayerId,
    Rotation, SU_PER_GU, Scenario, ScenarioProblem, Sim, SlotId, Spawn, SpawnAt, SpawnProblem, TilePos, TileProblem,
    Vec2, VisibilityState, WholeProblem, scenarios,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DC: ObjectKind = ObjectKind::DeploymentCenter;
const EF: ObjectKind = ObjectKind::ExtractionFacility;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
const PK: ObjectKind = ObjectKind::Peacekeeper;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

fn laid(kind: ObjectKind, x: i32, y: i32) -> LayoutObject {
    LayoutObject {
        kind,
        at: tile(x, y),
        orientation: BuildingOrientation::default(),
    }
}

fn turned(kind: ObjectKind, x: i32, y: i32, rotation: Rotation) -> LayoutObject {
    LayoutObject {
        orientation: BuildingOrientation {
            rotation,
            ..Default::default()
        },
        ..laid(kind, x, y)
    }
}

fn unit_at(kind: ObjectKind, at: Vec2) -> LayoutObject {
    LayoutObject {
        kind,
        at: SpawnAt::Point(at),
        orientation: BuildingOrientation::default(),
    }
}

fn patch(x: i32, y: i32) -> Spawn {
    Spawn::new(PATCH, None, tile(x, y))
}

/// A GDO base in the north-west of a map: the Deployment Center from (1, 1) to (4, 4), the Extraction Facility from
/// (6, 1) to (8, 3), and plates on patches at (1, 8) and (2, 8).
fn west_base() -> Vec<LayoutObject> {
    vec![laid(DC, 1, 1), laid(EF, 6, 1), laid(PLATE, 1, 8), laid(PLATE, 2, 8)]
}

/// Each player's banks: Space Crystals and Supplies.
fn banks(sim: &Sim) -> Vec<(u32, u32)> {
    sim.players().iter().map(|p| (p.crystals, p.supplies)).collect()
}

/// Design change 0008 and the plan's acceptance of the game start: each player's objects are exactly where their
/// faction's layout at their slot puts them, turned as it turns them, and none of another faction's layout comes; a
/// GDO player has 100 Space Crystals, 0 Supplies (design change 0077) and power +9, 21 used out of 30; the layout's
/// plates mine their patches from the start, and pay at the end of the first cycle. A player of a faction whose
/// Starting Conditions aren't implemented yet starts with empty banks (T11).
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
    // A Custom map's layouts may hold any objects (`core/map_editor.md > Map Types > Custom`); nobody plays the
    // Syndicate.
    *s.map.slots[0].layouts.get_mut(Faction::Syndicate) = vec![unit_at(PK, point(12.0, 6.0))];
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

    // Player 1 plays the Syndicate: its layout at slot 1, and no Starting Conditions yet.
    s.setup.players[0].faction = Faction::Syndicate;
    let sim = Sim::try_new(&s).unwrap();
    let p1: Vec<_> = sim
        .objects()
        .filter(|o| o.owner == Some(P1))
        .map(|o| (o.kind, o.pos))
        .collect();
    assert_eq!(p1, [(PK, point(12.0, 6.0))]);
    assert_eq!(banks(&sim), [(0, 0), (100, 0)]);
}

/// Design changes 0009 and 0012 (`core/map_editor.md > Starting Layouts > Skirmish Default restriction`): a new
/// Skirmish Default map's layouts are empty, and it can't be played until the GDO's is complete at every start; the
/// 180° symmetry mirrors a layout onto the opposite start, turning its buildings; a third plate, a Barracks, or
/// anything in the layout of a faction not implemented yet (T11), is beyond the Starting Conditions; an owned object
/// outside the layouts is refused. A Custom map has no such restriction.
#[test]
fn skirmish_default_layouts_hold_exactly_the_starting_conditions() {
    let mut map = MapSettings {
        width: 24,
        height: 16,
        ..Default::default()
    }
    .create()
    .unwrap();
    assert_eq!((map.map_type, map.slots.len()), (MapType::SkirmishDefault, 2));
    let missing = |slot, kind, missing| ScenarioProblem::LayoutMissing {
        slot: SlotId(slot),
        faction: Faction::Gdo,
        kind,
        missing,
    };
    let empty = vec![
        missing(1, DC, 1),
        missing(1, EF, 1),
        missing(1, PLATE, 2),
        missing(2, DC, 1),
        missing(2, EF, 1),
        missing(2, PLATE, 2),
    ];
    assert_eq!(map.validate(), empty);
    assert_eq!(
        empty[2].to_string(),
        "slot 1 Gdo layout lacks 2 ExtractionPlate of the Starting Conditions: a Skirmish Default map is played with \
         every layout complete"
    );
    assert_eq!(Sim::try_new(&Scenario::new(map.clone())), Err(empty));

    // The west half is the editable part: its patches and the first slot's layout are mirrored onto the east half.
    map.spawns = vec![patch(1, 8), patch(2, 8), patch(3, 8)];
    *map.slots[0].layouts.get_mut(Faction::Gdo) = west_base();
    map.regenerate();
    assert_eq!(map.validate(), []);
    let r180 = |kind, x, y| turned(kind, x, y, Rotation::R180);
    assert_eq!(
        map.slots[1].layouts.get(Faction::Gdo).to_vec(),
        [
            r180(DC, 19, 11),
            r180(EF, 15, 12),
            r180(PLATE, 22, 7),
            r180(PLATE, 21, 7)
        ]
    );
    let sim = Sim::try_new(&Scenario::new(map.clone())).unwrap();
    let p2: Vec<_> = sim
        .objects()
        .filter(|o| o.owner == Some(P2))
        .filter_map(|o| Some((o.kind, o.placement?.origin, o.placement?.rotation)))
        .collect();
    let at = TilePos::new;
    assert_eq!(
        p2,
        [
            (DC, at(19, 11), Rotation::R180),
            (EF, at(15, 12), Rotation::R180),
            (PLATE, at(22, 7), Rotation::R180),
            (PLATE, at(21, 7), Rotation::R180),
        ]
    );

    let mut extra = map.clone();
    let gdo = extra.slots[0].layouts.get_mut(Faction::Gdo);
    gdo.push(laid(PLATE, 3, 8));
    gdo.push(laid(ObjectKind::Barracks, 6, 5)); // (6, 5) to (8, 6)
    *extra.slots[0].layouts.get_mut(Faction::Syndicate) = vec![unit_at(PK, point(9.5, 7.5))];
    extra.regenerate();
    let beyond = |slot, faction, index, kind, allowed| ScenarioProblem::LayoutExtra {
        slot: SlotId(slot),
        faction,
        index,
        kind,
        allowed,
    };
    let (gdo, syndicate) = (Faction::Gdo, Faction::Syndicate);
    let expected = vec![
        beyond(1, gdo, 4, PLATE, 2),
        beyond(1, gdo, 5, ObjectKind::Barracks, 0),
        beyond(1, syndicate, 0, PK, 0),
        beyond(2, gdo, 4, PLATE, 2),
        beyond(2, gdo, 5, ObjectKind::Barracks, 0),
        beyond(2, syndicate, 0, PK, 0),
    ];
    assert_eq!(extra.validate(), expected);
    let messages: Vec<_> = expected[..3].iter().map(ToString::to_string).collect();
    assert_eq!(
        messages,
        [
            "slot 1 Gdo layout object 4 (ExtractionPlate): beyond the 2 of the Gdo Starting Conditions: a Skirmish \
             Default layout holds only those",
            "slot 1 Gdo layout object 5 (Barracks): not in the Gdo Starting Conditions: a Skirmish Default layout \
             holds only those",
            "slot 1 Syndicate layout object 0 (Peacekeeper): the Syndicate Starting Conditions aren't implemented \
             yet: a Skirmish Default layout of the faction stays empty",
        ]
    );
    extra.map_type = MapType::Custom;
    assert_eq!(extra.validate(), [], "a Custom map's layouts may hold anything");

    let mut incomplete = map.clone();
    incomplete.slots[0].layouts.get_mut(Faction::Gdo).remove(1); // the Extraction Facility
    incomplete.regenerate();
    let expected = vec![missing(1, EF, 1), missing(2, EF, 1)];
    assert_eq!(incomplete.validate(), expected);
    assert_eq!(Sim::try_new(&Scenario::new(incomplete)), Err(expected));

    // `core/map_editor.md > Map Types > Skirmish Default`: player-owned objects only in the layouts.
    let mut owned = map.clone();
    owned
        .spawns
        .push(Spawn::new(ObjectKind::Generator, Some(SlotId(1)), tile(9, 8)));
    owned.regenerate();
    let refused = |spawn| ScenarioProblem::InvalidSpawn {
        spawn,
        kind: ObjectKind::Generator,
        problem: SpawnProblem::OwnedOnSkirmishMap,
    };
    assert_eq!(owned.validate(), [refused(3), refused(7)]);
}

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

/// `core/map_editor.md > Starting Layouts > Outdated layouts`, derived (T11): a layout that holds objects but not
/// exactly its faction's Starting Conditions is flagged, an empty one isn't, nor one of a faction not implemented
/// yet. A flag only: on a Custom map, whose layouts may hold anything, it is no problem. (A layout made for other
/// Starting Conditions is in `scenario/starting.rs`.)
#[test]
fn layouts_that_dont_match_the_starting_conditions_are_flagged_as_outdated() {
    let mut map = scenarios::empty_map(24, 12);
    map.spawns = vec![patch(1, 8), patch(2, 8)];
    *map.slots[0].layouts.get_mut(Faction::Gdo) = west_base();
    *map.slots[0].layouts.get_mut(Faction::Syndicate) = vec![laid(DC, 15, 2)];
    assert_eq!(map.outdated_layouts(), []);
    *map.slots[1].layouts.get_mut(Faction::Gdo) = vec![laid(DC, 15, 2)];
    map.slots[0].layouts.get_mut(Faction::Gdo).pop();
    assert_eq!(
        map.outdated_layouts(),
        [(SlotId(1), Faction::Gdo), (SlotId(2), Faction::Gdo)]
    );
    assert_eq!(map.validate(), []);
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
