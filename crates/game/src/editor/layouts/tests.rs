use super::*;
use sc_sim::{BuildingOrientation, MapSettings, SpawnAt, Symmetry, TilePos};

fn skirmish() -> MapDef {
    MapSettings {
        map_type: MapType::SkirmishDefault,
        width: 40,
        height: 24,
        slots: 2,
        symmetry: Symmetry::Rotate180,
        name: "t".into(),
        description: String::new(),
    }
    .create()
    .unwrap()
}

fn object(kind: ObjectKind, x: i32, y: i32) -> LayoutObject {
    LayoutObject {
        kind,
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation: BuildingOrientation::default(),
    }
}

/// `> Skirmish Default restriction`: the GDO's Starting Conditions with their counts left; on a Custom map every
/// type a player may own; nothing for a faction not implemented yet (T11).
#[test]
fn the_palette_offers_what_the_layout_may_take() {
    let mut map = skirmish();
    let slot = SlotId(1);
    let labels = |map: &MapDef| -> Vec<String> {
        palette(map, slot, Faction::Gdo)
            .iter()
            .map(PaletteEntry::label)
            .collect()
    };
    assert_eq!(
        labels(&map),
        [
            "Deployment Center: 1 left",
            "Extraction Facility: 1 left",
            "Extraction Plate: 2 left"
        ]
    );
    let layouts = &mut map.slot_mut(slot).unwrap().layouts;
    layouts
        .get_mut(Faction::Gdo)
        .push(object(ObjectKind::DeploymentCenter, 2, 2));
    layouts
        .get_mut(Faction::Gdo)
        .push(object(ObjectKind::ExtractionPlate, 9, 2));
    assert_eq!(labels(&map)[0], "Deployment Center: done");
    assert_eq!(labels(&map)[2], "Extraction Plate: 1 left");
    assert_eq!(
        missing_text(
            &StartingConditions::of(Faction::Gdo)
                .unwrap()
                .missing(layout(&map, slot, Faction::Gdo))
        ),
        "Extraction Facility, Extraction Plate"
    );
    assert!(palette(&map, slot, Faction::Cults).is_empty());
    map.map_type = MapType::Custom;
    let custom = palette(&map, slot, Faction::Gdo);
    assert!(
        custom
            .iter()
            .any(|e| e.kind == ObjectKind::Barracks && e.left.is_none())
    );
    assert!(custom.iter().all(|e| e.kind.def().category != Category::Resource));
}

/// Slice M4-B: the Syndicate's palette on a Skirmish Default map holds its three kinds, one each
/// (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`: one Tier 1 Tunnel, one Hideout, one Agent), and
/// its missing list names them.
#[test]
fn the_syndicate_palette_holds_its_three_kinds() {
    let mut map = skirmish();
    let slot = SlotId(1);
    let labels = |map: &MapDef| -> Vec<String> {
        palette(map, slot, Faction::Syndicate)
            .iter()
            .map(PaletteEntry::label)
            .collect()
    };
    assert_eq!(labels(&map), ["Tunnel: 1 left", "Hideout: 1 left", "Agent: 1 left"]);
    let syndicate = StartingConditions::of(Faction::Syndicate).unwrap();
    assert_eq!(missing_text(&syndicate.missing(&[])), "Tunnel, Hideout, Agent");
    let layouts = &mut map.slot_mut(slot).unwrap().layouts;
    layouts
        .get_mut(Faction::Syndicate)
        .push(object(ObjectKind::Tunnel, 4, 4));
    assert_eq!(labels(&map), ["Tunnel: done", "Hideout: 1 left", "Agent: 1 left"]);
}

#[test]
fn missing_lists_read_like_the_design() {
    let gdo = StartingConditions::of(Faction::Gdo).unwrap();
    assert_eq!(
        missing_text(&gdo.missing(&[])),
        "Deployment Center, Extraction Facility, Extraction Plate x2"
    );
    assert_eq!(layout_name(SlotId(3), Faction::Gdo), "S3 GDO layout");
}

/// `> Outdated layouts`, derived (T11): any layout that isn't the current Starting Conditions is flagged; on a
/// Custom map, which takes anything, the line only says that it differs.
#[test]
fn layouts_unlike_the_starting_conditions_are_flagged() {
    let mut map = skirmish();
    assert_eq!(
        outdated_text(&map),
        None,
        "empty layouts are not outdated, only incomplete"
    );
    let gdo = map.slot_mut(SlotId(1)).unwrap().layouts.get_mut(Faction::Gdo);
    gdo.push(object(ObjectKind::DeploymentCenter, 2, 2));
    map.regenerate();
    assert_eq!(
        outdated_text(&map).unwrap(),
        "Outdated or unfinished: S1 GDO, S2 GDO (not the faction's current Starting Conditions; nothing is fixed \
         automatically)"
    );
    map.map_type = MapType::Custom;
    assert_eq!(
        outdated_text(&map).unwrap(),
        "Not the Starting Conditions: S1 GDO, S2 GDO (fine on a Custom map; outdated if made for older ones)"
    );
}

/// While the Layout tool edits a faction, the preview gives every slot a player of it.
#[test]
fn the_preview_plays_the_edited_faction() {
    let map = skirmish();
    assert_eq!(preview_scenario(&map, None), Scenario::new(map.clone()));
    let cults = preview_scenario(&map, Some(Faction::Cults));
    assert!(cults.setup.players.iter().all(|p| p.faction == Faction::Cults));
    assert_eq!(cults.setup.players.len(), 2);
    assert_eq!(cults.setup.seed, Scenario::new(map).setup.seed);
}
