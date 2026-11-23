//! The selection rules (`systems/controls/selection_and_hud.md`): what a click on the map, a box, a click on a
//! portrait, a control group key and Tab do to a player's ControlState, under the Constraints (`> Selection >
//! Constraints`). The objects arrive as [`Selectables`]: the ones the player may select, as they are drawn.
//!
//! Where the design is silent, the rules below are placeholders, each marked so where it is made.

use bevy::math::{Rect, Vec2};
use sc_sim::ObjectId;

use super::model::{ControlState, Selection};
use super::pick::{Selectable, Selectables};
use crate::input::{Bindings, ControlGroup, InputAction, Modifiers};

/// How a click is meant: the roles its modifiers hold in the bindings table (`ARCHITECTURE.md > Client architecture
/// > Input`), so rebinding a role changes the clicks too.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ClickRoles {
    /// [`InputAction::ToggleSelection`] (Shift).
    pub toggle: bool,
    /// [`InputAction::SelectSameType`] (Ctrl).
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
    Selection::new(ids.into_iter().filter_map(|id| objects.get(id).map(|o| (id, o.kind))))
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

    /// `> MapClickSelection` (design change 0011): a click at the view point `at` on the map, `double` for the second
    /// click of a double-click; `visible` is the world view (view space). On the object drawn there:
    /// - Ctrl or a double-click on an own unit selects all own units of its type on screen;
    /// - Shift on an own unit toggles it in the selection (when the selection holds another player's object, which
    ///   must stay alone, the unit replaces it instead);
    /// - any other click selects that object alone, so an enemy or neutral object is always alone. That includes
    ///   Shift- and Ctrl-clicks on objects other than own units, which the design leaves open (placeholder rule).
    ///
    /// On empty ground (placeholder rule, the design is silent): a plain click clears the selection, a Shift- or
    /// Ctrl-click changes nothing. Says whether the selection changed.
    pub fn click(&mut self, objects: &Selectables, at: Vec2, roles: ClickRoles, double: bool, visible: Rect) -> bool {
        let Some(target) = objects.at(at) else {
            return !roles.toggle && !roles.same_type && self.select(Selection::EMPTY);
        };
        if target.is_own_unit() && (roles.same_type || double) {
            let mut ids = objects.own_units_on_screen(target.kind, visible);
            ids.push(target.id); // Under the cursor, so on screen even if its box were cut off.
            return self.select(selection_of(ids, objects));
        }
        if target.is_own_unit() && roles.toggle && self.all_own(objects) {
            return if self.selection.contains(target.id) {
                self.selection.remove(target.id)
            } else {
                self.selection.insert(target.id, target.kind)
            };
        }
        self.select(selection_of([target.id], objects))
    }

    /// `> BoxSelection`: the box (view space) replaces the selection with what its highest tier selects
    /// ([`Selectables::in_box`]). Placeholder rules, the design is silent: a box that holds nothing clears the
    /// selection, and modifiers don't change what a box does.
    pub fn select_box(&mut self, objects: &Selectables, rect: Rect) -> bool {
        self.select(selection_of(objects.in_box(rect), objects))
    }

    /// `> SelectionPanel > Interactions`: a click on the portrait of the selected object `id`.
    /// - plain: only that unit stays selected;
    /// - Shift: it leaves the selection;
    /// - Ctrl: all selected units of its type, and only those, stay selected;
    /// - Ctrl + Shift: all units of its type leave the selection;
    /// - Alt: no change; returns the view point to center the camera on (where the unit is drawn). Alt wins over
    ///   the other modifiers (placeholder: the design doesn't combine them).
    ///
    /// Fewer than 2 objects left hide the panel, and none left is an empty selection (`> Edge cases`).
    pub fn click_portrait(&mut self, objects: &Selectables, id: ObjectId, roles: ClickRoles) -> Option<Vec2> {
        let kind = self.selection.kind_of(id)?;
        if roles.center {
            return objects.get(id).map(|o| o.drawn);
        }
        match (roles.same_type, roles.toggle) {
            (true, true) => {
                self.selection.retain(|_, k| k != kind);
            }
            (true, false) => {
                let same: Vec<ObjectId> = self
                    .selection
                    .members()
                    .filter(|&(_, k)| k == kind)
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
    /// empties it).
    pub fn assign_group(&mut self, group: ControlGroup) {
        let ids: Vec<ObjectId> = self.selection.ids().collect();
        self.groups.assign(group, ids);
    }

    /// `> Operations > Add`: merges the selection into the group, without duplicates. Placeholder rule, the design is
    /// silent: an Add that would put another player's object together with others in a group (whose recall would
    /// break the Constraints) changes nothing. Says whether the group took the selection.
    pub fn add_to_group(&mut self, group: ControlGroup, objects: &Selectables) -> bool {
        let mut merged: Vec<ObjectId> = self
            .groups
            .get(group)
            .iter()
            .copied()
            .chain(self.selection.ids())
            .collect();
        merged.sort();
        merged.dedup();
        if !fits_constraints(&merged, objects) {
            return false;
        }
        self.groups.assign(group, merged);
        true
    }

    /// `> Operations > Recall`: the group's objects replace the selection, its first group active. Destroyed objects
    /// have left the group already (`super::systems::sync_control_states`). Placeholder rule, the design is
    /// silent: recalling an empty group changes nothing. Says whether the group had objects to recall.
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
        true
    }

    /// `> GroupCycling`: Tab (`forward`) or Shift-Tab.
    pub fn cycle_groups(&mut self, forward: bool) -> bool {
        self.selection.cycle(forward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::pick::Relation;
    use sc_sim::ObjectKind;

    /// Units in a row along y = 100, 50 su apart: object `n` at x = 50·n.
    fn row(units: &[(u32, ObjectKind, Relation)]) -> Selectables {
        let unit = |&(n, kind, relation): &(u32, ObjectKind, Relation)| Selectable {
            id: ObjectId(n),
            kind,
            relation,
            drawn: at(n),
            rect: Rect::from_center_half_size(at(n), Vec2::splat(12.0)),
            depth: at(n).y,
        };
        Selectables::from_objects(units.iter().map(unit).collect())
    }

    const PK: ObjectKind = ObjectKind::Peacekeeper;
    const GUARD: ObjectKind = ObjectKind::Guard;

    fn at(n: u32) -> Vec2 {
        Vec2::new(50.0 * n as f32, 100.0)
    }

    fn ids(state: &ControlState) -> Vec<u32> {
        state.selection.ids().map(|i| i.0).collect()
    }

    const SHIFT: ClickRoles = ClickRoles {
        toggle: true,
        same_type: false,
        center: false,
    };
    const CTRL: ClickRoles = ClickRoles {
        toggle: false,
        same_type: true,
        center: false,
    };
    const PLAIN: ClickRoles = ClickRoles {
        toggle: false,
        same_type: false,
        center: false,
    };

    #[test]
    fn clicks_follow_the_map_click_rules_and_the_constraints() {
        use Relation::{Enemy, Neutral, Own};
        let objects = row(&[
            (1, PK, Own),
            (2, PK, Own),
            (3, GUARD, Own),
            (4, PK, Enemy),
            (5, PK, Own),
            (9, PK, Own),
        ]);
        // Unit 9 (x = 450) is off the world view.
        let visible = Rect::new(0.0, 0.0, 400.0, 300.0);
        let mut state = ControlState::default();
        assert!(state.click(&objects, at(1), PLAIN, false, visible));
        assert_eq!(ids(&state), [1]);
        assert!(state.click(&objects, at(3), SHIFT, false, visible));
        assert_eq!(ids(&state), [1, 3]);
        assert!(state.click(&objects, at(1), SHIFT, false, visible));
        assert_eq!(ids(&state), [3], "Shift toggles a selected unit off");
        // Ctrl and a double-click: every own Peacekeeper on screen, not the enemy's nor the one off screen.
        assert!(state.click(&objects, at(2), CTRL, false, visible));
        assert_eq!(ids(&state), [1, 2, 5]);
        state.click(&objects, at(3), PLAIN, false, visible);
        assert!(state.click(&objects, at(5), PLAIN, true, visible));
        assert_eq!(ids(&state), [1, 2, 5]);
        // An enemy is selected alone, also by Shift or Ctrl; a Shift-click on an own unit then replaces it.
        assert!(state.click(&objects, at(4), SHIFT, false, visible));
        assert_eq!(ids(&state), [4]);
        assert!(!state.click(&objects, at(4), CTRL, false, visible));
        assert!(state.click(&objects, at(2), SHIFT, false, visible));
        assert_eq!(ids(&state), [2]);
        // Empty ground: a plain click clears, a modified one keeps the selection.
        let nowhere = Vec2::new(25.0, 250.0);
        assert!(!state.click(&objects, nowhere, SHIFT, false, visible));
        assert!(!state.click(&objects, nowhere, CTRL, true, visible));
        assert_eq!(ids(&state), [2]);
        assert!(state.click(&objects, nowhere, PLAIN, false, visible));
        assert!(state.selection.is_empty());
        let neutral = row(&[(1, ObjectKind::CrystalPatch, Neutral)]);
        state.click(&neutral, at(1), CTRL, true, visible);
        assert_eq!(ids(&state), [1], "a resource is selected alone");
    }

    #[test]
    fn portrait_clicks_follow_the_panels_rules() {
        use Relation::Own;
        let objects = row(&[(1, PK, Own), (2, PK, Own), (3, GUARD, Own), (4, GUARD, Own)]);
        let all = |state: &mut ControlState| state.selection = selection_of([1, 2, 3, 4].map(ObjectId), &objects);
        let mut state = ControlState::default();
        all(&mut state);
        let alt = ClickRoles { center: true, ..SHIFT };
        assert_eq!(state.click_portrait(&objects, ObjectId(3), alt), Some(at(3)));
        assert_eq!(ids(&state), [1, 2, 3, 4], "Alt only centers");
        assert_eq!(state.click_portrait(&objects, ObjectId(3), SHIFT), None);
        assert_eq!(ids(&state), [1, 2, 4]);
        state.click_portrait(&objects, ObjectId(2), CTRL);
        assert_eq!(ids(&state), [1, 2]);
        all(&mut state);
        let both = ClickRoles { toggle: true, ..CTRL };
        state.click_portrait(&objects, ObjectId(4), both);
        assert_eq!(ids(&state), [1, 2]);
        state.click_portrait(&objects, ObjectId(1), PLAIN);
        assert_eq!(ids(&state), [1]);
        state.click_portrait(&objects, ObjectId(1), SHIFT);
        assert!(state.selection.is_empty(), "the last one removed empties the selection");
        assert_eq!(state.click_portrait(&objects, ObjectId(1), alt), None, "not selected");
    }

    #[test]
    fn control_groups_keep_the_constraints() {
        use Relation::{Enemy, Own};
        let objects = row(&[(1, PK, Own), (2, GUARD, Own), (3, PK, Enemy), (4, PK, Enemy)]);
        let one = ControlGroup::new(1).unwrap();
        let mut state = ControlState::default();
        assert!(!state.recall_group(one, &objects), "an empty group recalls nothing");
        state.selection = selection_of([ObjectId(1)], &objects);
        state.assign_group(one);
        state.selection = selection_of([ObjectId(2)], &objects);
        assert!(state.add_to_group(one, &objects));
        assert_eq!(state.groups.get(one), [ObjectId(1), ObjectId(2)]);
        // Another player's object can't join others in a group, but it may be a group by itself.
        state.selection = selection_of([ObjectId(3)], &objects);
        assert!(!state.add_to_group(one, &objects));
        assert_eq!(state.groups.get(one), [ObjectId(1), ObjectId(2)]);
        let two = ControlGroup::new(2).unwrap();
        state.assign_group(two);
        state.selection = selection_of([ObjectId(4)], &objects);
        assert!(!state.add_to_group(two, &objects));
        assert!(state.recall_group(one, &objects));
        assert_eq!(ids(&state), [1, 2]);
        assert!(state.recall_group(two, &objects));
        assert_eq!(ids(&state), [3]);
        assert_eq!(
            drawn_centroid([ObjectId(1), ObjectId(2)], &objects),
            Some(Vec2::new(75.0, 100.0))
        );
    }
}
