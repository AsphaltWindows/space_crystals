//! The selection model (`systems/controls/selection_and_hud.md > Selection`, `> SelectionGroup`, `> ControlGroups`):
//! plain data without Bevy systems. Which objects may be selected together (`> Selection > Constraints`) is the rules'
//! business (`super::rules`), because it depends on who owns them.

use std::collections::BTreeMap;

use sc_sim::{ObjectId, ObjectKind};

use crate::command_ui::InterfaceState;
use crate::input::ControlGroup;

/// The most objects a selection or a control group holds. Placeholder, unspecified: `> Selection > Constraints`
/// sets no limit on a selection of own objects; this is the sim's limit for one order
/// (`sc_sim::orders::MAX_ORDER_UNITS`), so that one command can always carry the whole selection.
pub const MAX_SELECTED: usize = sc_sim::orders::MAX_ORDER_UNITS;

/// Names a SelectionGroup (`> SelectionGroup`): its type and, for an Ungroupable type (`Groupable = false`), its one
/// instance, since each instance of such a type is a group of its own. The design types ActiveGroup as an
/// `ObjectEnum`, which can't tell an Ungroupable type's groups apart, so the client names groups by this key.
/// Groups are ordered by it: by type (`ObjectKind` order), then by instance.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupKey {
    pub kind: ObjectKind,
    /// The instance of an Ungroupable type's group; `None` for a Groupable type, whose instances share one group.
    pub instance: Option<ObjectId>,
}

impl GroupKey {
    /// The group that object `id` of type `kind` belongs to.
    pub fn of(id: ObjectId, kind: ObjectKind) -> Self {
        Self {
            kind,
            instance: (!kind.def().groupable).then_some(id),
        }
    }
}

/// `> SelectionGroup`: selected instances of one type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionGroup {
    pub key: GroupKey,
    /// In id order.
    pub instances: Vec<ObjectId>,
}

impl SelectionGroup {
    pub fn kind(&self) -> ObjectKind {
        self.key.kind
    }
}

/// `> Selection`: the selected objects, grouped by type ([`Selection::groups`]), and the ActiveGroup, which always
/// names one of the groups while the selection isn't empty (`> Selection > Constraints`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    members: BTreeMap<ObjectId, ObjectKind>,
    active: Option<GroupKey>,
}

impl Selection {
    pub const EMPTY: Selection = Selection {
        members: BTreeMap::new(),
        active: None,
    };

    /// A selection of these objects (at most [`MAX_SELECTED`], the first by id), whose first group is active.
    pub fn new(objects: impl IntoIterator<Item = (ObjectId, ObjectKind)>) -> Self {
        let mut members: BTreeMap<ObjectId, ObjectKind> = objects.into_iter().collect();
        while members.len() > MAX_SELECTED {
            members.pop_last();
        }
        let mut selection = Self { members, active: None };
        selection.fix_active();
        selection
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn contains(&self, id: ObjectId) -> bool {
        self.members.contains_key(&id)
    }

    /// The type of the selected object `id`.
    pub fn kind_of(&self, id: ObjectId) -> Option<ObjectKind> {
        self.members.get(&id).copied()
    }

    /// The selected objects with their types, in id order.
    pub fn members(&self) -> impl Iterator<Item = (ObjectId, ObjectKind)> + '_ {
        self.members.iter().map(|(&id, &kind)| (id, kind))
    }

    /// The selected objects, in id order.
    pub fn ids(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.members.keys().copied()
    }

    /// The SelectionGroups, in [`GroupKey`] order.
    pub fn groups(&self) -> Vec<SelectionGroup> {
        let mut keyed: Vec<(GroupKey, ObjectId)> =
            self.members().map(|(id, kind)| (GroupKey::of(id, kind), id)).collect();
        keyed.sort();
        let mut groups: Vec<SelectionGroup> = Vec::new();
        for (key, id) in keyed {
            match groups.last_mut() {
                Some(group) if group.key == key => group.instances.push(id),
                _ => groups.push(SelectionGroup {
                    key,
                    instances: vec![id],
                }),
            }
        }
        groups
    }

