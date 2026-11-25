//! The selection model (`systems/controls/selection_and_hud.md > Selection`, `> SelectionGroup`, `> ControlGroups`):
//! plain data without Bevy systems. Which objects may be selected together (`> Selection > Constraints`) is the rules'
//! business (`super::rules`), because it depends on who owns them.

use std::collections::BTreeMap;

use sc_sim::{Ability, Object, ObjectId, ObjectKind};

use crate::command_ui::InterfaceState;
use crate::input::ControlGroup;

/// The most objects a selection or a control group holds. Placeholder, unspecified: `> Selection > Constraints`
/// sets no limit on a selection of own objects; this is the sim's limit for one order
/// (`sc_sim::orders::MAX_ORDER_UNITS`), so that one command can always carry the whole selection.
pub const MAX_SELECTED: usize = sc_sim::orders::MAX_ORDER_UNITS;

/// What the selection groups objects by (`> SelectionGroup > Type`): their type, and whether they are finished. A
/// building still under construction or transformation forms a SelectionGroup of its own, apart from the finished
/// buildings of its type, "as if it were another type", with its own panel (only Cancel, `crate::command_ui`); when
/// it completes, it joins the finished group (`> Unfinished buildings form their own group`, design change 0185: the
/// group key is "(type, finished?)"). Ordered by type (`ObjectKind` order), the finished before the unfinished.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupType {
    pub kind: ObjectKind,
    /// Under construction or transformation: a Beacon a Drone deploys into, a building warped in from a Beacon
    /// (`sc_sim::Object::is_transforming`, the one way a building is unfinished in the game yet).
    pub unfinished: bool,
}

impl GroupType {
    /// A finished `kind`.
    pub const fn finished(kind: ObjectKind) -> Self {
        Self {
            kind,
            unfinished: false,
        }
    }

    /// An unfinished `kind`.
    pub const fn unfinished(kind: ObjectKind) -> Self {
        Self { kind, unfinished: true }
    }

    /// The type `object` groups as now, as the sim has it.
    pub fn of(object: &Object) -> Self {
        Self {
            kind: object.kind,
            unfinished: object.is_transforming(),
        }
    }
}

/// A finished `kind`: every object that is no building, and every building but those under construction or
/// transformation.
impl From<ObjectKind> for GroupType {
    fn from(kind: ObjectKind) -> Self {
        Self::finished(kind)
    }
}

/// Names a SelectionGroup (`> SelectionGroup`): its type, whether it is finished ([`GroupType`]) and, for an
/// Ungroupable type (`Groupable = false`), its one instance, since each instance of such a type is a group of its
/// own. The ActiveGroup is one SelectionGroup, which for an Ungroupable type means one instance (`> Selection >
/// ActiveGroup`, design change 0048); the client names it by this key. Groups are ordered by it: by type
/// (`ObjectKind` order), the finished before the unfinished, then by instance.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupKey {
    pub kind: ObjectKind,
    /// The group of the unfinished objects of its type (design change 0185).
    pub unfinished: bool,
    /// The instance of an Ungroupable type's group; `None` for a Groupable type, whose instances share one group.
    pub instance: Option<ObjectId>,
}

impl GroupKey {
    /// The group that object `id` of type `ty` (a kind: a finished one) belongs to.
    pub fn of(id: ObjectId, ty: impl Into<GroupType>) -> Self {
        let GroupType { kind, unfinished } = ty.into();
        Self {
            kind,
            unfinished,
            instance: (!kind.def().groupable).then_some(id),
        }
    }

    /// The type its objects group as.
    pub fn group_type(self) -> GroupType {
        GroupType {
            kind: self.kind,
            unfinished: self.unfinished,
        }
    }
}

/// `> SelectionGroup`: selected instances of one type, all finished or all unfinished ([`GroupType`]).
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

    /// Whether it is the group of unfinished buildings of its type (design change 0185).
    pub fn is_unfinished(&self) -> bool {
        self.key.unfinished
    }
}

/// `> Selection`: the selected objects, grouped by type and whether they are finished ([`Selection::groups`]), and
/// the ActiveGroup, which always names one of the groups while the selection isn't empty (`> Selection >
/// Constraints`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    members: BTreeMap<ObjectId, GroupType>,
    active: Option<GroupKey>,
}

