//! What a press, drag and release on the map do with the active tool. A gesture is one undo step: it opens an edit
//! on the press and closes it on the release. Positions are ground points: what the cursor picks
//! (`crate::coords::ground_under`).

use std::collections::BTreeSet;

use bevy::prelude::*;
use sc_sim::{Category, CornerPos, PlayerId, SU_PER_GU, Spawn, SpawnAt, TilePos};

use super::actions::Editor;
use super::objects;
use super::state::Selection;
use super::terrain;
use super::tools::{ElevationMode, Tool};

/// How close (in tiles) a click must be to a camera start marker to grab it.
const MARKER_GRAB_RADIUS_GU: f32 = 0.5;

/// The gesture in progress, from the press to the release.
#[derive(Resource, Default, Debug)]
pub struct Gesture(pub Option<GestureKind>);

#[derive(Clone, Debug, PartialEq)]
pub enum GestureKind {
    /// A Paint or Height stroke: the last dab's tile, the corners raised or lowered so far (each changes once per
    /// stroke), and the tiles too steep for the painted preset.
    Stroke {
        last: TilePos,
        touched: BTreeSet<CornerPos>,
        refused: BTreeSet<TilePos>,
    },
    /// Placing objects; dragging places more where they fit (a crystal field in one stroke).
    Place { last: SpawnAt },
    /// Dragging a spawn: its center sits at the cursor plus `offset`.
    MoveSpawn { index: usize, offset: Vec2 },
    /// Dragging a slot's start location or camera start.
    MoveMarker(Selection),
}