    /// The ActiveGroup; `None` exactly when nothing is selected.
    pub fn active(&self) -> Option<GroupKey> {
        self.active
    }

    /// The members of the ActiveGroup.
    pub fn active_group(&self) -> Option<SelectionGroup> {
        let active = self.active?;
        self.groups().into_iter().find(|g| g.key == active)
    }

    /// Whether `id` belongs to the ActiveGroup.
    pub fn in_active_group(&self, id: ObjectId) -> bool {
        self.members
            .get(&id)
            .is_some_and(|&kind| Some(GroupKey::of(id, kind)) == self.active)
    }

    /// Adds an object, unless it is selected already or the selection is full. The ActiveGroup stays.
    pub fn insert(&mut self, id: ObjectId, kind: ObjectKind) -> bool {
        if self.members.contains_key(&id) || self.members.len() >= MAX_SELECTED {
            return false;
        }
        self.members.insert(id, kind);
        self.fix_active();
        true
    }

    /// Removes an object. The ActiveGroup stays if it still has members, else the first group becomes active.
    pub fn remove(&mut self, id: ObjectId) -> bool {
        let removed = self.members.remove(&id).is_some();
        if removed {
            self.fix_active();
        }
        removed
    }

    /// Keeps only the objects for which `keep` holds; the ActiveGroup as in [`Selection::remove`]. Says whether any
    /// object was dropped.
    pub fn retain(&mut self, mut keep: impl FnMut(ObjectId, ObjectKind) -> bool) -> bool {
        let before = self.members.len();
        self.members.retain(|&id, &mut kind| keep(id, kind));
        let changed = self.members.len() != before;
        if changed {
            self.fix_active();
        }
        changed
    }

    /// `> GroupCycling`: makes the next group (`forward`) or the previous one active, wrapping around at the ends.
    pub fn cycle(&mut self, forward: bool) -> bool {
        let groups = self.groups();
        let Some(current) = groups.iter().position(|g| Some(g.key) == self.active) else {
            return false;
        };
        let n = groups.len();
        let next = if forward {
            (current + 1) % n
        } else {
            (current + n - 1) % n
        };
        let changed = next != current;
        self.active = Some(groups[next].key);
        changed
    }

    /// `> Selection > Constraints`: ActiveGroup must name an existing group. A group that is gone gives way to the
    /// first one (placeholder: the design doesn't say which group becomes active then).
    fn fix_active(&mut self) {
        let exists = |key: GroupKey| self.members.iter().any(|(&id, &kind)| GroupKey::of(id, kind) == key);
        if self.active.is_none_or(|key| !exists(key)) {
            self.active = self.groups().first().map(|g| g.key);
        }
    }
}

/// `> ControlGroups`: 10 saved selections. An object may be in several groups; a group never holds an object twice.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControlGroups {
    /// Each group's objects, in id order.
    groups: [Vec<ObjectId>; ControlGroup::COUNT as usize],
}

impl ControlGroups {
    pub const EMPTY: ControlGroups = ControlGroups {
        groups: [const { Vec::new() }; ControlGroup::COUNT as usize],
    };

    /// The objects in `group`, in id order.
    pub fn get(&self, group: ControlGroup) -> &[ObjectId] {
        &self.groups[usize::from(group.index())]
    }

    /// `> Operations > Assign`: replaces the group's contents.
    pub fn assign(&mut self, group: ControlGroup, ids: impl IntoIterator<Item = ObjectId>) {
        let mut ids: Vec<ObjectId> = ids.into_iter().collect();
        ids.sort();
        ids.dedup();
        ids.truncate(MAX_SELECTED);
        self.groups[usize::from(group.index())] = ids;
    }

    /// `> Operations > Add`: merges `ids` into the group, without duplicates.
    pub fn add(&mut self, group: ControlGroup, ids: impl IntoIterator<Item = ObjectId>) {
        let merged: Vec<ObjectId> = self.get(group).iter().copied().chain(ids).collect();
        self.assign(group, merged);
    }

