//! The selection rules (`systems/controls/selection_and_hud.md`): what a click on the map, a box, a click on a
//! portrait, a control group key and Tab do to a player's ControlState, under the Constraints (`> Selection >
//! Constraints`). The objects arrive as [`Selectables`]: the ones the player may select, as they are drawn.
//!
//! The add inputs (Shift-click, Shift + box and Ctrl+Shift-click; design changes 0049, 0052 and 0053) all add
//! through one rule, [`ControlState::add_to_selection`]: they never remove anything, and they add nothing that the
//! Constraints forbid.
//!
//! Where the design is silent, the rules below are placeholders, each marked so where it is made.

use bevy::math::{Rect, Vec2};
use sc_sim::ObjectId;

use super::model::{ControlState, Selection};
use super::pick::{Selectable, Selectables};
use crate::command_ui::InterfaceState;
use crate::input::{Bindings, ControlGroup, InputAction, Modifiers};

/// How a click or a box is meant: the roles its modifiers hold in the bindings table (`ARCHITECTURE.md > Client
/// architecture > Input`), so rebinding a role changes the clicks and the boxes too.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ClickRoles {
    /// [`InputAction::ToggleSelection`] (Shift), the modifier of the add inputs: on the map a click toggles an own
    /// unit, a box adds what it selects and, with `same_type`, a click adds the own units of a type; on a portrait
    /// it removes.
    pub toggle: bool,
    /// [`InputAction::SelectSameType`] (Ctrl): a click picks all own units of a type. A box ignores it.
    pub same_type: bool,
    /// [`InputAction::CenterOnObject`] (Alt), for portraits.
    pub center: bool,
}

impl ClickRoles {
    pub fn of(bindings: &Bindings, modifiers: Modifiers) -> Self {
        Self {
            toggle: bindings.holds(InputAction::ToggleSelection, modifiers),
            same_type: bindings.holds(InputAction::SelectSameType, modifiers),
            center: bindings.holds(InputAction::CenterOnObject, modifiers),
        }
    }
}

/// `> Selection > Constraints`: a selection that holds any object the player doesn't own holds that object alone;
/// one of own objects has no limit (but [`super::MAX_SELECTED`]).
pub fn fits_constraints(ids: &[ObjectId], objects: &Selectables) -> bool {
    ids.len() <= 1 || ids.iter().all(|&id| objects.get(id).is_some_and(Selectable::is_own))
}

/// The selection of these objects (those of them that may be selected), its first group active.
fn selection_of(ids: impl IntoIterator<Item = ObjectId>, objects: &Selectables) -> Selection {
    Selection::new(
        ids.into_iter()
            .filter_map(|id| objects.get(id).map(|o| (id, o.group_type()))),
    )
}

/// The view point at the centroid of where these objects are drawn: what the camera centers on for them.
pub fn drawn_centroid(ids: impl IntoIterator<Item = ObjectId>, objects: &Selectables) -> Option<Vec2> {
    let drawn: Vec<Vec2> = ids
        .into_iter()
        .filter_map(|id| objects.get(id).map(|o| o.drawn))
        .collect();
    (!drawn.is_empty()).then(|| drawn.iter().copied().sum::<Vec2>() / drawn.len() as f32)
}

impl ControlState {
    /// Replaces the selection; says whether that changed it.
    fn select(&mut self, selection: Selection) -> bool {
        let changed = self.selection != selection;
        self.selection = selection;
        changed
    }

    /// Whether every selected object is the player's own (an empty selection is).
    fn all_own(&self, objects: &Selectables) -> bool {
        self.selection
            .ids()
            .all(|id| objects.get(id).is_some_and(Selectable::is_own))
    }