impl Editor<'_> {
    pub(super) fn press(&mut self, pos: Vec2) {
        if !pos.is_finite() {
            return;
        }
        self.finish_gesture();
        self.cursor.0 = Some(pos);
        match self.tools.tool {
            Tool::Select => self.select_at(pos),
            Tool::Paint | Tool::Elevation => {
                let tile = TilePos::containing(pos);
                self.doc.begin_edit();
                self.gesture.0 = Some(GestureKind::Stroke {
                    last: tile,
                    touched: BTreeSet::new(),
                    refused: BTreeSet::new(),
                });
                self.dab(tile);
            }
            Tool::Fill => {
                let (tile, preset, symmetry) = (TilePos::containing(pos), self.tools.preset, self.tools.symmetry);
                if !self.doc.scenario().map.in_bounds(tile) {
                    return;
                }
                let filled = self.doc.edit(|s| terrain::fill(&mut s.map, tile, preset, symmetry));
                if !filled.refused.is_empty() {
                    self.warn_refused(filled.refused.len());
                } else if !filled.changed {
                    self.status.info("Nothing to fill: the region already has that preset");
                }
            }
            Tool::Place => {
                self.doc.begin_edit();
                let at = objects::snap_spawn(self.tools.kind, pos);
                self.gesture.0 = Some(GestureKind::Place { last: at });
                self.place(at, true);
            }
            Tool::StartLocation => {
                let slot = self.tools.slot;
                self.doc.begin_edit();
                self.gesture.0 = Some(GestureKind::MoveMarker(Selection::StartLocation(slot)));
                self.move_marker(Selection::StartLocation(slot), pos);
            }
            Tool::CameraStart => {
                let slot = self.tools.slot;
                self.doc.begin_edit();
                self.gesture.0 = Some(GestureKind::MoveMarker(Selection::CameraStart(slot)));
                self.move_marker(Selection::CameraStart(slot), pos);
            }
        }
    }

    pub(super) fn drag(&mut self, pos: Vec2) {
        if !pos.is_finite() {
            return;
        }
        self.cursor.0 = Some(pos);
        let Some(gesture) = self.gesture.0.clone() else {
            return;
        };
        match gesture {
            GestureKind::Stroke { last, .. } => {
                let tile = TilePos::containing(pos);
                if tile == last {
                    return;
                }
                for t in terrain::line_tiles(last, tile).into_iter().skip(1) {
                    self.dab(t);
                }
                if let Some(GestureKind::Stroke { last, .. }) = &mut self.gesture.0 {
                    *last = tile;
                }
            }
            GestureKind::Place { last } => {
                let at = objects::snap_spawn(self.tools.kind, pos);
                if at != last && self.place(at, false) {
                    self.gesture.0 = Some(GestureKind::Place { last: at });
                }
            }
            GestureKind::MoveSpawn { index, offset } => self.move_spawn(index, pos + offset),
            GestureKind::MoveMarker(marker) => self.move_marker(marker, pos),
        }
    }

    /// Ends the gesture: its changes become one undo step.
    pub(super) fn finish_gesture(&mut self) {
        if let Some(GestureKind::Stroke { refused, .. }) = self.gesture.0.take()
            && !refused.is_empty()
        {
            self.warn_refused(refused.len());
        }
        self.doc.end_edit();
    }

    /// `core/map.md > Elevation > Maximum slope`: says that the preset wasn't painted on `count` tiles.
    fn warn_refused(&mut self, count: usize) {
        let map = &self.doc.scenario().map;
        let name = map
            .presets()
            .get(self.tools.preset.0 as usize)
            .map_or("?", |p| p.name.as_str());
        let message = format!(
            "{name} is Traversible: left {count} tile(s) steeper than 1 level as they were (flatten them first)"
        );
        self.status.warn(message);
    }

    pub(super) fn cancel(&mut self) {
        if self.gesture.0.take().is_some() {
            self.doc.cancel_edit();
            self.status.info("Cancelled");
        } else if self.ui.help {
            self.ui.help = false;
        } else {
            self.selection.0 = None;
        }
    }

    /// The eyedropper: the terrain tools take the preset of the tile at `pos` and the height there, rounded to a
    /// whole level.
    pub(super) fn pick(&mut self, pos: Vec2) {
        let map = &self.doc.scenario().map;
        let Some(tile) = pos.is_finite().then(|| map.tile(TilePos::containing(pos))).flatten() else {
            return;
        };
        let name = map.preset(tile.preset).name.clone();
        let level = map.height_at(pos).round() as u8;
        self.tools.preset = tile.preset;
        self.tools.elevation = level;
        self.status.info(format!("Picked {name}, height {level}"));
    }

    pub(super) fn delete_selected(&mut self) {
        self.finish_gesture();
        let Some(selection) = self.selection.0.take() else {
            self.status.info("Nothing selected");
            return;
        };
        match selection {
            Selection::Spawn(index) => {
                let Some(spawn) = self.doc.scenario().spawns.get(index).cloned() else {
                    return;
                };
                self.doc.edit(|s| s.spawns.remove(index));
                self.status.info(format!("Deleted {}", spawn.kind.def().name));
            }
            Selection::StartLocation(slot) => self.set_start_location(slot, None),
            Selection::CameraStart(slot) => self.set_camera_start(slot, None),
        }
    }

    /// Select tool: grabs the marker or object under `pos` (markers first, they draw on top), else clears the
    /// selection.
    fn select_at(&mut self, pos: Vec2) {
        let scenario = self.doc.scenario();
        let tile = TilePos::containing(pos);
        let grab = MARKER_GRAB_RADIUS_GU * SU_PER_GU;
        let marker = scenario
            .players
            .iter()
            .find_map(|p| {
                p.camera_start
                    .filter(|c| c.distance(pos) <= grab)
                    .map(|_| Selection::CameraStart(p.id))
            })
            .or_else(|| {
                let p = scenario.players.iter().find(|p| p.start_location == Some(tile))?;
                Some(Selection::StartLocation(p.id))
            });
        if let Some(marker) = marker {
            self.selection.0 = Some(marker);
            self.doc.begin_edit();
            self.gesture.0 = Some(GestureKind::MoveMarker(marker));
            return;
        }
        match objects::spawn_at_point(scenario, pos) {
            Some(index) => {
                let spawn = &scenario.spawns[index];
                let center = objects::spawn_rect(spawn.kind, spawn.at).center();
                self.selection.0 = Some(Selection::Spawn(index));
                self.doc.begin_edit();
                self.gesture.0 = Some(GestureKind::MoveSpawn {
                    index,
                    offset: center - pos,
                });
            }
            None => self.selection.0 = None,
        }
    }

    /// One brush dab of the Paint or Height tool at `center`, repeated at its symmetry images.
    fn dab(&mut self, center: TilePos) {
        let tools = &*self.tools;
        let (tool, preset, mode, level) = (tools.tool, tools.preset, tools.elevation_mode, tools.elevation);
        let (brush, symmetry) = (tools.brush, tools.symmetry);
        if tool == Tool::Paint {
            let tiles = terrain::dab_tiles(&self.doc.scenario().map, center, brush, symmetry);
            let painted = terrain::paint(&mut self.doc.scenario_mut().map, tiles, preset);
            if let Some(GestureKind::Stroke { refused, .. }) = &mut self.gesture.0 {
                refused.extend(painted.refused);
            }
            return;
        }
        let corners = terrain::dab_corners(&self.doc.scenario().map, center, brush, symmetry);
        let Some(GestureKind::Stroke { touched, .. }) = &mut self.gesture.0 else {
            return;
        };
        let fresh: Vec<CornerPos> = match mode {
            // Raise and lower change a corner once per stroke, however often the brush passes over it.
            ElevationMode::Raise | ElevationMode::Lower => corners.into_iter().filter(|c| touched.insert(*c)).collect(),
            ElevationMode::Set => corners.into_iter().collect(),
        };
        if fresh.is_empty() {
            return;
        }
        let targets = terrain::height_targets(&self.doc.scenario().map, fresh, mode, level);
        terrain::shape(&mut self.doc.scenario_mut().map, &targets);
    }

    /// Places the chosen object type at `at` with its kind's default values, and copies at its symmetry images
    /// where they fit. Returns whether the object was placed. `report` says why not in the status line (the press
    /// does, a drag skips bad spots silently).
    fn place(&mut self, at: SpawnAt, report: bool) -> bool {
        let kind = self.tools.kind;
        let owner = match kind.def().category {
            Category::Resource => None, // `core/map.md > Resource`: always neutral.
            _ => self.tools.owner,
        };
        if let Err(problem) = objects::check_placement(self.doc.scenario(), kind, at, None) {
            if report {
                self.status.warn(problem.message(kind));
            }
            return false;
        }
        let players = self.doc.scenario().players.len();
        let symmetry = self.tools.symmetry;
        self.doc.edit(|s| {
            let spawn = Spawn::new(kind, owner, at);
            let images = objects::spawn_images(&s.map, kind, at, symmetry);
            let count = terrain::transforms(&s.map, symmetry).len();
            s.spawns.push(spawn.clone());
            for (k, image) in images {
                if objects::check_placement(s, kind, image, None).is_ok() {
                    s.spawns.push(Spawn {
                        owner: objects::image_owner(owner, k, count, players),
                        at: image,
                        ..spawn.clone()
                    });
                }
            }
        });
        true
    }

    /// Moves spawn `index` so that its center comes as close to `center` as the grid and the map allow; a spot
    /// where it would overlap another footprint is skipped.
    fn move_spawn(&mut self, index: usize, center: Vec2) {
        let Some(kind) = self.doc.scenario().spawns.get(index).map(|s| s.kind) else {
            return;
        };
        let at = objects::dragged_spawn(&self.doc.scenario().map, kind, center);
        if self.doc.scenario().spawns[index].at != at
            && objects::check_placement(self.doc.scenario(), kind, at, Some(index)).is_ok()
        {
            self.doc.scenario_mut().spawns[index].at = at;
        }
    }

    /// Puts a slot's marker under the cursor (kept on the map).
    fn move_marker(&mut self, marker: Selection, pos: Vec2) {
        let point = objects::clamp_to_map(&self.doc.scenario().map, pos);
        let (slot, tile, camera) = match marker {
            Selection::StartLocation(slot) => (slot, Some(TilePos::containing(point)), None),
            Selection::CameraStart(slot) => (slot, None, Some(point)),
            Selection::Spawn(_) => return,
        };
        let Some(setup) = self.doc.scenario().players.iter().find(|p| p.id == slot) else {
            self.status.warn(format!("there is no player {}", slot.0));
            return;
        };
        let unchanged = match marker {
            Selection::StartLocation(_) => setup.start_location == tile,
            _ => setup.camera_start == camera,
        };
        if unchanged {
            return;
        }
        let setup = self.slot_mut(slot);
        match marker {
            Selection::StartLocation(_) => setup.start_location = tile,
            _ => setup.camera_start = camera,
        }
        self.selection.0 = Some(marker);
    }

    fn slot_mut(&mut self, slot: PlayerId) -> &mut sc_sim::PlayerSetup {
        let players = &mut self.doc.scenario_mut().players;
        let index = players
            .iter()
            .position(|p| p.id == slot)
            .expect("the caller checked the slot");
        &mut players[index]
    }
}
