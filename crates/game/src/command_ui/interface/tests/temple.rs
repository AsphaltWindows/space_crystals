//! (M5-N3) The Temple's panel as plain data (`factions/cults/structures/temple.md > Placeholder stats`: "Panel:
//! **Q** Enhanced Constitution (next level), **W** Improved Metabolism (next level), **A/S/D/F** release one Recruit of
//! a kind (README > Choosing Recruits, Release and Eject All), **V: Eject All** (staff leave), **X: Cancel** (last
//! research)"): its grid, its keys following the gate's answers, its releases' counts, and who its commands go to. In
//! a game: `crates/game/tests/commands/temple.rs`.

use sc_sim::{HoldCommand, RecruitKind, Tech};

use super::cults::counted;
use super::*;
use crate::command_ui::feedback::{IN_PROGRESS_TEXT, unavailable_text};

const TEMPLE: ObjectKind = ObjectKind::Temple;
const EC1: Tech = Tech::EnhancedConstitutionL1;
const EC2: Tech = Tech::EnhancedConstitutionL2;
const IM1: Tech = Tech::ImprovedMetabolismL1;
const IM2: Tech = Tech::ImprovedMetabolismL2;

/// The reason greying the button in `slot`, as pressing it says it.
fn why_text(panel: &Panel, slot: CommandSlot) -> Option<&'static str> {
    panel.get(slot)?.why.map(unavailable_text)
}

/// The Temple's DefaultState with no research done: Q and W each line's Level 1, labelled with its line's initials
/// and its level and named after the technology; A, S, D and F the releases of one Recruit of each kind, in the kind
/// buttons' order, with the count of its staff of the kind, greyed where the gate refuses them for want of one
/// (`Reject::NoRecruit`: "it is greyed if there are none"); X Cancel Production, the last research; V Eject All. Each
/// is issued at once: Q its Research, A its release of one Recruit, V the Eject All. No C and no Z: no rally point.
#[test]
fn the_temples_panel_researches_releases_and_ejects() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, ReleaseOne, Research};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let temple = Selection::new([(id(1), TEMPLE)]);
    let conditions = Conditions::default()
        .with_pool([2, 1, 0, 0])
        .refusing(ReleaseOne(Metabolism), Reject::NoRecruit(Metabolism))
        .refusing(ReleaseOne(Both), Reject::NoRecruit(Both));
    let shown = panel(&temple, InterfaceState::DefaultState, &conditions);
    let command = PanelButton::Command;
    let (x, v) = (CommandSlot::CANCEL_PRODUCTION, slot(2, 3));
    assert_eq!(
        counted(&shown),
        [
            (slot(0, 0), command(Research(EC1)), Common, None),
            (slot(0, 1), command(Research(IM1)), Common, None),
            (slot(1, 0), command(ReleaseOne(Plain)), Common, Some(2)),
            (slot(1, 1), command(ReleaseOne(Constitution)), Common, Some(1)),
            (slot(1, 2), command(ReleaseOne(Metabolism)), Disabled, Some(0)),
            (slot(1, 3), command(ReleaseOne(Both)), Disabled, Some(0)),
            (x, command(CancelProduction), Common, None),
            (v, command(EjectAll), Common, None),
        ]
    );
    assert_eq!(InterfaceTemplate::of(TEMPLE), Some(InterfaceTemplate::Temple));
    let labels: Vec<&str> = shown.slots().filter_map(|(_, s)| Some(s?.label())).collect();
    assert_eq!(
        labels,
        ["EC L1", "IM L1", "Plain", "Const.", "Metab.", "Both", "Cancel", "Eject"]
    );
    let names: Vec<String> = shown.slots().filter_map(|(slot, _)| shown.name(slot)).collect();
    assert_eq!(
        names,
        [
            "Research Enhanced Constitution Level 1",
            "Research Improved Metabolism Level 1",
            "Release one Plain Recruit",
            "Release one Constitution Recruit",
            "Release one Metabolism Recruit",
            "Release one Both Recruit",
            "Cancel Production",
            "Eject All",
        ]
    );
    assert_eq!(why_text(&shown, slot(1, 2)), Some("No free unit of that kind"));
    assert_eq!(
        press(&shown, slot(0, 0)),
        SlotPress::Produce(Research(EC1), ProductionCommand::Research(EC1))
    );
    assert_eq!(
        press(&shown, slot(1, 0)),
        SlotPress::Hold(ReleaseOne(Plain), HoldCommand::ReleaseOne(Plain))
    );
    assert_eq!(press(&shown, slot(1, 2)), SlotPress::Nothing, "greyed");
    assert_eq!(press(&shown, v), SlotPress::Hold(EjectAll, HoldCommand::EjectAll));
    assert_eq!(
        press(&shown, x),
        SlotPress::Produce(CancelProduction, ProductionCommand::CancelProduction)
    );
    for free in [CommandSlot::BACK, CommandSlot::RALLY_POINT, slot(0, 2), slot(0, 3)] {
        assert_eq!(shown.get(free), None);
    }
}