impl Selection {
    pub const EMPTY: Selection = Selection {
        members: BTreeMap::new(),
        active: None,
    };

    /// A selection of these objects (at most [`MAX_SELECTED`], the first by id), with the types they group as (a
    /// kind: a finished one), whose first group is active.
    pub fn new<T: Into<GroupType>>(objects: impl IntoIterator<Item = (ObjectId, T)>) -> Self {
        let mut members: BTreeMap<ObjectId, GroupType> = objects.into_iter().map(|(id, ty)| (id, ty.into())).collect();
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
        self.members.get(&id).map(|ty| ty.kind)
    }

    /// The type the selected object `id` groups as: its kind, and whether it is finished.
    pub fn type_of(&self, id: ObjectId) -> Option<GroupType> {
        self.members.get(&id).copied()
    }

    /// The selected objects with their types, in id order.
    pub fn members(&self) -> impl Iterator<Item = (ObjectId, ObjectKind)> + '_ {
        self.members.iter().map(|(&id, ty)| (id, ty.kind))
    }

    /// The selected objects with the types they group as ([`GroupType`]), in id order.
    pub fn member_types(&self) -> impl Iterator<Item = (ObjectId, GroupType)> + '_ {
        self.members.iter().map(|(&id, &ty)| (id, ty))
    }

    /// The selected objects, in id order.
    pub fn ids(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.members.keys().copied()
    }

    /// The SelectionGroups, in [`GroupKey`] order: the one order in which the SelectionPanel shows them and Tab and
    /// Shift-Tab cycle them (`> SelectionPanel`, design change 0055: a stable order of the implementation's choosing,
    /// a placeholder until the HUD pass), a type's unfinished group after its finished one. It doesn't depend on the
    /// order in which the objects were selected.
    pub fn groups(&self) -> Vec<SelectionGroup> {
        let mut keyed: Vec<(GroupKey, ObjectId)> =
            self.member_types().map(|(id, ty)| (GroupKey::of(id, ty), id)).collect();
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
            .is_some_and(|&ty| Some(GroupKey::of(id, ty)) == self.active)
    }

    /// Adds an object, which groups as `ty` (a kind: a finished one), unless it is selected already or the selection
    /// is full. The ActiveGroup stays.
    pub fn insert(&mut self, id: ObjectId, ty: impl Into<GroupType>) -> bool {
        if self.members.contains_key(&id) || self.members.len() >= MAX_SELECTED {
            return false;
        }
        self.members.insert(id, ty.into());
        self.fix_active();
        true
    }

    /// Removes an object. The ActiveGroup stays if it still has members, else the next group in Tab order becomes
    /// active ([`Selection::fix_active`]).
    pub fn remove(&mut self, id: ObjectId) -> bool {
        let removed = self.members.remove(&id).is_some();
        if removed {
            self.fix_active();
        }
        removed
    }

    /// Keeps only the objects for which `keep` holds, given their ids and the types they group as; the ActiveGroup as
    /// in [`Selection::remove`]. Says whether any object was dropped.
    pub fn retain(&mut self, mut keep: impl FnMut(ObjectId, GroupType) -> bool) -> bool {
        let before = self.members.len();
        self.members.retain(|&id, &mut ty| keep(id, ty));
        let changed = self.members.len() != before;
        if changed {
            self.fix_active();
        }
        changed
    }

    /// The type object `id` groups as changed in place, to `ty`: its kind (`SimEvent::KindChanged`, plan decision T15:
    /// a Beacon warping in a building; a mode toggle's press, design change 0198), or whether it is finished (design
    /// change 0185: a building that completes "joins the finished group"). It stays selected, with the same id, in its
    /// new type's group. An ActiveGroup that it leaves empty gives way to its new group (`> Selection > ActiveGroup >
    /// When the ActiveGroup disappears`, design change 0199: "Its members changed type and joined another group [...]
    /// the group they joined becomes active"); otherwise the ActiveGroup stays ("Any other group change [...] never
    /// changes the ActiveGroup"). Says whether anything changed.
    pub fn set_type(&mut self, id: ObjectId, ty: GroupType) -> bool {
        let Some(member) = self.members.get_mut(&id) else {
            return false;
        };
        if *member == ty {
            return false;
        }
        let left = GroupKey::of(id, *member);
        *member = ty;
        if self.active == Some(left) && !self.exists(left) {
            self.active = Some(GroupKey::of(id, ty));
        }
        true
    }

    /// Whether the group `key` has members.
    fn exists(&self, key: GroupKey) -> bool {
        self.members.iter().any(|(&id, &ty)| GroupKey::of(id, ty) == key)
    }

    /// `> GroupCycling`: makes the next group (`forward`) or the previous one active, in [`Selection::groups`] order,
    /// wrapping around at the ends.
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

    /// `> Selection > Constraints`: ActiveGroup must name an existing group. An ActiveGroup whose last member left
    /// the selection (it died, or was removed from it) gives way to the next group in Tab order, wrapping around to
    /// the first (`> Selection > ActiveGroup > When the ActiveGroup disappears`, design change 0199: "the next group
    /// in Tab order becomes active (the group after it in the SelectionPanel, wrapping around to the first). If no
    /// group is left, the selection is empty"): the first group after its key ([`Selection::groups`] order). A
    /// selection that had none, a new one, has its first group active.
    fn fix_active(&mut self) {
        match self.active {
            Some(key) if self.exists(key) => {}
            gone => {
                let groups = self.groups();
                let next = gone.and_then(|gone| groups.iter().find(|g| g.key > gone));
                self.active = next.or(groups.first()).map(|g| g.key);
            }
        }
    }
}

