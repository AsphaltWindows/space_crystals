//! (M5-S6) The panel of an unfinished building that will produce or release units (design change 0396): C, Set Rally
//! Point, next to its Cancel, on a site and while it transforms; none on a building that produces nothing.

use super::*;
use crate::selection::GroupType;

const X: CommandSlot = CommandSlot::CANCEL_PRODUCTION;
const C: CommandSlot = CommandSlot::RALLY_POINT;

/// Design change 0396: "Its unfinished panel adds **C: Set Rally Point**, next to Cancel." An Armory's site, which its
/// Recruits build, shows X Cancel Construction and C, which awaits its target as on the finished Armory; so do the
/// sites of the other buildings that will produce or release units, a Recruitment Hall's, an Altar's, a Cathedral's
/// and a Hideout's (an expansion). A site of one that produces nothing shows X alone: a Storage's, a Garrison's (it
/// "has no rally point", `factions/cults/structures/garrison.md > Placeholder stats > Panel`) and a Temple's, which
/// researches.
#[test]
fn an_unfinished_producer_shows_its_rally_point_next_to_cancel() {
    use ButtonLook::Common;
    use PanelCommand::{CancelConstruction, SetRallyPoint};
    let c = PanelButton::Command;
    let on_site = Conditions::default().on_site();
    let site_panel = |kind| {
        let site = Selection::new([(id(1), GroupType::unfinished(kind))]);
        panel(&site, InterfaceState::DefaultState, &on_site)
    };
    for kind in [
        ObjectKind::Armory,
        ObjectKind::RecruitmentHall,
        ObjectKind::Altar,
        ObjectKind::Cathedral,
        ObjectKind::Hideout,
    ] {
        let shown_panel = site_panel(kind);
        assert_eq!(
            shown(&shown_panel),
            [(X, c(CancelConstruction), Common), (C, c(SetRallyPoint), Common)],
            "{kind:?}"
        );
        assert_eq!(
            press(&shown_panel, C),
            SlotPress::AwaitTarget(SetRallyPoint),
            "{kind:?}"
        );
        assert_eq!(shown_panel.get(C).map(|slot| slot.label()), Some("Rally"));
        assert!(offers(GroupType::unfinished(kind), SetRallyPoint), "{kind:?}");
    }
    for kind in [ObjectKind::Storage, ObjectKind::Garrison, ObjectKind::Temple] {
        let shown_panel = site_panel(kind);
        assert_eq!(shown(&shown_panel), [(X, c(CancelConstruction), Common)], "{kind:?}");
        assert_eq!(press(&shown_panel, C), SlotPress::Nothing, "{kind:?}");
        assert!(!offers(GroupType::unfinished(kind), SetRallyPoint), "{kind:?}");
    }
}

/// A pool building takes a command one building at a time (design change 0348 point 5: "a command goes to one
/// building"), and an unfinished one its C alike ("as on the finished building", design change 0396), as it takes its
/// Cancel: of two Armory sites selected, each a group of its own, C goes to the one Tabbed to. Two Recruitment Hall
/// sites, which aren't pool buildings, both take it, a common command.
#[test]
fn an_unfinished_armory_takes_its_rally_point_alone() {
    use PanelCommand::{CancelConstruction, SetRallyPoint};
    let armory = GroupType::unfinished(ObjectKind::Armory);
    let mut sites = Selection::new([(id(1), armory), (id(2), armory)]);
    assert_eq!(recipients(&sites, SetRallyPoint), [id(1)]);
    assert_eq!(recipients(&sites, CancelConstruction), [id(1)]);
    let hall = GroupType::unfinished(ObjectKind::RecruitmentHall);
    let halls = Selection::new([(id(3), hall), (id(4), hall)]);
    assert!(is_common(&halls, SetRallyPoint));
    assert_eq!(recipients(&halls, SetRallyPoint), [id(3), id(4)]);
    let shown_panel = panel(&sites, InterfaceState::DefaultState, &Conditions::default().on_site());
    assert_eq!(shown_panel.get(C).map(|slot| slot.look), Some(ButtonLook::Group));
    assert!(sites.cycle(true));
    assert_eq!(recipients(&sites, SetRallyPoint), [id(2)]);
}
