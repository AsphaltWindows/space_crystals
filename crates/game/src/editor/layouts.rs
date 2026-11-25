//! Starting Layout editing (slice M2-I; `core/map_editor.md > Starting Layouts`, design changes 0008, 0009, 0012
//! and 0097): the Layout tool (T) places, moves and deletes the objects of one layout, the one of the chosen faction
//! ([`EditorTools::faction`]) at the chosen slot ([`EditorTools::slot`], an editable one: on a symmetric map the
//! slots of the locked part are generated from it, building orientations turned to match, `core/map_editor.md >
//! Symmetry > Editing a symmetric map`).
//!
//! - **Placing from scratch**: layouts start empty (0009); a click places the chosen object type
//!   ([`EditorTools::layout_kind`]), turned as R, Shift+R, F and Shift+F turned it ([`super::orientation`]); a click
//!   on an object of the layout selects it and drags it; Delete removes it.
//! - **Validation** ([`check`]): an object goes only where its layout stays valid by the sim's rules, "that faction's
//!   normal in-game placement rules, minus visibility" (`core/map_editor.md > Layout validation`, 0097), so a refused
//!   spot shows red under the cursor and says why. No visibility checks, no sight warnings.
//! - **Skirmish Default restriction** (`core/map_editor.md > Skirmish Default restriction`, 0009, 0012): the palette
//!   offers only the faction's Starting Conditions, each with how many the layout may still take ([`palette`]), and
//!   what the layout still lacks is listed ([`StartingConditions::missing`]); the Problems list says what to add where
//!   ([`problems`]).
//! - **Outdated layouts** (`core/map_editor.md > Outdated layouts`): flagged as the sim derives them
//!   ([`MapDef::outdated_layouts`], plan decision T11), never fixed automatically.
//! - **Overlay editing** (`core/map_editor.md > Overlay editing`, 0008): while the Layout tool is on, the editor's
//!   preview gives every slot a player of the edited faction ([`preview_scenario`]), so the sim shows that faction's
//!   layouts; the other factions' layouts of the edited slot are drawn dimmed over them (`super::overlay`).
//! - **Factions not implemented yet** (plan decision T11): the GDO, the Colonists (slice M3-C) and the Syndicate
//!   (slice M4-B) have Starting Conditions so far, the Colonists' palette offering their Egress Node, Robo Portal and
//!   four Drones, placed as units at the point clicked, and the Syndicate's their Tunnel, Hideout and Agent
//!   (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`). A Syndicate layout's Tunnel goes only where its
//!   whole Tier 1 Tunnel Area is on the map and Drillable (design change 0242), and its Hideout only wholly inside that
//!   area: [`check`] refuses the rest with the rule's text. The Cults' layouts can be picked but take no objects
//!   ("not implemented yet").

pub mod check;
pub mod problems;

use bevy::prelude::*;
use sc_sim::{
    Category, Faction, GameSetup, LayoutObject, MapDef, MapType, ObjectKind, Scenario, SlotId, StartingConditions,
};

use super::actions::Editor;
use super::gesture::GestureKind;
use super::objects::{self, faction_name};
use super::orientation::{self, Turn};
use super::state::Selection;
use super::tools::{EditorTools, Tool};

/// The objects of the layout of `faction` at `slot`: none if the map has no such slot.
pub fn layout(map: &MapDef, slot: SlotId, faction: Faction) -> &[LayoutObject] {
    map.slot(slot).map_or(&[], |s| s.layouts.get(faction))
}

/// The ground rectangle a layout object covers: its footprint as turned, or a unit's silhouette.
pub fn object_rect(object: &LayoutObject) -> Rect {
    objects::spawn_rect(object.kind, object.at, object.orientation)
}

/// The object of the layout of `faction` at `slot` drawn at `pos`: of those whose area contains it, the southernmost
/// (southern objects draw in front), then the latest.
pub fn layout_object_at(map: &MapDef, slot: SlotId, faction: Faction, pos: Vec2) -> Option<usize> {
    layout(map, slot, faction)
        .iter()
        .enumerate()
        .map(|(i, o)| (i, object_rect(o)))
        .filter(|(_, r)| r.contains(pos))
        .max_by(|(i, a), (j, b)| a.center().y.total_cmp(&b.center().y).then(i.cmp(j)))
        .map(|(i, _)| i)
}

