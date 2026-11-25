//! Starting Layouts in the map editor (slice M2-I; `core/map_editor.md > Starting Layouts`, design changes 0008,
//! 0009, 0012 and 0097): the Layout tool (T) edits the layout of the chosen faction at the chosen slot, turned as R,
//! Shift+R, F and Shift+F turn it; a Skirmish Default layout takes only its faction's Starting Conditions and says
//! what it lacks; every layout follows its faction's placement rules minus sight; symmetric images follow, turned
//! to match; a completed map saves, opens and play-tests.
//!
//! By topic: the Layout tool, its slot and faction, the objects it places, turns, moves and removes, and their
//! symmetric images (`tool`); the layouts and their faction's Starting Conditions, on a Skirmish Default map and
//! outdated (`starting_conditions`); the placement rules minus sight, plates onto uncovered patches, and layouts in
//! each other's way (`rules`).

mod rules;
mod starting_conditions;
mod tool;

use bevy::prelude::*;
use sc_sim::{
    BuildingOrientation, Faction, LayoutObject, MapDef, MapSettings, MapType, ObjectKind, PlayerId, Rotation,
    SU_PER_GU, ScenarioProblem, SlotId, Spawn, SpawnAt, Symmetry, TilePos, TilePresetId, scenarios,
};
use space_crystals::ClientMode;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::layouts::{PaletteEntry, palette};
use space_crystals::editor::orientation::Turn;
use space_crystals::editor::state::{Selection, StatusLevel};
use space_crystals::editor::tools::Tool;
use space_crystals::editor::ui::lists::ProblemList;
use space_crystals::editor::ui::starting::LayoutPanel;

use super::colonists::with_colonists;
use super::syndicate::with_syndicate;
use super::{
    act, assert_warned, at, created, doc, editor, patch, problems, selection, sim, status, tap, temp_maps, tools,
};

const DC: ObjectKind = ObjectKind::DeploymentCenter; // 4 × 4 tiles
const EF: ObjectKind = ObjectKind::ExtractionFacility; // 3 × 3
const PLATE: ObjectKind = ObjectKind::ExtractionPlate; // 1 × 1
const BARRACKS: ObjectKind = ObjectKind::Barracks; // 3 × 2
const PK: ObjectKind = ObjectKind::Peacekeeper;
const S1: SlotId = SlotId(1);
const S2: SlotId = SlotId(2);
const UNTURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R0,
    flip_h: false,
    flip_v: false,
};

fn turned(rotation: Rotation, flip_h: bool) -> BuildingOrientation {
    BuildingOrientation {
        rotation,
        flip_h,
        flip_v: false,
    }
}

/// A flat Skirmish Default map, as the creation dialog makes it.
pub(super) fn skirmish(width: u32, height: u32, slots: usize, symmetry: Symmetry) -> MapDef {
    MapSettings {
        map_type: MapType::SkirmishDefault,
        width,
        height,
        slots,
        symmetry,
        name: "test".into(),
        description: String::new(),
    }
    .create()
    .unwrap()
}

/// `map` with crystal patches on `tiles` and their images.
pub(super) fn with_patches(mut map: MapDef, tiles: &[(i32, i32)]) -> MapDef {
    map.spawns.extend(tiles.iter().map(|&(x, y)| patch(x, y)));
    map.regenerate();
    map
}

/// The point where a press puts a `w` × `h` footprint's top-left tile at (`x`, `y`): the footprint's center.
pub(super) fn spot(x: i32, y: i32, w: u32, h: u32) -> Vec2 {
    TilePos::new(x, y).corner() + Vec2::new(w as f32, h as f32) * SU_PER_GU / 2.0
}

pub(super) fn laid(kind: ObjectKind, x: i32, y: i32, orientation: BuildingOrientation) -> LayoutObject {
    LayoutObject {
        kind,
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation,
    }
}

fn gdo(app: &App, slot: SlotId) -> Vec<LayoutObject> {
    doc(app).slot(slot).unwrap().layouts.get(Faction::Gdo).to_vec()
}

/// A click of the Layout tool at `pos`.
pub(super) fn click(app: &mut App, pos: Vec2) {
    act(app, [A::Hover(Some(pos)), A::Press(pos), A::Release]);
}

/// The texts of the UI under the node marked `M`, in order.
pub(super) fn texts_under<M: Component>(app: &mut App) -> Vec<String> {
    app.update(); // The lists rebuild after the actions, their texts appear a frame later.
    let mut roots = app.world_mut().query_filtered::<Entity, With<M>>();
    let root = roots.single(app.world()).unwrap();
    let mut texts = Vec::new();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Some(text) = app.world().get::<Text>(entity) {
            texts.push(text.0.clone());
        }
        if let Some(children) = app.world().get::<Children>(entity) {
            stack.extend(children.iter().rev());
        }
    }
    texts
}

fn status_message(app: &App) -> (StatusLevel, String) {
    (status(app).level, status(app).message.clone())
}
