use super::*;

const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;
/// Ungroupable (`core/map.md > SpaceCrystalsPatch`). The rules never select a resource with other objects (design
/// change 0048); the patch stands in for the Ungroupable own structures that don't exist yet.
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
    let key = |kind, instance| GroupKey {
        kind,
        unfinished: false,
        instance,
    };
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
    assert_eq!(Selection::new::<ObjectKind>([]).active(), None);
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
    let awaiting = InterfaceState::awaiting(crate::command_ui::PanelCommand::Attack);
    let mut state = ControlState::EMPTY;
    state.selection = Selection::new([(id(1), PK), (id(2), PK), (id(3), GUARD)]);
    state.interface = awaiting;
    assert!(state.retain(|i| i != id(2)));
    assert_eq!(state.interface, awaiting, "the Peacekeepers' group is still active");
    assert!(state.retain(|i| i != id(1)));
    assert_eq!(state.selection.active().map(|k| k.kind), Some(GUARD));
    assert_eq!(state.interface, InterfaceState::DefaultState);
}

/// Plan decision T15 (`SimEvent::KindChanged`): an object that changes its type in place stays selected, with its
/// id, in its new type's group. An ActiveGroup it leaves empty gives way to that group; one it doesn't leave
/// empty stays active. The ObjectInterfaceState resets only with the ActiveGroup. Beacons 1 and 2 (Ungroupable,
/// design change 0122: a group each) warp in Robo Portals (Groupable: one group for both) and an Egress Node
/// (Ungroupable); a cancel turns one back.
#[test]
fn a_kind_change_keeps_the_object_selected_in_its_new_group() {
    use ObjectKind::{Beacon, EgressNode, RoboPortal};
    let awaiting = InterfaceState::awaiting(crate::command_ui::PanelCommand::Attack);
    let mut state = ControlState::EMPTY;
    state.selection = Selection::new([(id(1), Beacon), (id(2), Beacon)]);
    assert_eq!(state.selection.active(), Some(GroupKey::of(id(1), Beacon)));
    let portals = GroupKey::of(id(1), RoboPortal);
    state.interface = awaiting;
    assert!(state.set_type(id(1), RoboPortal.into()));
    assert_eq!(state.selection.kind_of(id(1)), Some(RoboPortal));
    assert_eq!(state.selection.active(), Some(portals), "Beacon 1's group is gone");
    assert_eq!(state.interface, InterfaceState::DefaultState);
    state.interface = awaiting;
    assert!(state.set_type(id(2), RoboPortal.into()));
    assert_eq!(state.selection.active_group().unwrap().instances, [id(1), id(2)]);
    assert_eq!(state.interface, awaiting, "the ActiveGroup stays");
    assert!(!state.set_type(id(2), RoboPortal.into()), "no change");
    assert!(!state.set_type(id(3), RoboPortal.into()), "not selected");
    assert!(state.set_type(id(1), EgressNode.into()));
    let node = GroupKey::of(id(1), EgressNode);
    let keys: std::collections::BTreeSet<GroupKey> = state.selection.groups().iter().map(|g| g.key).collect();
    assert_eq!(keys, [node, portals].into(), "the Egress Node in a group of its own");
    assert_eq!(state.selection.active(), Some(portals), "Robo Portal 2 is still in it");
    assert!(state.set_type(id(2), Beacon.into()), "cancelled");
    assert_eq!(state.selection.active(), Some(GroupKey::of(id(2), Beacon)));
    assert_eq!(state.selection.ids().collect::<Vec<_>>(), [id(1), id(2)]);
}