/// The slot whose layout of `faction` has an object drawn at `pos`, if any: what the Select tool points at there.
pub fn layout_slot_at(map: &MapDef, faction: Faction, pos: Vec2) -> Option<SlotId> {
    map.slot_ids()
        .find(|&slot| layout_object_at(map, slot, faction, pos).is_some())
}

/// One entry of the Layout tool's palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteEntry {
    pub kind: ObjectKind,
    /// On a Skirmish Default map: how many more objects of the kind the layout may take.
    pub left: Option<u32>,
}

impl PaletteEntry {
    /// The button's text: the type, and on a Skirmish Default map how many are left ("2 left", "done").
    pub fn label(&self) -> String {
        let name = self.kind.def().name;
        match self.left {
            None => name.to_string(),
            Some(0) => format!("{name}: done"),
            Some(n) => format!("{name}: {n} left"),
        }
    }
}

/// What the Layout tool offers for the layout of `faction` at `slot`. On a Skirmish Default map, only the faction's
/// Starting Conditions (`core/map_editor.md > Skirmish Default restriction`: "may contain only the objects in that
/// faction's standard Starting Conditions"), each with how many more the layout may take; on a Custom map, which has
/// "no such restriction", every type a player may own (map resources never are, `core/map.md > Resource`). Nothing for
/// a faction not implemented yet (plan decision T11).
pub fn palette(map: &MapDef, slot: SlotId, faction: Faction) -> Vec<PaletteEntry> {
    let Some(start) = StartingConditions::of(faction) else {
        return Vec::new();
    };
    if map.map_type == MapType::SkirmishDefault {
        let objects = layout(map, slot, faction);
        return start
            .objects
            .iter()
            .map(|&(kind, n)| {
                let held = objects.iter().filter(|o| o.kind == kind).count();
                let held = u32::try_from(held).unwrap_or(u32::MAX);
                PaletteEntry {
                    kind,
                    left: Some(n.saturating_sub(held)),
                }
            })
            .collect();
    }
    ObjectKind::ALL
        .iter()
        .copied()
        .filter(|k| k.def().category != Category::Resource)
        .map(|kind| PaletteEntry { kind, left: None })
        .collect()
}

/// `n` objects of `kind`: "Deployment Center", "Extraction Plate x2".
pub fn count_text(kind: ObjectKind, n: u32) -> String {
    match n {
        1 => kind.def().name.to_string(),
        n => format!("{} x{n}", kind.def().name),
    }
}

/// What a layout lacks, as the editor lists it: "Deployment Center, Extraction Plate x2" (slice M2-I's acceptance).
pub fn missing_text(missing: &[(ObjectKind, u32)]) -> String {
    let parts: Vec<String> = missing.iter().map(|&(kind, n)| count_text(kind, n)).collect();
    parts.join(", ")
}

/// A layout's name in messages: "S1 GDO layout".
pub fn layout_name(slot: SlotId, faction: Faction) -> String {
    format!("S{} {} layout", slot.0, faction_name(faction))
}

/// What the Starting layouts section says of the edited layout, and whether it is fine (complete and valid).
pub fn layout_state(
    map: &MapDef,
    problems: &[sc_sim::ScenarioProblem],
    slot: SlotId,
    faction: Faction,
) -> (String, bool) {
    let name = layout_name(slot, faction);
    let Some(start) = StartingConditions::of(faction) else {
        return (
            format!("{}: not implemented yet, its layouts stay empty", faction_name(faction)),
            true,
        );
    };
    let objects = layout(map, slot, faction);
    let broken = problems
        .iter()
        .filter(|p| (0..objects.len()).any(|i| check::concerns(p, slot, faction, i)))
        .count();
    let broken = match broken {
        0 => String::new(),
        n => format!("; {n} object problem(s), outlined in red"),
    };
    if map.map_type != MapType::SkirmishDefault {
        let fine = broken.is_empty();
        return (format!("{name}: {} object(s){broken}", objects.len()), fine);
    }
    let missing = start.missing(objects);
    if missing.is_empty() {
        let fine = broken.is_empty();
        return (format!("{name}: complete{broken}"), fine);
    }
    (format!("{name}: missing {}{broken}", missing_text(&missing)), false)
}

