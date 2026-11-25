//! Layouts and the Starting Conditions (`core/map_editor.md > Starting Layouts > Skirmish Default
//! restriction`, design changes 0009 and 0012): a Skirmish Default map's layouts hold exactly them, and a
//! layout that doesn't is flagged as outdated (`> Outdated layouts`).

use super::*;

/// Design changes 0009 and 0012 (`core/map_editor.md > Starting Layouts > Skirmish Default restriction`): a new
/// Skirmish Default map's layouts are empty, and it can't be played until the GDO's and the Colonists' (slice M3-C,
/// plan decision T24) are complete at every start; the 180° symmetry mirrors a layout onto the opposite start,
/// turning its buildings; a third plate, a Barracks, or anything in the layout of a faction not implemented yet (T11),
/// is beyond the Starting Conditions; an owned object outside the layouts is refused. A Custom map has no such
/// restriction.
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
    let lacks = |slot, faction, kind, missing| ScenarioProblem::LayoutMissing {
        slot: SlotId(slot),
        faction,
        kind,
        missing,
    };
    let missing = |slot, kind, missing| lacks(slot, Faction::Gdo, kind, missing);
    let colonists = |slot| {
        [
            (ObjectKind::EgressNode, 1),
            (ObjectKind::RoboPortal, 1),
            (ObjectKind::Drone, 4),
        ]
        .map(|(kind, n)| lacks(slot, Faction::Colonists, kind, n))
    };
    let empty: Vec<ScenarioProblem> = [missing(1, DC, 1), missing(1, EF, 1), missing(1, PLATE, 2)]
        .into_iter()
        .chain(colonists(1))
        .chain([missing(2, DC, 1), missing(2, EF, 1), missing(2, PLATE, 2)])
        .chain(colonists(2))
        .collect();
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
    assert_eq!(map.validate(), [colonists(1), colonists(2)].concat());
    *map.slots[0].layouts.get_mut(Faction::Colonists) = west_colonists();
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
