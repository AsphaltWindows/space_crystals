//! The Colonists' Starting Layouts in the map editor (slice M3-C; `core/map_editor.md > Starting Layouts`,
//! `factions/colonists/economy.md > Starting Conditions`, plan decision T24): on a Skirmish Default map the Layout
//! tool's palette is their Starting Conditions, the Drones go where they are clicked, as units, and the map plays
//! once their layout is complete too.

use bevy::prelude::App;
use sc_sim::{
    ArmyCap, ArmyResource, BuildingOrientation, Faction, LayoutObject, MapDef, ObjectKind, PlayerId, Rotation,
    SU_PER_GU, SlotId, SpawnAt, Symmetry, Vec2,
};
use space_crystals::ClientMode;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::layouts::{PaletteEntry, palette};
use space_crystals::editor::ui::lists::ProblemList;
use space_crystals::editor::ui::starting::LayoutPanel;

use super::cults::with_cults;
use super::layouts::{click, laid, skirmish, spot, texts_under, with_patches};
use super::syndicate::with_syndicate;
use super::{act, doc, editor, sim, temp_maps, tools};

const S1: SlotId = SlotId(1);
const S2: SlotId = SlotId(2);
const NODE: ObjectKind = ObjectKind::EgressNode; // 2 × 2 tiles
const PORTAL: ObjectKind = ObjectKind::RoboPortal; // 4 × 4
const DRONE: ObjectKind = ObjectKind::Drone;
const UNTURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R0,
    flip_h: false,
    flip_v: false,
};

/// Where [`colonists_base`] puts its Drones, in grid units: south of the Egress Node, 1 grid unit apart.
const DRONES: [(f32, f32); 4] = [(10.5, 6.5), (11.5, 6.5), (10.5, 7.5), (11.5, 7.5)];

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

fn drone(x: f32, y: f32) -> LayoutObject {
    LayoutObject {
        kind: DRONE,
        at: SpawnAt::Point(point(x, y)),
        orientation: UNTURNED,
    }
}

/// A Colonists Starting Layout for slot 1 of the 32 × 24 maps of the layout tests, clear of their GDO bases: the
/// Egress Node on (10, 2) to (11, 3), the Robo Portal on (12, 2) to (15, 5), and the four [`DRONES`].
fn colonists_base() -> Vec<LayoutObject> {
    let mut base = vec![laid(NODE, 10, 2, UNTURNED), laid(PORTAL, 12, 2, UNTURNED)];
    base.extend(DRONES.map(|(x, y)| drone(x, y)));
    base
}

/// `map` with [`colonists_base`] at slot 1 and its images: for the tests of other layouts on a Skirmish Default map,
/// which can't be played without it (plan decision T24).
pub(super) fn with_colonists(mut map: MapDef) -> MapDef {
    *map.slot_mut(S1).unwrap().layouts.get_mut(Faction::Colonists) = colonists_base();
    map.regenerate();
    map
}

fn colonists(app: &App, slot: SlotId) -> Vec<LayoutObject> {
    doc(app).slot(slot).unwrap().layouts.get(Faction::Colonists).to_vec()
}

/// The plan's slice M3-C: "The Colonists layout palette comes from the table, and Drones are placed as units." On a
/// new Skirmish Default map (here with the GDO's, the Syndicate's and the Cults' layouts done) the Problems list says
/// what the Colonists' layout lacks; the palette offers the Egress Node, the Robo Portal and four Drones, and moves on
/// to the next kind as each is done; a Drone goes exactly where it is clicked, a point, not a tile. Slot 2's layout is
/// the image, and with the layout complete the map plays: slot 2's Colonists player starts with it, at Beacon
/// Capacity 4/10.
#[test]
fn the_colonists_layout_is_placed_from_their_starting_conditions() {
    let dir = temp_maps("colonists_layout");
    let mut map = with_cults(with_syndicate(with_patches(
        skirmish(32, 24, 2, Symmetry::Rotate180),
        &[(3, 3), (4, 3)],
    )));
    let gdo = [
        laid(ObjectKind::DeploymentCenter, 2, 6, UNTURNED),
        laid(ObjectKind::ExtractionFacility, 7, 6, UNTURNED),
        laid(ObjectKind::ExtractionPlate, 3, 3, UNTURNED),
        laid(ObjectKind::ExtractionPlate, 4, 3, UNTURNED),
    ];
    *map.slot_mut(S1).unwrap().layouts.get_mut(Faction::Gdo) = gdo.to_vec();
    map.regenerate();
    let mut app = editor(map, &dir);
    assert_eq!(
        texts_under::<ProblemList>(&mut app),
        ["S1 Colonists layout: add Egress Node, Robo Portal, Drone x4 (Layout tool, T); S2 follows"]
    );

    act(&mut app, [A::SetLayoutFaction(Faction::Colonists)]);
    let entry = |kind, left| PaletteEntry { kind, left: Some(left) };
    assert_eq!(
        palette(doc(&app), S1, Faction::Colonists),
        [entry(NODE, 1), entry(PORTAL, 1), entry(DRONE, 4)]
    );
    let panel = texts_under::<LayoutPanel>(&mut app);
    for expected in ["Egress Node: 1 left", "Robo Portal: 1 left", "Drone: 4 left"] {
        assert!(panel.iter().any(|t| t == expected), "{expected:?} not in {panel:?}");
    }
    assert_eq!(tools(&app).layout_kind, NODE);
    click(&mut app, spot(10, 2, 2, 2));
    assert_eq!(tools(&app).layout_kind, PORTAL, "on to the next kind the layout lacks");
    click(&mut app, spot(12, 2, 4, 4));
    assert_eq!(tools(&app).layout_kind, DRONE);
    for (x, y) in DRONES {
        click(&mut app, point(x, y));
    }
    assert_eq!(
        colonists(&app, S1),
        colonists_base(),
        "the Drones at the points clicked"
    );
    assert_eq!(
        palette(doc(&app), S1, Faction::Colonists),
        [entry(NODE, 0), entry(PORTAL, 0), entry(DRONE, 0)]
    );
    // Slot 2's is the image: the footprints through the map's center, turned half around, and the Drones' points.
    let half = BuildingOrientation {
        rotation: Rotation::R180,
        ..UNTURNED
    };
    let mut image = vec![laid(NODE, 20, 20, half), laid(PORTAL, 16, 18, half)];
    image.extend(DRONES.map(|(x, y)| drone(32.0 - x, 24.0 - y)));
    assert_eq!(colonists(&app, S2), image);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(panel.iter().any(|t| t == "S1 Colonists layout: complete"), "{panel:?}");
    assert_eq!(texts_under::<ProblemList>(&mut app), ["None: the map is playable"]);
    assert_eq!(doc(&app).outdated_layouts(), []);

    act(&mut app, [A::SetPlayTestFaction(S2, Faction::Colonists), A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    let p2 = PlayerId(2);
    let base: Vec<ObjectKind> = sim(&app)
        .objects()
        .filter(|o| o.owner == Some(p2))
        .map(|o| o.kind)
        .collect();
    assert_eq!(base, [NODE, PORTAL, DRONE, DRONE, DRONE, DRONE]);
    let cap = ArmyCap {
        resource: ArmyResource::BeaconCapacity,
        used: 4,
        available: 10,
    };
    assert_eq!(sim(&app).army_cap(p2), Some(cap));
    let _ = std::fs::remove_dir_all(&dir);
}