/// `factions/cults/structures/temple.md > Placeholder stats`: "a Level 2 can't be queued until its Level 1 has
/// **completed** (its key shows "In progress" meanwhile)". A key shows its line's next level as the gate has it: Level
/// 1 "In progress" while the gate refuses it as under way (queued or in progress at any of the player's structures),
/// greyed, saying so when pressed and pressing nothing; Level 2 once Level 1 is completed, greyed "Requires a
/// Cathedral" while the gate wants one, and in progress in its turn; nothing once both are completed
/// (`core/tech_and_research.md > Research > Shared Rules` 4: "Once completed, it no longer appears in any research
/// menu"). Each line's key follows its own line.
#[test]
fn a_key_shows_its_lines_next_level_and_in_progress() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{Research, Researching};
    let temple = Selection::new([(id(1), TEMPLE)]);
    let (q, w) = (slot(0, 0), slot(0, 1));
    let keys = |conditions: &Conditions| {
        let shown = panel(&temple, InterfaceState::DefaultState, conditions);
        [q, w].map(|at| shown.get(at).map(|s| (s.button, s.look)))
    };
    let command = PanelButton::Command;
    let under_way = Conditions::default().refusing(Research(EC1), Reject::ResearchUnderWay(EC1));
    assert_eq!(
        keys(&under_way),
        [
            Some((command(Researching(EC1)), Disabled)),
            Some((command(Research(IM1)), Common))
        ]
    );
    let shown = panel(&temple, InterfaceState::DefaultState, &under_way);
    assert_eq!(shown.get(q).map(|s| s.label()), Some("InProg"));
    assert_eq!(why_text(&shown, q), Some(IN_PROGRESS_TEXT));
    assert_eq!(IN_PROGRESS_TEXT, "In progress");
    assert_eq!(
        shown.name(q).as_deref(),
        Some("Enhanced Constitution Level 1: In progress")
    );
    assert_eq!(press(&shown, q), SlotPress::Nothing);

    let level_2 = Conditions::default()
        .refusing(Research(EC1), Reject::Researched(EC1))
        .refusing(Research(EC2), Reject::Requires(ObjectKind::Cathedral));
    assert_eq!(keys(&level_2)[0], Some((command(Research(EC2)), Disabled)));
    let shown = panel(&temple, InterfaceState::DefaultState, &level_2);
    assert_eq!(shown.get(q).map(|s| s.label()), Some("EC L2"));
    assert_eq!(why_text(&shown, q), Some("Requires a Cathedral"));
    let cathedral = Conditions::default().refusing(Research(EC1), Reject::Researched(EC1));
    assert_eq!(keys(&cathedral)[0], Some((command(Research(EC2)), Common)));
    let both = cathedral.clone().refusing(Research(EC2), Reject::ResearchUnderWay(EC2));
    assert_eq!(keys(&both)[0], Some((command(Researching(EC2)), Disabled)));

    let done = Conditions::default()
        .refusing(Research(EC1), Reject::Researched(EC1))
        .refusing(Research(EC2), Reject::Researched(EC2))
        .refusing(Research(IM1), Reject::Researched(IM1))
        .refusing(Research(IM2), Reject::ResearchUnderWay(IM2));
    assert_eq!(keys(&done), [None, Some((command(Researching(IM2)), Disabled))]);
    for tech in Tech::ALL {
        assert!(
            offers(TEMPLE, Research(tech)) && offers(TEMPLE, Researching(tech)),
            "{tech:?}"
        );
        assert!(commands(TEMPLE).contains(&Research(tech)), "{tech:?}");
        assert!(!offers(ObjectKind::Armory, Research(tech)), "{tech:?}");
    }
}

/// The Temple is Groupable (`factions/cults/structures/temple.md > Placeholder stats`: "Groupable true"): two
/// Temples are one group, a Research goes to both, of which the sim takes one (`sc_sim::ProductionCommand::Research`,
/// design change 0083), and so do a release of one Recruit and an Eject All, each Temple letting its own out. With a
/// Barracks beside them, X is common, as a Barracks' X is with another producer's. It has no rally point: its panel
/// has no Set Rally Point (its right-click: `crate::command_ui::resolution`'s tests).
#[test]
fn temples_group_and_have_no_rally_point() {
    use PanelCommand::{CancelProduction, EjectAll, ReleaseOne, Research};
    let two = Selection::new([(id(1), TEMPLE), (id(2), TEMPLE)]);
    assert_eq!(recipients(&two, Research(EC1)), [id(1), id(2)]);
    assert_eq!(recipients(&two, ReleaseOne(RecruitKind::Plain)), [id(1), id(2)]);
    assert_eq!(recipients(&two, EjectAll), [id(1), id(2)]);
    let mixed = Selection::new([(id(1), TEMPLE), (id(3), BARRACKS)]);
    assert!(is_common(&mixed, CancelProduction));
    assert!(!offers(TEMPLE, PanelCommand::SetRallyPoint));
}
