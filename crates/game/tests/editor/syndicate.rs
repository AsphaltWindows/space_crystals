//! The Syndicate's Starting Layouts in the map editor (slice M4-B; `core/map_editor.md > Starting Layouts`,
//! `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`): on a Skirmish Default map the Layout tool's
//! palette is their Tunnel, Hideout and Agent; a starting Tunnel goes only where its whole Tier 1 Tunnel Area is on
//! the map and Drillable (design change 0242), and the Hideout only wholly inside that area, the ghost red and the
//! status line saying the rule elsewhere; and the map plays once their layout is complete too.

use bevy::prelude::App;
use sc_sim::{
    ArmyCap, ArmyResource, BuildingOrientation, Faction, LayoutObject, MapDef, ObjectKind, PlayerId, Rotation,
    SU_PER_GU, SlotId, SpawnAt, Symmetry, Tile, TilePos, TilePresetId, Vec2,
};
use space_crystals::ClientMode;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::layouts::check::LayoutPreview;
use space_crystals::editor::layouts::{PaletteEntry, palette};
use space_crystals::editor::ui::lists::ProblemList;
use space_crystals::editor::ui::starting::LayoutPanel;

use super::colonists::with_colonists;
use super::cults::with_cults;
use super::layouts::{click, laid, skirmish, spot, texts_under, with_patches};
use super::{act, assert_warned, doc, editor, sim, temp_maps, tools};

const S1: SlotId = SlotId(1);
const S2: SlotId = SlotId(2);
const TUNNEL: ObjectKind = ObjectKind::Tunnel; // 4 × 4 tiles
const HIDEOUT: ObjectKind = ObjectKind::Hideout; // 2 × 2
const AGENT: ObjectKind = ObjectKind::Agent;
const UNTURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R0,
    flip_h: false,
    flip_v: false,
};

/// Where [`syndicate_base`] puts its Agent, in grid units: 1.5 south of the Tunnel, clear of it and of the Hideout.
const AGENT_AT: (f32, f32) = (8.0, 19.5);

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

/// A Syndicate Starting Layout for slot 1 of the 32 × 24 maps of the layout tests, clear of their patches and of
/// the lake of [`syndicate_test_map`]: the Tunnel on (6, 14) to (9, 17), its Tier 1 Tunnel Area (4, 12) to
/// (11, 19); the Hideout on (10, 18) to (11, 19), in that area; the Agent at [`AGENT_AT`].
fn syndicate_base() -> Vec<LayoutObject> {
    vec![
        laid(TUNNEL, 6, 14, UNTURNED),
        laid(HIDEOUT, 10, 18, UNTURNED),
        LayoutObject {
            kind: AGENT,
            at: SpawnAt::Point(point(AGENT_AT.0, AGENT_AT.1)),
            orientation: UNTURNED,
        },
    ]
}

/// `map` with [`syndicate_base`] at slot 1 and its images: for the tests of other layouts on a Skirmish Default map,
/// which can't be played without it (slice M4-B).
pub(super) fn with_syndicate(mut map: MapDef) -> MapDef {
    *map.slot_mut(S1).unwrap().layouts.get_mut(Faction::Syndicate) = syndicate_base();
    map.regenerate();
    map
}

/// The map of the Colonists' layout test (a 32 × 24 Skirmish Default map turned 180°, patches on (3, 3) and (4, 3)),
/// the GDO's, the Colonists' and the Cults' layouts done, and a column of Water on x = 2 from y = 12 to 20, with
/// their images.
fn syndicate_test_map() -> MapDef {
    let mut map = with_cults(with_colonists(with_patches(
        skirmish(32, 24, 2, Symmetry::Rotate180),
        &[(3, 3), (4, 3)],
    )));
    *map.slot_mut(S1).unwrap().layouts.get_mut(Faction::Gdo) = vec![
        laid(ObjectKind::DeploymentCenter, 2, 6, UNTURNED),
        laid(ObjectKind::ExtractionFacility, 7, 6, UNTURNED),
        laid(ObjectKind::ExtractionPlate, 3, 3, UNTURNED),
        laid(ObjectKind::ExtractionPlate, 4, 3, UNTURNED),
    ];
    for y in 12..=20 {
        map.terrain.set_tile(TilePos::new(2, y), Tile::new(TilePresetId::WATER));
    }
    map.regenerate();
    map
}

fn syndicate(app: &App, slot: SlotId) -> Vec<LayoutObject> {
    doc(app).slot(slot).unwrap().layouts.get(Faction::Syndicate).to_vec()
}

/// The Layout tool's ghost at `pos`: whether it is red as a whole, and what a click there would say.
fn ghost_at(app: &mut App, pos: Vec2) -> (bool, String) {
    act(app, [A::Hover(Some(pos))]);
    app.update();
    let preview = app.world().resource::<LayoutPreview>();
    let ghost = preview.0.as_ref().expect("a ghost under the cursor");
    match &ghost.verdict {
        Ok(()) => (false, String::new()),
        Err(refusal) => (refusal.marked_tiles().1, refusal.message(ghost.kind)),
    }
}