/// The layouts flagged as outdated (`core/map_editor.md > Outdated layouts`, [`MapDef::outdated_layouts`]), for the
/// Starting layouts section: "Outdated or unfinished: S1 GDO, S2 GDO (...)". The flag is derived (plan decision T11),
/// so a layout still being placed is flagged too, and on a Custom map, whose layouts may hold anything
/// (`core/map_editor.md > Skirmish Default restriction`: "Custom maps have no such restriction"), one made different on
/// purpose: there the line only says so. Nothing is fixed automatically.
pub fn outdated_text(map: &MapDef) -> Option<String> {
    let outdated = map.outdated_layouts();
    if outdated.is_empty() {
        return None;
    }
    let names: Vec<String> = outdated
        .iter()
        .map(|(slot, faction)| format!("S{} {}", slot.0, faction_name(*faction)))
        .collect();
    let names = names.join(", ");
    Some(match map.map_type {
        MapType::SkirmishDefault => format!(
            "Outdated or unfinished: {names} (not the faction's current Starting Conditions; nothing is fixed \
             automatically)"
        ),
        MapType::Custom => {
            format!("Not the Starting Conditions: {names} (fine on a Custom map; outdated if made for older ones)")
        }
    })
}

/// The scenario the editor's preview shows (`super::sync_preview`): the map with a player in every slot, all GDO
/// (the default setup), or, while the Layout tool edits `faction`'s layouts, all of that faction, so that the sim
/// shows the edited faction's layout at every start position (`core/map_editor.md > Overlay editing`).
pub fn preview_scenario(map: &MapDef, faction: Option<Faction>) -> Scenario {
    let mut scenario = Scenario::new(map.clone());
    if let Some(faction) = faction {
        let factions = vec![faction; map.slots.len()];
        let defaults = &scenario.setup;
        scenario.setup = GameSetup {
            config: defaults.config,
            ..GameSetup::per_slot(map, &factions, defaults.seed)
        };
    }
    scenario
}

/// The faction whose layouts the preview shows instead of the GDO's: the edited one, while the Layout tool is on.
pub fn previewed_faction(tools: &EditorTools) -> Option<Faction> {
    (tools.tool == Tool::Layout).then_some(tools.faction)
}