    /// Drops the objects for which `keep` fails from every group (`> ControlGroups`: destroyed entities are removed).
    pub fn retain(&mut self, mut keep: impl FnMut(ObjectId) -> bool) -> bool {
        let mut changed = false;
        for group in &mut self.groups {
            let before = group.len();
            group.retain(|&id| keep(id));
            changed |= group.len() != before;
        }
        changed
    }
}

/// `> ControlState`: a player's selection, control groups and ObjectInterfaceState, client-side UI state, never part
/// of the sim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControlState {
    pub selection: Selection,
    pub groups: ControlGroups,
    /// `> ObjectInterfaceState[ActiveGroup]`: where the ActiveGroup's command flow stands (`crate::command_ui`).
    pub interface: InterfaceState,
}

impl ControlState {
    pub const EMPTY: ControlState = ControlState {
        selection: Selection::EMPTY,
        groups: ControlGroups::EMPTY,
        interface: InterfaceState::DefaultState,
    };

    /// Drops the objects for which `keep` fails from the selection and the control groups. Says whether anything
    /// changed. `> ControlState`: an ObjectInterfaceState the removal makes invalid resets to DefaultState; one of an
    /// ActiveGroup that is gone does (it was its type's, `> ObjectInterfaceState[ObjectEnum]`), and the command flow
    /// checks the rest against the game every frame.
    pub fn retain(&mut self, mut keep: impl FnMut(ObjectId) -> bool) -> bool {
        let active = self.selection.active();
        let selection = self.selection.retain(|id, _| keep(id));
        let groups = self.groups.retain(keep);
        let reset = self.selection.active() != active && self.interface != InterfaceState::DefaultState;
        if reset {
            self.interface = InterfaceState::DefaultState;
        }
        selection || groups || reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PK: ObjectKind = ObjectKind::Peacekeeper;
    const GUARD: ObjectKind = ObjectKind::Guard;
    /// Ungroupable (`core/map.md > SpaceCrystalsPatch`).
    const PATCH: ObjectKind = ObjectKind::CrystalPatch;

    fn id(n: u32) -> ObjectId {
        ObjectId(n)
    }

    fn group(n: u8) -> ControlGroup {
        ControlGroup::new(n).unwrap()
    }

    #[test]
    fn groups_collect_a_groupable_types_instances_and_split_an_ungroupable_ones() {
        let s = Selection::new([(id(7), GUARD), (id(2), PK), (id(5), PATCH), (id(3), PATCH), (id(4), PK)]);
        let groups = s.groups();
        let keys: Vec<GroupKey> = groups.iter().map(|g| g.key).collect();
        let key = |kind, instance| GroupKey { kind, instance };
        // In `ObjectKind` order: CrystalPatch, then Peacekeeper, then Guard; an ungroupable type's groups by id.
        assert_eq!(
            keys,
            [
                key(PATCH, Some(id(3))),
                key(PATCH, Some(id(5))),
                key(PK, None),
                key(GUARD, None)
            ]
        );
        assert_eq!(groups[2].instances, [id(2), id(4)]);
        assert_eq!(s.active(), Some(keys[0]), "a new selection's first group is active");
        assert!(s.in_active_group(id(3)) && !s.in_active_group(id(5)));
        assert_eq!(s.ids().collect::<Vec<_>>(), [id(2), id(3), id(4), id(5), id(7)]);
        assert_eq!(Selection::new([]).active(), None);
    }

    #[test]
    fn cycling_wraps_around_both_ways() {
        let mut s = Selection::new([(id(1), PK), (id(2), GUARD), (id(3), PATCH)]);
        let kinds = |s: &Selection| s.active().map(|k| k.kind);
        assert_eq!(kinds(&s), Some(PATCH));
        assert!(s.cycle(true));
        assert_eq!(kinds(&s), Some(PK));
        assert!(s.cycle(true));
        assert!(s.cycle(true));
        assert_eq!(kinds(&s), Some(PATCH), "wraps to the first after the last");
        assert!(s.cycle(false));
        assert_eq!(kinds(&s), Some(GUARD), "and to the last before the first");
        let mut single = Selection::new([(id(1), PK), (id(4), PK)]);
        assert!(!single.cycle(true), "one group: nothing to cycle");
        assert!(!Selection::EMPTY.clone().cycle(false));
    }

    #[test]
    fn the_active_group_stays_while_it_has_members() {
        let mut s = Selection::new([(id(1), PK), (id(2), GUARD)]);
        s.cycle(true);
        assert_eq!(s.active().unwrap().kind, GUARD);
        assert!(s.insert(id(3), PATCH));
        assert!(!s.insert(id(3), PATCH), "already selected");
        assert_eq!(s.active().unwrap().kind, GUARD, "adding keeps the active group");
        assert!(s.remove(id(1)));
        assert_eq!(s.active().unwrap().kind, GUARD);
        assert!(s.remove(id(2)));
        assert_eq!(
            s.active().unwrap().kind,
            PATCH,
            "a group that is gone gives way to the first"
        );
        assert!(s.retain(|_, _| false));
        assert!(s.is_empty() && s.active().is_none());
        assert!(!s.remove(id(9)));
    }

    #[test]
    fn selections_and_groups_hold_at_most_the_maximum() {
        let many = (1..=MAX_SELECTED as u32 + 5).map(|n| (id(n), PK));
        let mut s = Selection::new(many);
        assert_eq!(s.len(), MAX_SELECTED);
        assert!(s.contains(id(1)) && !s.contains(id(MAX_SELECTED as u32 + 1)));
        assert!(!s.insert(id(9999), PK));
        let mut groups = ControlGroups::default();
        groups.assign(group(1), (1..=MAX_SELECTED as u32 + 5).map(id));
        assert_eq!(groups.get(group(1)).len(), MAX_SELECTED);
    }

    #[test]
    fn control_groups_assign_add_without_duplicates_and_forget() {
        let mut groups = ControlGroups::EMPTY;
        groups.assign(group(1), [id(5), id(2), id(5)]);
        assert_eq!(groups.get(group(1)), [id(2), id(5)]);
        groups.add(group(1), [id(3), id(2)]);
        assert_eq!(groups.get(group(1)), [id(2), id(3), id(5)]);
        groups.assign(group(0), [id(3)]);
        // An object may be in several groups; a removed one leaves all of them.
        assert!(groups.retain(|i| i != id(3)));
        assert_eq!(groups.get(group(1)), [id(2), id(5)]);
        assert!(groups.get(group(0)).is_empty());
        assert!(!groups.retain(|_| true));
        groups.assign(group(1), []);
        assert!(groups.get(group(1)).is_empty());
    }

    #[test]
    fn a_control_state_drops_objects_from_both() {
        let mut state = ControlState::EMPTY;
        state.selection = Selection::new([(id(1), PK), (id(2), PK)]);
        state.groups.assign(group(4), [id(2), id(3)]);
        assert!(state.retain(|i| i != id(2)));
        assert_eq!(state.selection.ids().collect::<Vec<_>>(), [id(1)]);
        assert_eq!(state.groups.get(group(4)), [id(3)]);
        assert!(!state.retain(|_| true));
    }

    /// `> ControlState`: a removal resets the ObjectInterfaceState only when it takes away the ActiveGroup.
    #[test]
    fn a_removal_resets_the_interface_state_only_with_the_active_group() {
        let awaiting = InterfaceState::AwaitingTarget(crate::command_ui::UnitCommand::Attack);
        let mut state = ControlState::EMPTY;
        state.selection = Selection::new([(id(1), PK), (id(2), PK), (id(3), GUARD)]);
        state.interface = awaiting;
        assert!(state.retain(|i| i != id(2)));
        assert_eq!(state.interface, awaiting, "the Peacekeepers' group is still active");
        assert!(state.retain(|i| i != id(1)));
        assert_eq!(state.selection.active().map(|k| k.kind), Some(GUARD));
        assert_eq!(state.interface, InterfaceState::DefaultState);
    }
}