    /// `> MapClickSelection` (design changes 0011, 0048, 0052 and 0053): a click at the view point `at` on the map,
    /// `double` for the second click of a double-click; `visible` is the world view (view space). On an own unit:
    /// - a plain click selects that unit alone;
    /// - Shift toggles it: out of the selection if it is in, else in, by [`ControlState::add_to_selection`];
    /// - Ctrl, or a double-click (the same as a Ctrl-click), selects all own units of its type on screen;
    /// - Ctrl + Shift adds them, by [`ControlState::add_to_selection`]. So does a double-click with Shift held, as a
    ///   double-click counts as a Ctrl-click (placeholder reading: the design doesn't give that combination).
    ///
    /// Any click on another object selects that object alone, so an enemy or a neutral object is always alone, also
    /// when the click is modified (0048). So is an own structure (placeholder: the design gives the modified clicks
    /// for own units only).
    ///
    /// On empty ground (`> BoxSelection > Empty clicks and boxes`, 0051 and 0052): a plain click clears the
    /// selection; a Shift-, Ctrl- or Ctrl+Shift-click changes nothing. Says whether the selection changed.
    pub fn click(&mut self, objects: &Selectables, at: Vec2, roles: ClickRoles, double: bool, visible: Rect) -> bool {
        let Some(target) = objects.at(at) else {
            return !roles.toggle && !roles.same_type && self.select(Selection::EMPTY);
        };
        if !target.is_own_unit() {
            return self.select(selection_of([target.id], objects));
        }
        if roles.same_type || double {
            let mut ids = objects.own_units_on_screen(target.kind, visible);
            ids.push(target.id); // Under the cursor, so on screen even if its box were cut off.
            return if roles.toggle {
                self.add_to_selection(ids, objects)
            } else {
                self.select(selection_of(ids, objects))
            };
        }
        if roles.toggle {
            return if self.selection.contains(target.id) {
                self.selection.remove(target.id)
            } else {
                self.add_to_selection([target.id], objects)
            };
        }
        self.select(selection_of([target.id], objects))
    }

    /// `> BoxSelection`: the box (view space) selects what its highest tier holds ([`Selectables::in_box`]). A plain
    /// box replaces the selection with that, and one that holds nothing clears it (`> Empty clicks and boxes`,
    /// design change 0051). A Shift + box adds it instead (`> Shift + box (add)`, 0049), by
    /// [`ControlState::add_to_selection`], so one that holds nothing changes nothing. Ctrl doesn't change what a box
    /// does (placeholder: the design gives boxes no Ctrl). Says whether the selection changed.
    pub fn select_box(&mut self, objects: &Selectables, rect: Rect, roles: ClickRoles) -> bool {
        let boxed = objects.in_box(rect);
        if roles.toggle {
            return self.add_to_selection(boxed, objects);
        }
        self.select(selection_of(boxed, objects))
    }

    /// The add inputs (`> BoxSelection > Shift + box (add)`, `> MapClickSelection`; design changes 0049, 0052 and
    /// 0053): adds the objects `ids` that the player may select to the selection, which keeps everything it holds and
    /// its ActiveGroup. Under the Constraints nothing is added when that would put a non-owned object together with
    /// others: neither another player's or a neutral object into a selection of own objects (0049), nor anything into
    /// the selection of one non-owned object, which stays selected (`> Adding to a non-owned selection`, 0053). A
    /// full selection ([`super::MAX_SELECTED`]) takes no more. Says whether the selection changed.
    pub fn add_to_selection(&mut self, ids: impl IntoIterator<Item = ObjectId>, objects: &Selectables) -> bool {
        let mut added: Vec<&Selectable> = ids
            .into_iter()
            .filter(|&id| !self.selection.contains(id))
            .filter_map(|id| objects.get(id))
            .collect();
        added.sort_by_key(|o| o.id);
        added.dedup_by_key(|o| o.id);
        let merged: Vec<ObjectId> = self.selection.ids().chain(added.iter().map(|o| o.id)).collect();
        if !fits_constraints(&merged, objects) {
            return false;
        }
        let mut changed = false;
        for object in added {
            changed |= self.selection.insert(object.id, object.group_type());
        }
        changed
    }