/// `> ControlGroups`: 10 saved selections. An object may be in several groups; a group never holds an object twice,
/// and holds only the player's own objects (design change 0056, which the rules see to: `ControlState::assign_group`
/// and `add_to_group`).
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
    /// ActiveGroup that is gone does (it was its type's, `> ObjectInterfaceState[ObjectEnum]`; `> AwaitingTarget >
    /// Selection changes while targeting`: "Targeting is only cancelled if the whole ActiveGroup is gone"), the next
    /// group in Tab order becoming active (design change 0199, [`Selection::remove`]), and the command flow checks the
    /// rest against the game every frame.
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

    /// The type object `id` groups as changed in place, to `ty`: its kind, or whether it is finished
    /// ([`Selection::set_type`]). Selected, it stays so; the control groups, which hold ids, keep it as they are. An
    /// ObjectInterfaceState of an ActiveGroup that the change replaces resets to DefaultState, as in
    /// [`ControlState::retain`]: it was the old type's. Says whether anything changed.
    pub fn set_type(&mut self, id: ObjectId, ty: GroupType) -> bool {
        let active = self.selection.active();
        let changed = self.selection.set_type(id, ty);
        if changed && self.selection.active() != active {
            self.interface = InterfaceState::DefaultState;
        }
        changed
    }

    /// The selected `units` were pressed with the mode toggle `ability` (`factions/colonists/units.md > Weapon
    /// Conversion > A toggle sets a target form`, `factions/gdo/units.md > Peacekeeper > Riot Shield > Two unit
    /// types`; design change 0198): "The unit's type, and so its SelectionGroup, switches at the press to the target
    /// form. The pressed units join the group of that form (merging with any units of that form already selected),
    /// and that merged group becomes the ActiveGroup", the panel back in its DefaultState (design change 0199). Each
    /// takes the type the sim switches it to when it applies the press (`sc_sim::Ability::switched_kind`), so the
    /// panel shows the new group's commands at once, and a second press goes to the merged group (until the sim has
    /// applied the press, `crate::sim_driver::PendingCommands::kind_of` keeps the type the selection's checks find).
    /// Says whether anything changed.
    pub fn toggle(&mut self, units: &[ObjectId], ability: Ability) -> bool {
        let mut joined = None;
        for &id in units {
            let Some(ty) = self.selection.type_of(id) else {
                continue;
            };
            if let Some(kind) = ability.switched_kind(ty.kind) {
                let ty = GroupType { kind, ..ty };
                self.set_type(id, ty);
                joined = Some(GroupKey::of(id, ty));
            }
        }
        let Some(joined) = joined else {
            return false;
        };
        self.selection.active = Some(joined);
        self.interface = InterfaceState::DefaultState;
        true
    }
}

#[cfg(test)]
mod tests;
