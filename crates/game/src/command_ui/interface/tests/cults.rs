//! The Cults' panels as plain data (slice M5-C): the Recruitment Hall's, read from a game, which has no X (design change
//! 0342); the Armory's (slice M5-F2, design change 0337): a kind of Recruit first, then its KindMenu, each command to
//! the one Armory Tabbed to (design change 0348 point 5). The panel in a game, its counts and its presses, is
//! `crates/game/tests/commands/armory.rs`'s.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, HoldCommand, PlayerId, RecruitKind, SpawnAt, TilePos};

use super::*;
use crate::command_ui::feedback::REQUIRES_GUNWORKS_TEXT;

const HALL: ObjectKind = ObjectKind::RecruitmentHall;
const ARMORY: ObjectKind = ObjectKind::Armory;
const MILITANT: ObjectKind = ObjectKind::Militant;

/// The buttons of `panel` with their counts.
fn counted(panel: &Panel) -> Vec<(CommandSlot, PanelButton, ButtonLook, Option<u32>)> {
    panel
        .slots()
        .filter_map(|(slot, shown)| shown.map(|s| (slot, s.button, s.look, s.count)))
        .collect()
}

/// `factions/cults/structures/recruitment_hall.md > ObjectInterfaceState[RecruitmentHall]`: C Set Rally Point alone,
/// with no Train, as the Hall recruits by itself (`factions/cults/structures/recruitment_hall.md > Production
/// Behavior`), and no X (design change 0342: "Recruits can't be cancelled. The X button is removed"), before its first
/// step or with a Recruit in production: X's slot is empty and its key does nothing. C awaits its target, and the
/// Hall's right-click sets the rally point (its Production template).
#[test]
fn the_hall_has_c_alone_and_no_x() {
    use ButtonLook::Common;
    use PanelCommand::SetRallyPoint;
    let player = PlayerId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(player, Faction::Cults)
        .spawn(HALL, Some(player), SpawnAt::Tile(TilePos::new(8, 8)))
        .start();
    let hall = Selection::new([(id(1), HALL)]);
    let (x, c) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    let command = PanelButton::Command;
    for ticks in [0, 1, 100] {
        t.run(ticks);
        let shown_now = panel(
            &hall,
            InterfaceState::DefaultState,
            &Conditions::of(t.sim(), player, &hall),
        );
        assert_eq!(
            shown(&shown_now),
            [(c, command(SetRallyPoint), Common)],
            "tick {}",
            t.sim().tick()
        );
        assert_eq!(shown_now.get(x), None);
        assert_eq!(press(&shown_now, x), SlotPress::Nothing);
        assert_eq!(press(&shown_now, c), SlotPress::AwaitTarget(SetRallyPoint));
    }
    assert!(t.sim().production_status(id(1)).is_some(), "a Recruit in production");
    assert_eq!(InterfaceTemplate::of(HALL), Some(InterfaceTemplate::Production));
}

/// `factions/cults/structures/armory.md > ObjectInterfaceState[Armory]` and design change 0337
/// (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`): "DefaultState: **Q** Plain, **W**
/// Constitution, **E** Metabolism, **R** Both (each enters KindMenu[kind]); **A** Eject All; **C** Set Rally Point;
/// **X** Cancel Production". Each kind's button shows its count of free Recruits inside, "greyed if there are none",
/// saying so; a kind with some opens its KindMenu, A ejects at once, X cancels at once and C awaits the rally point.
#[test]
fn the_armory_picks_a_kind_of_recruit_first() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, SetRallyPoint};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let armory = Selection::new([(id(1), ARMORY)]);
    let conditions = Conditions::default().with_pool([2, 1, 0, 0]);
    let default = panel(&armory, InterfaceState::DefaultState, &conditions);
    let command = PanelButton::Command;
    let (x, c) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        counted(&default),
        [
            (slot(0, 0), command(KindMenu(Plain)), Common, Some(2)),
            (slot(0, 1), command(KindMenu(Constitution)), Common, Some(1)),
            (slot(0, 2), command(KindMenu(Metabolism)), Disabled, Some(0)),
            (slot(0, 3), command(KindMenu(Both)), Disabled, Some(0)),
            (slot(1, 0), command(EjectAll), Common, None),
            (x, command(CancelProduction), Common, None),
            (c, command(SetRallyPoint), Common, None),
        ]
    );
    let labels = [slot(0, 0), slot(0, 1), slot(1, 0)].map(|slot| default.get(slot).unwrap().label());
    assert_eq!(labels, ["Plain", "Const.", "Eject"]);
    assert_eq!(press(&default, slot(0, 1)), SlotPress::OpenKindMenu(Constitution));
    assert_eq!(press(&default, slot(0, 2)), SlotPress::Nothing);
    let why = default.get(slot(0, 2)).and_then(|shown| shown.why);
    assert_eq!(why, Some(WhyUnavailable::Refused(Reject::NoRecruit(Metabolism))));
    assert_eq!(
        press(&default, slot(1, 0)),
        SlotPress::Hold(EjectAll, HoldCommand::EjectAll)
    );
    assert_eq!(
        press(&default, x),
        SlotPress::Produce(CancelProduction, ProductionCommand::CancelProduction)
    );
    assert_eq!(press(&default, c), SlotPress::AwaitTarget(SetRallyPoint));
    // An empty pool greys Eject All by the gate's refusal (`Conditions::of` asks it).
    let empty = Conditions::default().refusing(EjectAll, Reject::NothingInside);
    let shown = panel(&armory, InterfaceState::DefaultState, &empty);
    assert_eq!(shown.get(slot(1, 0)).map(|s| s.look), Some(Disabled));
    assert_eq!(InterfaceTemplate::of(ARMORY), Some(InterfaceTemplate::PoolProducer));
}