    /// `> SelectionPanel > Interactions`: a click on the portrait of the selected object `id`.
    /// - plain: only that unit stays selected;
    /// - Shift: it leaves the selection;
    /// - Ctrl: all selected units of its type, and only those, stay selected;
    /// - Ctrl + Shift: all units of its type leave the selection;
    /// - Alt: no change; returns the view point to center the camera on (where the unit is drawn). Alt wins over
    ///   the other modifiers (placeholder: the design doesn't combine them).
    ///
    /// Its type is the one it groups as: the unfinished buildings of a type count "as if [they were] another type"
    /// (`> SelectionGroup > Unfinished buildings form their own group`, design change 0185), so Ctrl keeps, and
    /// Ctrl + Shift removes, the finished or the unfinished ones of the type, as the portrait's own are (literal
    /// reading). Fewer than 2 objects left hide the panel, and none left is an empty selection (`> Edge cases`).
    pub fn click_portrait(&mut self, objects: &Selectables, id: ObjectId, roles: ClickRoles) -> Option<Vec2> {
        let ty = self.selection.type_of(id)?;
        if roles.center {
            return objects.get(id).map(|o| o.drawn);
        }
        match (roles.same_type, roles.toggle) {
            (true, true) => {
                self.selection.retain(|_, t| t != ty);
            }
            (true, false) => {
                let same: Vec<ObjectId> = self
                    .selection
                    .member_types()
                    .filter(|&(_, t)| t == ty)
                    .map(|(i, _)| i)
                    .collect();
                self.select(selection_of(same, objects));
            }
            (false, true) => {
                self.selection.remove(id);
            }
            (false, false) => {
                self.select(selection_of([id], objects));
            }
        }
        None
    }

    /// `> ControlGroups > Operations > Assign`: the selection replaces the group's contents (an empty selection
    /// empties it). Only own objects go into control groups (`> ControlGroups > Groups`, design change 0056): while
    /// another player's or a neutral object is selected, Assign changes nothing. Says whether the group took the
    /// selection.
    pub fn assign_group(&mut self, group: ControlGroup, objects: &Selectables) -> bool {
        if !self.all_own(objects) {
            return false;
        }
        self.groups.assign(group, self.selection.ids());
        true
    }

    /// `> Operations > Add`: merges the selection into the group, without duplicates. Only own objects go into
    /// control groups (design change 0056): while another player's or a neutral object is selected, Add changes
    /// nothing. Says whether the group took the selection.
    pub fn add_to_group(&mut self, group: ControlGroup, objects: &Selectables) -> bool {
        if !self.all_own(objects) {
            return false;
        }
        self.groups.add(group, self.selection.ids());
        true
    }

    /// `> Operations > Recall`: the group's objects replace the selection, its first group active. Destroyed objects
    /// have left the group already (`super::systems::sync_control_states`). A group holds own objects only (design
    /// change 0056), so its recall keeps the Constraints; one that wouldn't is refused all the same. A recall resets
    /// the ObjectInterfaceState to DefaultState, also one that recalls exactly the current selection
    /// (`> ObjectInterfaceState`, design change 0078: a recall refreshes the selection). Placeholder rule, the design
    /// is silent: recalling an empty group changes nothing, the ObjectInterfaceState included. Says whether the
    /// group had objects to recall.
    pub fn recall_group(&mut self, group: ControlGroup, objects: &Selectables) -> bool {
        let ids: Vec<ObjectId> = self
            .groups
            .get(group)
            .iter()
            .copied()
            .filter(|&id| objects.get(id).is_some())
            .collect();
        if ids.is_empty() || !fits_constraints(&ids, objects) {
            return false;
        }
        self.select(selection_of(ids, objects));
        self.interface = InterfaceState::DefaultState;
        true
    }

    /// `> GroupCycling`: Tab (`forward`) or Shift-Tab, in the order of the groups ([`Selection::groups`]), which is
    /// the SelectionPanel's too (design change 0055).
    pub fn cycle_groups(&mut self, forward: bool) -> bool {
        self.selection.cycle(forward)
    }
}

#[cfg(test)]
mod tests;