impl Editor<'_> {
    /// Chooses the faction whose layout the Layout tool edits, and the tool.
    pub(super) fn set_layout_faction(&mut self, faction: Faction) {
        self.finish_gesture();
        self.tools.faction = faction;
        self.tools.tool = Tool::Layout;
        let slot = self.tools.slot;
        let map = self.doc.map();
        if StartingConditions::of(faction).is_none() {
            self.status.info(format!(
                "{}: not implemented yet, its layouts stay empty",
                faction_name(faction)
            ));
            return;
        }
        let (state, _) = layout_state(map, &self.problems.0, slot, faction);
        self.status.info(format!("Editing the {state}"));
    }

    /// Chooses the object type the Layout tool places, and the tool. On a Skirmish Default map only the faction's
    /// Starting Conditions may be chosen; map resources never (`core/map.md > Resource`: they are never owned).
    pub(super) fn set_layout_kind(&mut self, kind: ObjectKind) {
        let faction = self.tools.faction;
        let map = self.doc.map();
        if kind.def().category == Category::Resource {
            self.status.warn(format!(
                "A {} is a map resource, never a player's: place it with the Place tool (O)",
                kind.def().name
            ));
            return;
        }
        let offered = palette(map, self.tools.slot, faction).iter().any(|e| e.kind == kind);
        if !offered && StartingConditions::of(faction).is_some() {
            let refusal = check::LayoutRefusal::NotStarting { kind, faction };
            self.status.warn(refusal.message(kind));
            return;
        }
        self.finish_gesture();
        self.tools.layout_kind = kind;
        self.tools.tool = Tool::Layout;
    }

    /// Turns the structure that the Place or the Layout tool places (R, Shift+R, F, Shift+F).
    pub(super) fn turn(&mut self, turn: Turn) {
        let Some(kind) = self.tools.placed_kind().filter(|k| orientation::turns(*k)) else {
            self.status
                .warn("Only structures turn: choose one to place first (Place or Layout tool)");
            return;
        };
        self.tools.orientation = turn.apply(self.tools.orientation);
        self.status.info(format!(
            "{}: {}",
            kind.def().name,
            orientation::describe(self.tools.orientation)
        ));
    }

    /// The Layout tool's press at `pos`: grabs the object of the edited layout drawn there, else places the chosen
    /// object type there if the layout takes it, and selects it.
    pub(super) fn press_layout(&mut self, pos: Vec2) {
        let (slot, faction) = (self.tools.slot, self.tools.faction);
        if !self.slot_editable(slot) {
            return;
        }
        let map = self.doc.map();
        if let Some(index) = layout_object_at(map, slot, faction, pos) {
            let center = object_rect(&layout(map, slot, faction)[index]).center();
            self.selection.0 = Some(Selection::LayoutObject { slot, faction, index });
            self.doc.begin_edit();
            self.gesture.0 = Some(GestureKind::MoveLayoutObject {
                slot,
                faction,
                index,
                offset: center - pos,
            });
            return;
        }
        let kind = self.tools.layout_kind;
        let orientation = self.tools.placed_orientation();
        let object = LayoutObject {
            kind,
            at: objects::snap_spawn(kind, orientation, pos),
            orientation,
        };
        let index = layout(map, slot, faction).len();
        if let Err(refusal) = check::check_object(map, slot, faction, index, &object) {
            match refusal {
                check::LayoutRefusal::Locked => self.refuse_locked(&format!("that spot for the {}", kind.def().name)),
                refusal => self.status.warn(refusal.message(kind)),
            }
            return;
        }
        self.doc.edit(|m| {
            if let Some(s) = m.slot_mut(slot) {
                s.layouts.get_mut(faction).push(object);
            }
        });
        self.selection.0 = Some(Selection::LayoutObject { slot, faction, index });
        // With the kind done, on to the next one the layout lacks.
        let entries = palette(self.doc.map(), slot, faction);
        let next = entries
            .iter()
            .any(|e| e.kind == kind && e.left == Some(0))
            .then(|| entries.iter().find(|e| e.left.is_some_and(|n| n > 0)))
            .flatten()
            .map(|e| e.kind);
        let progress = self.layout_progress(slot, faction);
        let next = match next {
            Some(next) => {
                self.tools.layout_kind = next;
                format!(" (next: {})", next.def().name)
            }
            None => String::new(),
        };
        self.status.info(format!(
            "Placed the {} in the {}{progress}{next}",
            kind.def().name,
            layout_name(slot, faction)
        ));
    }

    /// Moves object `index` of a layout so that its center comes as close to `center` as the grid and the map allow;
    /// a spot where the layout wouldn't take it is skipped.
    pub(super) fn move_layout_object(&mut self, slot: SlotId, faction: Faction, index: usize, center: Vec2) {
        let map = self.doc.map();
        let Some(object) = layout(map, slot, faction).get(index).cloned() else {
            return;
        };
        let at = objects::dragged_spawn(&map.terrain, object.kind, object.orientation, center);
        if at == object.at {
            return;
        }
        let moved = LayoutObject { at, ..object };
        if check::check_object(map, slot, faction, index, &moved).is_ok()
            && let Some(s) = self.doc.map_mut().slot_mut(slot)
            && let Some(o) = s.layouts.get_mut(faction).get_mut(index)
        {
            o.at = at;
        }
    }

    /// Deletes object `index` of a layout (Delete with it selected).
    pub(super) fn delete_layout_object(&mut self, slot: SlotId, faction: Faction, index: usize) {
        let Some(kind) = layout(self.doc.map(), slot, faction).get(index).map(|o| o.kind) else {
            return;
        };
        if !self.slot_editable(slot) {
            return;
        }
        self.selection.0 = None;
        self.doc.edit(|m| {
            if let Some(s) = m.slot_mut(slot) {
                s.layouts.get_mut(faction).remove(index);
            }
        });
        let progress = self.layout_progress(slot, faction);
        self.status.info(format!(
            "Deleted the {} from the {}{progress}",
            kind.def().name,
            layout_name(slot, faction)
        ));
    }

    /// What a Skirmish Default layout still lacks, for a status message: "; still missing: Extraction Plate x2", or
    /// "; complete". Nothing on a Custom map.
    fn layout_progress(&self, slot: SlotId, faction: Faction) -> String {
        let map = self.doc.map();
        let Some(start) = StartingConditions::of(faction).filter(|_| map.map_type == MapType::SkirmishDefault) else {
            return String::new();
        };
        let missing = start.missing(layout(map, slot, faction));
        if missing.is_empty() {
            "; complete".into()
        } else {
            format!("; still missing: {}", missing_text(&missing))
        }
    }

    /// After a change: drops a layout object selection that no longer exists or lies in the locked part, and on a
    /// Skirmish Default map puts the Layout tool on a type of the faction's Starting Conditions. Writes only what
    /// changes, so that the UI doesn't redraw for nothing.
    pub(super) fn keep_layout_references_valid(&mut self) {
        let map = self.doc.map();
        if let Some(Selection::LayoutObject { slot, faction, index }) = self.selection.0
            && !(map.is_slot_editable(slot) && index < layout(map, slot, faction).len())
        {
            self.selection.0 = None;
        }
        let (slot, faction, kind) = (self.tools.slot, self.tools.faction, self.tools.layout_kind);
        let entries = palette(map, slot, faction);
        if !entries.is_empty() && !entries.iter().any(|e| e.kind == kind) {
            let first = entries.iter().find(|e| e.left != Some(0)).unwrap_or(&entries[0]).kind;
            self.tools.layout_kind = first;
        }
    }

    /// Selects the problem's layout object, or for a layout that lacks objects chooses the first one it lacks, with
    /// the Layout tool on its slot and faction: what a click on a layout problem in the Problems list does.
    pub(super) fn focus_layout_problem(&mut self, problem: &sc_sim::ScenarioProblem) {
        use sc_sim::ScenarioProblem as P;
        let map = self.doc.map();
        let (slot, faction, object) = match *problem {
            P::InvalidLayoutObject {
                slot, faction, index, ..
            }
            | P::LayoutPlacement {
                slot, faction, index, ..
            }
            | P::LayoutExtra {
                slot, faction, index, ..
            }
            | P::LayoutOverlap {
                slot, faction, index, ..
            } => (slot, faction, Some(index)),
            P::LayoutMissing {
                slot, faction, kind, ..
            } => {
                // The layout of the slot's set that is edited: its images follow it.
                let source = map.source_slot(slot);
                if map.slot(source).is_none() {
                    return;
                }
                self.tools.slot = source;
                self.tools.faction = faction;
                self.tools.tool = Tool::Layout;
                if palette(map, source, faction).iter().any(|e| e.kind == kind) {
                    self.tools.layout_kind = kind;
                }
                return;
            }
            _ => return,
        };
        if map.slot(slot).is_none() || !map.is_slot_editable(slot) {
            return;
        }
        self.tools.slot = slot;
        self.tools.faction = faction;
        self.tools.tool = Tool::Layout;
        if let Some(index) = object.filter(|i| *i < layout(map, slot, faction).len()) {
            self.selection.0 = Some(Selection::LayoutObject { slot, faction, index });
        }
    }
}

#[cfg(test)]
mod tests;