/// Design change 0185 (`> SelectionGroup > Unfinished buildings form their own group`): a building under construction
/// or transformation groups apart from the finished ones of its type, "as if it were another type", in a group after
/// theirs; Ungroupable types still split by instance. 2 finished Bio Portals and 1 warping in: two groups, Tab between
/// them.
#[test]
fn unfinished_buildings_form_a_group_apart_from_the_finished_ones() {
    use ObjectKind::{Beacon, BioPortal};
    let unfinished = GroupType::unfinished;
    let s = Selection::new([
        (id(1), GroupType::finished(BioPortal)),
        (id(2), BioPortal.into()),
        (id(3), unfinished(BioPortal)),
        (id(4), PK.into()),
        (id(5), unfinished(Beacon)),
        (id(6), Beacon.into()),
    ]);
    let groups: Vec<(GroupKey, Vec<ObjectId>)> = s.groups().into_iter().map(|g| (g.key, g.instances)).collect();
    let key = |kind, unfinished, instance| GroupKey {
        kind,
        unfinished,
        instance,
    };
    assert_eq!(
        groups,
        [
            (key(PK, false, None), vec![id(4)]),
            (key(Beacon, false, Some(id(6))), vec![id(6)]),
            (key(Beacon, true, Some(id(5))), vec![id(5)]),
            (key(BioPortal, false, None), vec![id(1), id(2)]),
            (key(BioPortal, true, None), vec![id(3)]),
        ]
    );
    assert_eq!(
        GroupKey::of(id(3), unfinished(BioPortal)).group_type(),
        unfinished(BioPortal)
    );
    assert_eq!(
        (s.kind_of(id(3)), s.type_of(id(3))),
        (Some(BioPortal), Some(unfinished(BioPortal)))
    );
    assert_eq!(s.members().nth(2), Some((id(3), BioPortal)), "members give the kind");
    let mut portals = Selection::new([
        (id(1), BioPortal.into()),
        (id(2), BioPortal.into()),
        (id(3), unfinished(BioPortal)),
    ]);
    let active = |s: &Selection| s.active_group().map(|g| (g.is_unfinished(), g.instances));
    assert_eq!(active(&portals), Some((false, vec![id(1), id(2)])));
    assert!(portals.cycle(true));
    assert_eq!(active(&portals), Some((true, vec![id(3)])));
    assert!(portals.in_active_group(id(3)) && !portals.in_active_group(id(1)));
}

/// Design change 0185: "When it completes, it joins the finished group." The ActiveGroup is kept while it still
/// exists (`> ObjectInterfaceState`: a change the player didn't make keeps the state "unless the ActiveGroup becomes
/// empty"), with its ObjectInterfaceState; an unfinished group that its last building leaves gives way to the group
/// that building joins, and its state resets. Bio Portals 1 and 2 finished, 3 and 4 warping in.
#[test]
fn a_completed_building_joins_the_finished_group() {
    use ObjectKind::{Beacon, BioPortal};
    let (finished, unfinished) = (GroupType::finished(BioPortal), GroupType::unfinished(BioPortal));
    let rally = InterfaceState::awaiting(crate::command_ui::PanelCommand::SetRallyPoint);
    let portals = [
        (id(1), finished),
        (id(2), finished),
        (id(3), unfinished),
        (id(4), unfinished),
    ];
    let mut state = ControlState::EMPTY;
    state.selection = Selection::new(portals);
    state.interface = rally;
    assert!(state.set_type(id(3), finished));
    let groups = |state: &ControlState| -> Vec<Vec<ObjectId>> {
        state.selection.groups().into_iter().map(|g| g.instances).collect()
    };
    assert_eq!(groups(&state), [vec![id(1), id(2), id(3)], vec![id(4)]]);
    assert_eq!(
        state.selection.active(),
        Some(GroupKey::of(id(1), finished)),
        "the finished group stays active"
    );
    assert_eq!(state.interface, rally, "and keeps its state");

    state.selection = Selection::new(portals);
    state.selection.cycle(true);
    assert!(state.set_type(id(3), finished));
    assert_eq!(
        state.selection.active(),
        Some(GroupKey::of(id(4), unfinished)),
        "Bio Portal 4 is still in it"
    );
    assert_eq!(state.interface, rally);
    assert!(state.set_type(id(4), finished));
    assert_eq!(groups(&state), [vec![id(1), id(2), id(3), id(4)]]);
    assert_eq!(
        state.selection.active(),
        Some(GroupKey::of(id(4), finished)),
        "the group it joined"
    );
    assert_eq!(
        state.interface,
        InterfaceState::DefaultState,
        "the unfinished group's state ended with it"
    );
    assert!(!state.set_type(id(4), finished), "no change");

    // A cancelled Warp In: the building is a Beacon again, a group of its own (Ungroupable).
    state.selection = Selection::new([(id(1), finished), (id(3), unfinished)]);
    state.selection.cycle(true);
    assert!(state.set_type(id(3), Beacon.into()));
    assert_eq!(state.selection.active(), Some(GroupKey::of(id(3), Beacon)));
}