/// Design change 0337: KindMenu[kind] has "the building's production options, using Recruits of that kind (e.g.
/// Armory: **Q** Train Militant, **W** Train Gunner [...]), plus **A: Release**; **Z** back. Clicks stay in the menu so
/// they can be repeated." Q trains a Militant of a Recruit of the kind and A releases one, both at once; the Gunner, not
/// in the game yet, holds W, greyed: it "Requires a Gunworks" (`factions/cults/structures/armory.md > Training`). The
/// menu stays open while no Recruit of its kind is left (its entries greyed by the gate then), and Z goes back to
/// DefaultState.
#[test]
fn a_kind_menu_trains_and_releases_recruits_of_its_kind() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::{Planned as P, Release, TrainFrom};
    use RecruitKind::Metabolism;
    let armory = Selection::new([(id(1), ARMORY)]);
    let conditions = Conditions::default().with_pool([0, 0, 1, 0]);
    let menu = InterfaceState::KindMenu(Metabolism);
    let shown_menu = panel(&armory, menu, &conditions);
    let command = PanelButton::Command;
    assert_eq!(
        shown(&shown_menu),
        [
            (slot(0, 0), command(TrainFrom(MILITANT, Metabolism)), Common),
            (slot(0, 1), command(P(Planned::Gunner)), Unavailable),
            (slot(1, 0), command(Release(Metabolism)), Common),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let labels = [slot(0, 0), slot(0, 1), slot(1, 0)].map(|slot| shown_menu.get(slot).unwrap().label());
    assert_eq!(labels, ["MT", "GU", "Rel."]);
    assert_eq!(
        press(&shown_menu, slot(0, 0)),
        SlotPress::Produce(
            TrainFrom(MILITANT, Metabolism),
            ProductionCommand::TrainFrom {
                kind: MILITANT,
                from: Metabolism
            }
        )
    );
    assert_eq!(
        press(&shown_menu, slot(1, 0)),
        SlotPress::Produce(Release(Metabolism), ProductionCommand::Release(Metabolism))
    );
    assert_eq!(press(&shown_menu, slot(0, 1)), SlotPress::Nothing);
    let why = shown_menu.get(slot(0, 1)).and_then(|shown| shown.why);
    assert_eq!(why, Some(WhyUnavailable::Placeholder(REQUIRES_GUNWORKS_TEXT)));
    assert_eq!(press(&shown_menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(menu.back(&armory), InterfaceState::DefaultState);

    // Its last Recruit of the kind used: the menu holds, its entries greyed by the gate.
    let used = (Conditions::default())
        .refusing(TrainFrom(MILITANT, Metabolism), Reject::NoRecruit(Metabolism))
        .refusing(Release(Metabolism), Reject::NoRecruit(Metabolism));
    assert!(still_valid(menu, &armory, true, &used));
    let greyed = panel(&armory, menu, &used);
    for entry in [slot(0, 0), slot(1, 0)] {
        assert_eq!(greyed.get(entry).map(|s| s.look), Some(Disabled), "{entry:?}");
        assert_eq!(press(&greyed, entry), SlotPress::Nothing);
    }
    let barracks = Selection::new([(id(2), BARRACKS)]);
    assert!(!still_valid(menu, &barracks, true, &Conditions::default()));
    assert!(!still_valid(menu, &armory, false, &conditions), "not commandable");
}

/// Design change 0348 point 5 (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "Cults
/// production buildings (Armory, Altar, Cathedral, Chop Shop, Airstrip) are Ungroupable: a command goes to one
/// building"): two selected Armories are two SelectionGroups, and every command of the panel, X and C too, which a
/// Barracks shares with the other Barracks of the selection (`production_commands_go_to_every_selected_barracks`),
/// goes to the one Tabbed to, a group command.
#[test]
fn each_command_goes_to_the_armory_tabbed_to() {
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, Release, SetRallyPoint, TrainFrom};
    use RecruitKind::Plain;
    let mut two = Selection::new([(id(1), ARMORY), (id(2), ARMORY)]);
    assert_eq!(two.groups().len(), 2);
    let commands = [
        KindMenu(Plain),
        TrainFrom(MILITANT, Plain),
        Release(Plain),
        EjectAll,
        CancelProduction,
        SetRallyPoint,
    ];
    for armory in [id(1), id(2)] {
        assert!(two.in_active_group(armory));
        for command in commands {
            assert_eq!(recipients(&two, command), [armory], "{command:?}");
            assert!(!is_common(&two, command), "{command:?}");
        }
        let shown = panel(
            &two,
            InterfaceState::DefaultState,
            &Conditions::default().with_pool([1, 0, 0, 0]),
        );
        assert_eq!(shown.get(slot(0, 0)).map(|s| s.look), Some(ButtonLook::Group));
        two.cycle(true);
    }
    let one = Selection::new([(id(1), ARMORY)]);
    assert!(is_common(&one, CancelProduction), "the ActiveGroup is all of it");
}
