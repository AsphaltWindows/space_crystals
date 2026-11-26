//! The Cults' panels as plain data (slice M5-C): the Recruitment Hall's, read from a game, which has no X (design change
//! 0342); the Armory's (slice M5-F2, design change 0337): a kind of Recruit first, then its KindMenu, each command to
//! the one Armory Tabbed to (design change 0348 point 5); the Gunworks', the Machine Shop's and the Scrap Hangar's,
//! empty (slice M5-G2); and the Garrison's, its Eject All (slice M5-N2). The Armory's panel in a game, its counts and
//! its presses, is `crates/game/tests/commands/armory.rs`'s.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, HoldCommand, PlayerId, RecruitKind, SpawnAt, TilePos};

use super::*;
use crate::command_ui::feedback::unavailable_text;
use crate::selection::GroupType;

const HALL: ObjectKind = ObjectKind::RecruitmentHall;
const ARMORY: ObjectKind = ObjectKind::Armory;
const MILITANT: ObjectKind = ObjectKind::Militant;
const GUNNER: ObjectKind = ObjectKind::Gunner;

/// The buttons of `panel` with their counts (the Altar's tests' too, `super::altar`).
pub(super) fn counted(panel: &Panel) -> Vec<(CommandSlot, PanelButton, ButtonLook, Option<u32>)> {
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
/// they can be repeated." Q trains a Militant of a Recruit of the kind and A releases one, both at once; W trains a
/// Gunner (slice M5-G2), greyed while the gate refuses it for want of a Gunworks
/// (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`: "**W: Train Gunner** [...] and the player
/// owns at least one Gunworks (TechPrerequisite)": `Reject::Requires`), saying so: "Requires a Gunworks". The menu
/// stays open while no Recruit of its kind is left (its entries greyed by the gate then), and Z goes back to
/// DefaultState.
#[test]
fn a_kind_menu_trains_and_releases_recruits_of_its_kind() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{Release, TrainFrom};
    use RecruitKind::Metabolism;
    let armory = Selection::new([(id(1), ARMORY)]);
    let conditions = (Conditions::default().with_pool([0, 0, 1, 0]))
        .refusing(TrainFrom(GUNNER, Metabolism), Reject::Requires(ObjectKind::Gunworks));
    let menu = InterfaceState::KindMenu(Metabolism);
    let shown_menu = panel(&armory, menu, &conditions);
    let command = PanelButton::Command;
    assert_eq!(
        shown(&shown_menu),
        [
            (slot(0, 0), command(TrainFrom(MILITANT, Metabolism)), Common),
            (slot(0, 1), command(TrainFrom(GUNNER, Metabolism)), Disabled),
            (slot(1, 0), command(Release(Metabolism)), Common),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let labels = [slot(0, 0), slot(0, 1), slot(1, 0)].map(|slot| shown_menu.get(slot).unwrap().label());
    assert_eq!(labels, ["MT", "GU", "Rel."]);
    assert_eq!(shown_menu.name(slot(0, 1)).as_deref(), Some("Train Gunner"));
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
    assert_eq!(
        why,
        Some(WhyUnavailable::Refused(Reject::Requires(ObjectKind::Gunworks)))
    );
    assert_eq!(why.map(unavailable_text), Some("Requires a Gunworks"));
    assert_eq!(press(&shown_menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(menu.back(&armory), InterfaceState::DefaultState);

    // A Gunworks owned: W trains a Gunner of a Recruit of the kind.
    let gunworks = Conditions::default().with_pool([0, 0, 1, 0]);
    let with_gunworks = panel(&armory, menu, &gunworks);
    let w = with_gunworks.get(slot(0, 1)).map(|s| (s.button, s.look, s.why));
    assert_eq!(w, Some((command(TrainFrom(GUNNER, Metabolism)), Common, None)));
    assert_eq!(
        press(&with_gunworks, slot(0, 1)),
        SlotPress::Produce(
            TrainFrom(GUNNER, Metabolism),
            ProductionCommand::TrainFrom {
                kind: GUNNER,
                from: Metabolism
            }
        )
    );

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

/// Slice M5-G2: the Gunworks and the Machine Shop "Until its research is designed ... [have] no panel commands"
/// (`factions/cults/structures/gunworks.md`, `factions/cults/structures/machine_shop.md`), and the Scrap Hangar "No
/// panel commands (it only unlocks)" (`factions/cults/structures/scrap_hangar.md`): one of each, complete and selected,
/// shows an empty panel, alone or together.
#[test]
fn the_tech_buildings_have_no_commands() {
    let player = PlayerId(1);
    let kinds = [ObjectKind::Gunworks, ObjectKind::MachineShop, ObjectKind::ScrapHangar];
    let mut builder = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n")).faction(player, Faction::Cults);
    for (x, kind) in [2, 7, 12].into_iter().zip(kinds) {
        builder = builder.spawn(kind, Some(player), SpawnAt::Tile(TilePos::new(x, 4)));
    }
    let t = builder.start();
    for (n, kind) in (1..).zip(kinds) {
        assert_eq!(t.sim().object(id(n)).map(|o| o.kind), Some(kind));
        let one = Selection::new([(id(n), kind)]);
        let shown = panel(
            &one,
            InterfaceState::DefaultState,
            &Conditions::of(t.sim(), player, &one),
        );
        assert!(shown.is_empty(), "{kind:?}: {:?}", self::shown(&shown));
        assert_eq!(InterfaceTemplate::of(kind), None, "{kind:?}");
    }
    let all = Selection::new([(id(1), kinds[0]), (id(2), kinds[1]), (id(3), kinds[2])]);
    let shown = panel(
        &all,
        InterfaceState::DefaultState,
        &Conditions::of(t.sim(), player, &all),
    );
    assert!(shown.is_empty());
}

/// `factions/cults/structures/garrison.md > Placeholder stats > Panel` (slice M5-N2): "**A: Eject All** (everyone
/// inside leaves); **X: Cancel construction** while being built". A Garrison shows A alone, which lets everyone out at
/// once, greyed by the gate's refusal while it is empty (`Reject::NothingInside`); it is Groupable (its placeholder
/// stats: "Groupable true"), so two Garrisons both take it, a common command, unlike a pool building's. Its site shows
/// X Cancel Construction alone, as every site does.
#[test]
fn the_garrison_shows_eject_all_and_its_site_its_cancel() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelConstruction, EjectAll};
    const GARRISON: ObjectKind = ObjectKind::Garrison;
    let one = Selection::new([(id(1), GARRISON)]);
    let default = panel(&one, InterfaceState::DefaultState, &Conditions::default());
    let command = PanelButton::Command;
    assert_eq!(shown(&default), [(slot(1, 0), command(EjectAll), Common)]);
    assert_eq!(default.get(slot(1, 0)).map(|shown| shown.label()), Some("Eject"));
    assert_eq!(default.name(slot(1, 0)).as_deref(), Some("Eject All"));
    assert_eq!(
        press(&default, slot(1, 0)),
        SlotPress::Hold(EjectAll, HoldCommand::EjectAll)
    );
    let empty = Conditions::default().refusing(EjectAll, Reject::NothingInside);
    let shown_empty = panel(&one, InterfaceState::DefaultState, &empty);
    assert_eq!(shown_empty.get(slot(1, 0)).map(|s| s.look), Some(Disabled));
    let two = Selection::new([(id(1), GARRISON), (id(2), GARRISON)]);
    assert!(is_common(&two, EjectAll));
    assert_eq!(recipients(&two, EjectAll), [id(1), id(2)]);
    let site = Selection::new([(id(3), GroupType::unfinished(GARRISON))]);
    let on_site = panel(&site, InterfaceState::DefaultState, &Conditions::default().on_site());
    assert_eq!(shown(&on_site), [(slot(2, 1), command(CancelConstruction), Common)]);
    assert_eq!(InterfaceTemplate::of(GARRISON), Some(InterfaceTemplate::Garrison));
}
