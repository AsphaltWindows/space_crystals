use super::*;

const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;

/// The objects of the highlighted portraits, in the panel's order.
fn highlighted(selection: &Selection) -> Vec<u32> {
    portraits(selection, 10)
        .into_iter()
        .filter_map(|cell| match cell {
            PortraitCell::Object { id, active: true, .. } => Some(id.0),
            _ => None,
        })
        .collect()
}

#[test]
fn portraits_need_two_objects_and_end_in_a_count_when_full() {
    let selection = |n: u32| Selection::new((1..=n).map(|i| (ObjectId(i), PK)));
    assert!(portraits(&selection(1), 10).is_empty());
    let two = portraits(&selection(2), 10);
    assert_eq!(
        two,
        [1, 2].map(|i| PortraitCell::Object {
            id: ObjectId(i),
            kind: PK,
            unfinished: false,
            active: true
        })
    );
    let full = portraits(&selection(12), 10);
    assert_eq!(full.len(), 10);
    assert_eq!(full[9], PortraitCell::More { count: 3, active: true });
    let exactly = portraits(&selection(10), 10);
    assert_eq!(exactly.len(), 10, "exactly full shows them all");
    assert!(exactly.iter().all(|cell| matches!(cell, PortraitCell::Object { .. })));
}

#[test]
fn only_the_active_groups_portraits_are_highlighted() {
    let mut selection = Selection::new([(ObjectId(1), PK), (ObjectId(2), GUARD), (ObjectId(3), PK)]);
    selection.cycle(true);
    let active: Vec<(ObjectId, bool)> = portraits(&selection, 10)
        .into_iter()
        .map(|c| match c {
            PortraitCell::Object { id, active, .. } => (id, active),
            PortraitCell::More { .. } => unreachable!(),
        })
        .collect();
    assert_eq!(
        active,
        [(ObjectId(1), false), (ObjectId(3), false), (ObjectId(2), true)]
    );
}

/// `systems/controls/selection_and_hud.md > SelectionPanel` (design change 0055): the portraits come group by group,
/// and Tab and Shift-Tab make the groups active in that order, wrapping around. Patches stand in for the Ungroupable
/// own structures that don't exist yet (a resource is never selected with other objects, 0048).
#[test]
fn tab_and_shift_tab_follow_the_order_of_the_portraits() {
    let patch = ObjectKind::CrystalPatch;
    let mut selection = Selection::new([
        (ObjectId(1), GUARD),
        (ObjectId(2), PK),
        (ObjectId(3), patch),
        (ObjectId(4), PK),
        (ObjectId(5), patch),
    ]);
    let order: Vec<u32> = portraits(&selection, 10)
        .into_iter()
        .map(|cell| match cell {
            PortraitCell::Object { id, .. } => id.0,
            PortraitCell::More { .. } => unreachable!(),
        })
        .collect();
    assert_eq!(
        order,
        [3, 5, 2, 4, 1],
        "each patch alone, then the Peacekeepers, then the Guard"
    );
    assert_eq!(highlighted(&selection), [3], "the first group is active");
    let mut tab = |forward: bool| {
        selection.cycle(forward);
        highlighted(&selection)
    };
    let forward: Vec<Vec<u32>> = (0..4).map(|_| tab(true)).collect();
    assert_eq!(forward, [vec![5], vec![2, 4], vec![1], vec![3]]);
    let backward: Vec<Vec<u32>> = (0..4).map(|_| tab(false)).collect();
    assert_eq!(backward, [vec![1], vec![2, 4], vec![5], vec![3]]);
}

/// The count of the portraits that don't fit is highlighted when it hides some of the ActiveGroup's.
#[test]
fn the_count_of_hidden_portraits_is_highlighted_when_it_hides_the_active_group() {
    let pks = (1..=9).map(|i| (ObjectId(i), PK));
    let mut selection = Selection::new(pks.chain((10..=12).map(|i| (ObjectId(i), GUARD))));
    let cells = portraits(&selection, 10);
    assert!(cells[..9].iter().all(|cell| cell.is_active()));
    assert_eq!(
        cells[9],
        PortraitCell::More {
            count: 3,
            active: false
        }
    );
    selection.cycle(true);
    let cells = portraits(&selection, 10);
    assert!(cells[..9].iter().all(|cell| !cell.is_active()));
    assert_eq!(
        cells[9],
        PortraitCell::More { count: 3, active: true },
        "the Guards are all hidden"
    );
}

/// Design change 0185: 2 finished Bio Portals and 1 warping in, two groups; the unfinished one's portrait comes
/// after the finished ones' and is marked, and Tab highlights it alone.
#[test]
fn an_unfinished_buildings_portrait_comes_in_a_group_of_its_own() {
    let bio = ObjectKind::BioPortal;
    let unfinished = crate::selection::GroupType::unfinished(bio);
    let mut selection = Selection::new([
        (ObjectId(3), unfinished),
        (ObjectId(4), bio.into()),
        (ObjectId(5), bio.into()),
    ]);
    let cells = |selection: &Selection| -> Vec<(u32, bool, bool)> {
        portraits(selection, 10)
            .into_iter()
            .map(|cell| match cell {
                PortraitCell::Object {
                    id, unfinished, active, ..
                } => (id.0, unfinished, active),
                PortraitCell::More { .. } => unreachable!(),
            })
            .collect()
    };
    assert_eq!(
        cells(&selection),
        [(4, false, true), (5, false, true), (3, true, false)]
    );
    selection.cycle(true);
    assert_eq!(
        cells(&selection),
        [(4, false, false), (5, false, false), (3, true, true)]
    );
}
