//! (M5-N7) The Airstrip's panel as plain data: the pool buildings' (`factions/cults/structures/airstrip.md >
//! Placeholder stats`: "Panel like the Armory's (choose a kind with Q/W/E/R first, then; README > Choosing Recruits,
//! Release and Eject All): **Q: Build Sparrow**, **A: Release** in the kind menu; **A: Eject All**, **C: Set Rally
//! Point**, **X: Cancel Production**"). In a game: `crates/game/tests/commands/airstrip.rs`.

use sc_sim::RecruitKind;

use super::cults::counted;
use super::*;

const AIRSTRIP: ObjectKind = ObjectKind::Airstrip;
const SPARROW: ObjectKind = ObjectKind::Sparrow;

/// The Airstrip's DefaultState is a pool building's: the four kinds with their counts of free Recruits inside, A Eject
/// All, X Cancel Production and C Set Rally Point. KindMenu\[kind\] holds its one recipe's training, "**Q: Build
/// Sparrow**" (so named on its panel, `PanelCommand::name_on`, a Train elsewhere), labelled with the Sparrow's "SW",
/// and "**A: Release**", each issued at once; Z goes back. Ungroupable (design change 0348 point 5), a command goes to
/// the one Airstrip Tabbed to.
#[test]
fn the_airstrip_builds_sparrows_from_its_kind_menus() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, Release, SetRallyPoint, TrainFrom};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let airstrip = Selection::new([(id(1), AIRSTRIP)]);
    let conditions = Conditions::default().with_pool([2, 1, 0, 0]);
    let default = panel(&airstrip, InterfaceState::DefaultState, &conditions);
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
    assert_eq!(InterfaceTemplate::of(AIRSTRIP), Some(InterfaceTemplate::PoolProducer));
    assert_eq!(default.name(slot(1, 0)).as_deref(), Some("Eject All"));
    assert_eq!(press(&default, c), SlotPress::AwaitTarget(SetRallyPoint));
    for from in RecruitKind::ALL {
        let menu = panel(&airstrip, InterfaceState::KindMenu(from), &conditions);
        assert_eq!(menu.name(slot(0, 0)).as_deref(), Some("Build Sparrow"), "{from:?}");
    }
    assert_eq!(press(&default, slot(0, 1)), SlotPress::OpenKindMenu(Constitution));
    let shown_menu = panel(&airstrip, InterfaceState::KindMenu(Constitution), &conditions);
    assert_eq!(
        shown(&shown_menu),
        [
            (slot(0, 0), command(TrainFrom(SPARROW, Constitution)), Common),
            (slot(1, 0), command(Release(Constitution)), Common),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let labels = [slot(0, 0), slot(1, 0)].map(|slot| shown_menu.get(slot).unwrap().label());
    assert_eq!(labels, ["SW", "Rel."]);
    assert_eq!(shown_menu.name(slot(1, 0)).as_deref(), Some("Release"));
    assert_eq!(
        press(&shown_menu, slot(0, 0)),
        SlotPress::Produce(
            TrainFrom(SPARROW, Constitution),
            ProductionCommand::TrainFrom {
                kind: SPARROW,
                from: Constitution
            }
        )
    );
    assert_eq!(
        press(&shown_menu, slot(1, 0)),
        SlotPress::Produce(Release(Constitution), ProductionCommand::Release(Constitution))
    );
    assert_eq!(press(&shown_menu, CommandSlot::BACK), SlotPress::Back);
    // Its generic name, a training's: only the Airstrip's panel calls it a Build.
    assert_eq!(TrainFrom(SPARROW, Plain).name(), "Train Sparrow");
    let two = Selection::new([(id(1), AIRSTRIP), (id(2), AIRSTRIP)]);
    assert_eq!(two.groups().len(), 2, "Ungroupable");
    assert_eq!(recipients(&two, TrainFrom(SPARROW, Plain)), [id(1)]);
    assert!(!is_common(&two, CancelProduction));
}