/// Slice M4-B's acceptance in the editor: "The editor's Syndicate palette offers the Tunnel, the Hideout and the
/// Agent", and "A starting Tunnel whose area breaches shows a red ghost with that text". On a map with every other
/// layout done, the Problems list says what the Syndicate's lacks; a Tunnel whose Tier 1 area takes in the lake, or
/// lies 1 tile from the map's edge, shows a red ghost and is refused with the rule's text (0242); 2 tiles from the edge
/// is fine (design change 0283: "a footprint 2 tiles from the edge is valid, and 1 tile is red"). A Hideout outside the
/// Tunnel's area is refused too. Placed where they may go, the layout is complete, slot 2's is the image, and the map
/// plays: slot 2's Syndicate player starts with it, at Tunnel Space 1/11.
#[test]
fn the_syndicate_layout_is_placed_from_their_starting_conditions() {
    let dir = temp_maps("syndicate_layout");
    let mut app = editor(syndicate_test_map(), &dir);
    assert_eq!(
        texts_under::<ProblemList>(&mut app),
        ["S1 Syndicate layout: add Tunnel, Hideout, Agent (Layout tool, T); S2 follows"]
    );

    act(&mut app, [A::SetLayoutFaction(Faction::Syndicate)]);
    let entry = |kind, left| PaletteEntry { kind, left: Some(left) };
    assert_eq!(
        palette(doc(&app), S1, Faction::Syndicate),
        [entry(TUNNEL, 1), entry(HIDEOUT, 1), entry(AGENT, 1)]
    );
    let panel = texts_under::<LayoutPanel>(&mut app);
    for expected in ["Tunnel: 1 left", "Hideout: 1 left", "Agent: 1 left"] {
        assert!(panel.iter().any(|t| t == expected), "{expected:?} not in {panel:?}");
    }
    assert_eq!(tools(&app).layout_kind, TUNNEL);

    // Against the lake: the Tunnel on (4, 14) to (7, 17) has its area from x = 2.
    let lake = "Can't place the Tunnel there: the Tunnel Area takes in undrillable ground: a Tunnel's whole Tunnel \
                Area must be Drillable (no Water or Mountain)";
    assert_eq!(ghost_at(&mut app, spot(4, 14, 4, 4)), (true, lake.to_string()));
    click(&mut app, spot(4, 14, 4, 4));
    assert_warned(&app, lake);
    // 1 tile from the south edge: the Tunnel on (6, 19) to (9, 22), its area down to y = 24, off the map.
    let edge = "Can't place the Tunnel there: the Tunnel Area reaches off the map: a Tunnel's whole Tunnel Area must \
                be on the map";
    assert_eq!(ghost_at(&mut app, spot(6, 19, 4, 4)), (true, edge.to_string()));
    click(&mut app, spot(6, 19, 4, 4));
    assert_warned(&app, edge);
    assert_eq!(syndicate(&app, S1), []);
    // 2 tiles from it, the area just reaches the edge: valid (design change 0283).
    assert_eq!(ghost_at(&mut app, spot(6, 18, 4, 4)), (false, String::new()));

    click(&mut app, spot(6, 14, 4, 4));
    assert_eq!(tools(&app).layout_kind, HIDEOUT, "on to the next kind the layout lacks");
    // The Hideout on (12, 14) to (13, 15), east of the area, which ends with x = 11.
    let outside = "Can't place the Hideout there: not wholly inside the Tier 1 Tunnel Area of a Tunnel of the layout";
    assert_eq!(ghost_at(&mut app, spot(12, 14, 2, 2)), (true, outside.to_string()));
    click(&mut app, spot(12, 14, 2, 2));
    assert_warned(&app, outside);
    click(&mut app, spot(10, 18, 2, 2));
    assert_eq!(tools(&app).layout_kind, AGENT);
    click(&mut app, point(AGENT_AT.0, AGENT_AT.1));
    assert_eq!(syndicate(&app, S1), syndicate_base());
    assert_eq!(
        palette(doc(&app), S1, Faction::Syndicate),
        [entry(TUNNEL, 0), entry(HIDEOUT, 0), entry(AGENT, 0)]
    );
    // Slot 2's is the image: the footprints through the map's center, turned half around, and the Agent's point.
    let half = BuildingOrientation {
        rotation: Rotation::R180,
        ..UNTURNED
    };
    let image = vec![
        laid(TUNNEL, 22, 6, half),
        laid(HIDEOUT, 20, 4, half),
        LayoutObject {
            kind: AGENT,
            at: SpawnAt::Point(point(32.0 - AGENT_AT.0, 24.0 - AGENT_AT.1)),
            orientation: UNTURNED,
        },
    ];
    assert_eq!(syndicate(&app, S2), image);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(panel.iter().any(|t| t == "S1 Syndicate layout: complete"), "{panel:?}");
    assert_eq!(texts_under::<ProblemList>(&mut app), ["None: the map is playable"]);
    assert_eq!(doc(&app).outdated_layouts(), []);

    act(&mut app, [A::SetPlayTestFaction(S2, Faction::Syndicate), A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    let p2 = PlayerId(2);
    let base: Vec<ObjectKind> = sim(&app)
        .objects()
        .filter(|o| o.owner == Some(p2))
        .map(|o| o.kind)
        .collect();
    assert_eq!(base, [TUNNEL, HIDEOUT, AGENT]);
    let cap = ArmyCap {
        resource: ArmyResource::TunnelSpace,
        used: 1,
        available: 11,
    };
    assert_eq!(sim(&app).army_cap(p2), Some(cap));
    let _ = std::fs::remove_dir_all(&dir);
}
