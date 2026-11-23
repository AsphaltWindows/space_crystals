//! What a press, drag and release on the map do with the active tool. A gesture is one undo step: it opens an edit
//! on the press and closes it on the release. Positions are ground points: what the cursor picks
//! (`crate::coords::ground_under`).
//!
//! On a symmetric map the tools edit the editable part only (`core/map_editor.md > Symmetry > Editing a symmetric
//! map`): a press in the locked part is refused with a status message, a brush stroke changes the tiles and corners
//! of the editable part it covers, and dragged objects and markers stay in the editable part. The editor generates
//! the locked part after every action ([`super::doc::EditorDoc::regenerate`]).

use std::collections::BTreeSet;

use bevy::prelude::*;
use sc_sim::{Category, CornerPos, SU_PER_GU, SlotId, Spawn, SpawnAt, TilePos};

use super::actions::Editor;
use super::objects;
use super::state::Selection;
use super::terrain;
use super::tools::{ElevationMode, Tool, symmetry_label};

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
    /// Dragging a slot's start position or camera start.
    MoveMarker(Selection),
}

impl Editor<'_> {
    pub(super) fn press(&mut self, pos: Vec2) {
        if !pos.is_finite() {
            return;
        }
        self.finish_gesture();
        self.cursor.0 = Some(pos);
        let tool = self.tools.tool;
        let region = self.doc.map().region();
        let tile = TilePos::containing(pos);
        let locked = match tool {
            Tool::Paint | Tool::Elevation | Tool::Fill | Tool::Start => !region.tile(tile),
            Tool::CameraStart => !region.point(pos),
            // Placements and selections check what they touch.
            Tool::Place | Tool::Select => false,
        };
        if locked {
            self.refuse_locked("That tile");
            return;
        }
        match tool {
            Tool::Select => self.select_at(pos),
            Tool::Paint | Tool::Elevation => {
                self.doc.begin_edit();
                self.gesture.0 = Some(GestureKind::Stroke {
                    last: tile,
                    touched: BTreeSet::new(),
                    refused: BTreeSet::new(),
                });
                self.dab(tile);
            }
            Tool::Fill => {
                let preset = self.tools.preset;
                if !self.doc.map().terrain.in_bounds(tile) {
                    return;
                }
                let filled = self.doc.edit(|m| terrain::fill(m, tile, preset));
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
            Tool::Start | Tool::CameraStart => {
                let slot = self.tools.slot;
                if !self.slot_editable(slot) {
                    return;
                }
                let marker = if tool == Tool::Start {
                    Selection::Start(slot)
                } else {
                    Selection::CameraStart(slot)
                };
                self.doc.begin_edit();
                self.gesture.0 = Some(GestureKind::MoveMarker(marker));
                self.move_marker(marker, pos);
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
        if self.doc.edit_open() {
            self.doc.end_edit();
        }
    }

    /// `core/map.md > Elevation > Maximum slope`: says that the preset wasn't painted on `count` tiles.
    fn warn_refused(&mut self, count: usize) {
        let terrain = &self.doc.map().terrain;
        let name = terrain
            .presets()
            .get(self.tools.preset.0 as usize)
            .map_or("?", |p| p.name.as_str());
        let message = format!(
            "{name} is Traversible: left {count} tile(s) steeper than 1 level as they were (flatten them first)"
        );
        self.status.warn(message);
    }

    /// Says that `what` lies in the locked part of the symmetric map, which is generated.
    pub(super) fn refuse_locked(&mut self, what: &str) {
        let symmetry = symmetry_label(self.doc.map().symmetry);
        self.status.warn(format!(
            "Locked: {what} is generated by the map's symmetry ({symmetry}); edit the bright part"
        ));
    }

    /// Whether `slot`'s markers can be edited: the slot exists and is not the image of another slot. Says why not.
    pub(super) fn slot_editable(&mut self, slot: SlotId) -> bool {
        let map = self.doc.map();
        if map.slot(slot).is_none() {
            self.status.warn(format!("There is no slot {}", slot.0));
            return false;
        }
        if !map.is_slot_editable(slot) {
            let source = map.source_slot(slot);
            self.refuse_locked(&format!("Slot {} (the image of slot {})", slot.0, source.0));
            return false;
        }
        true
    }

    pub(super) fn cancel(&mut self) {
        if self.gesture.0.take().is_some() {
            self.doc.cancel_edit();
            self.status.info("Cancelled");
        } else if self.dialog.open {
            self.cancel_new_map();
        } else if self.ui.help {
            self.ui.help = false;
        } else if self.selection.0.is_some() {
            self.selection.0 = None;
        }
    }

    /// The eyedropper: the terrain tools take the preset of the tile at `pos` and the height there, rounded to a
    /// whole level. It works on the locked part too: it changes nothing.
    pub(super) fn pick(&mut self, pos: Vec2) {
        let terrain = &self.doc.map().terrain;
        let Some(tile) = pos.is_finite().then(|| terrain.tile(TilePos::containing(pos))).flatten() else {
            return;
        };
        let name = terrain.preset(tile.preset).name.clone();
        let level = terrain.height_at(pos).round() as u8;
        self.tools.preset = tile.preset;
        self.tools.elevation = level;
        self.status.info(format!("Picked {name}, height {level}"));
    }

    pub(super) fn delete_selected(&mut self) {
        self.finish_gesture();
        let Some(selection) = self.selection.0 else {
            self.status.info("Nothing selected");
            return;
        };
        match selection {
            Selection::Spawn(index) => {
                let Some(spawn) = self.doc.map().spawns.get(index).cloned() else {
                    return;
                };
                self.selection.0 = None;
                self.doc.edit(|m| m.spawns.remove(index));
                self.status.info(format!("Deleted {}", spawn.kind.def().name));
            }
            Selection::Start(slot) => self
                .status
                .warn(format!("Slot {} always has a start position: drag it instead", slot.0)),
            Selection::CameraStart(slot) => {
                self.selection.0 = None;
                self.set_camera_start(slot, None);
            }
        }
    }

    /// Select tool: grabs the marker or object under `pos` (markers first, they draw on top; editable slots' before
    /// generated ones), else clears the selection. What lies in the locked part can't be selected.
    fn select_at(&mut self, pos: Vec2) {
        let map = self.doc.map();
        let tile = TilePos::containing(pos);
        let grab = MARKER_GRAB_RADIUS_GU * SU_PER_GU;
        let mut slots: Vec<SlotId> = map.slot_ids().collect();
        slots.sort_by_key(|s| !map.is_slot_editable(*s));
        let marker = slots
            .iter()
            .find_map(|&id| {
                let slot = map.slot(id)?;
                slot.camera_start
                    .filter(|c| c.distance(pos) <= grab)
                    .map(|_| Selection::CameraStart(id))
            })
            .or_else(|| {
                let id = slots.iter().find(|&&id| map.slot(id).is_some_and(|s| s.start == tile))?;
                Some(Selection::Start(*id))
            });
        if let Some(marker) = marker {
            let (Selection::Start(slot) | Selection::CameraStart(slot)) = marker else {
                return;
            };
            if !self.slot_editable(slot) {
                return;
            }
            self.selection.0 = Some(marker);
            self.doc.begin_edit();
            self.gesture.0 = Some(GestureKind::MoveMarker(marker));
            return;
        }
        let map = self.doc.map();
        match objects::spawn_at_point(map, pos) {
            Some(index) if !map.is_spawn_editable(&map.spawns[index]) => {
                let name = map.spawns[index].kind.def().name;
                self.selection.0 = None;
                self.refuse_locked(&format!("This {name}"));
            }
            Some(index) => {
                let spawn = &map.spawns[index];
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

    /// One brush dab of the Paint or Height tool at `center`: on the brush's tiles in the editable part.
    fn dab(&mut self, center: TilePos) {
        let tools = &*self.tools;
        let (tool, preset, mode, level, brush) =
            (tools.tool, tools.preset, tools.elevation_mode, tools.elevation, tools.brush);
        if tool == Tool::Paint {
            let tiles = terrain::dab_tiles(self.doc.map(), center, brush);
            if tiles.is_empty() {
                return;
            }
            let painted = terrain::paint(&mut self.doc.map_mut().terrain, tiles, preset);
            if let Some(GestureKind::Stroke { refused, .. }) = &mut self.gesture.0 {
                refused.extend(painted.refused);
            }
            return;
        }
        let corners = terrain::dab_corners(self.doc.map(), center, brush);
        let Some(GestureKind::Stroke { touched, .. }) = &mut self.gesture.0 else {
            return;
        };
        let fresh: BTreeSet<CornerPos> = match mode {
            // Raise and lower change a corner once per stroke, however often the brush passes over it.
            ElevationMode::Raise | ElevationMode::Lower => corners.into_iter().filter(|c| touched.insert(*c)).collect(),
            ElevationMode::Set => corners,
        };
        if fresh.is_empty() {
            return;
        }
        terrain::raise_or_set(self.doc.map_mut(), &fresh, mode, level);
    }

    /// Places the chosen object type at `at` with its kind's default values; the editor generates its images.
    /// Returns whether the object was placed. `report` says why not in the status line (the press does, a drag
    /// skips bad spots silently).
    fn place(&mut self, at: SpawnAt, report: bool) -> bool {
        let kind = self.tools.kind;
        let owner = match kind.def().category {
            Category::Resource => None, // `core/map.md > Resource`: always neutral.
            _ => self.tools.owner,
        };
        let spawn = Spawn::new(kind, owner, at);
        if let Err(problem) = objects::check_placement(self.doc.map(), &spawn, None) {
            if report {
                self.status.warn(problem.message(kind));
            }
            return false;
        }
        self.doc.edit(|m| m.spawns.push(spawn));
        true
    }

    /// Moves spawn `index` so that its center comes as close to `center` as the grid and the map allow; a spot
    /// where it, or one of its images, wouldn't fit is skipped.
    fn move_spawn(&mut self, index: usize, center: Vec2) {
        let map = self.doc.map();
        let Some(spawn) = map.spawns.get(index) else {
            return;
        };
        let at = objects::dragged_spawn(&map.terrain, spawn.kind, center);
        let moved = Spawn {
            at,
            ..spawn.clone()
        };
        if spawn.at != at && objects::check_placement(map, &moved, Some(index)).is_ok() {
            self.doc.map_mut().spawns[index].at = at;
        }
    }

    /// Puts a slot's marker under the cursor, kept on the map. In the locked part of a symmetric map it stays
    /// where it was.
    fn move_marker(&mut self, marker: Selection, pos: Vec2) {
        let map = self.doc.map();
        let point = objects::clamp_to_map(&map.terrain, pos);
        let region = map.region();
        let (slot, start) = match marker {
            Selection::Start(slot) => (slot, true),
            Selection::CameraStart(slot) => (slot, false),
            Selection::Spawn(_) => return,
        };
        let Some(current) = map.slot(slot) else {
            return;
        };
        let tile = TilePos::containing(point);
        let (editable, unchanged) = if start {
            (region.tile(tile), current.start == tile)
        } else {
            (region.point(point), current.camera_start == Some(point))
        };
        if !editable || unchanged {
            return;
        }
        let Some(slot_mut) = self.doc.map_mut().slot_mut(slot) else {
            return;
        };
        if start {
            slot_mut.start = tile;
        } else {
            slot_mut.camera_start = Some(point);
        }
        self.selection.0 = Some(marker);
    }
}
