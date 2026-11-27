//! (M5-N6) The Chop Shop's panel as plain data: the pool buildings', as the Airstrip's is
//! (`factions/cults/structures/chop_shop.md > Placeholder stats`: "Panel like the Armory's (choose a kind with Q/W/E/R
//! first, then; README > Choosing Recruits, Release and Eject All): **Q: Build Seeker**, **A: Release** in the kind
//! menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**"). In a game:
//! `crates/game/tests/commands/chop_shop.rs`.

use sc_sim::RecruitKind;

use super::cults::counted;
use super::*;

const CHOP_SHOP: ObjectKind = ObjectKind::ChopShop;
const SEEKER: ObjectKind = ObjectKind::Seeker;

/// The Chop Shop's DefaultState is a pool building's: the four kinds with their counts of free Recruits inside, A Eject
/// All, X Cancel Production and C Set Rally Point. KindMenu\[kind\] holds its one recipe's training, "**Q: Build
/// Seeker**" (so named on its panel, `PanelCommand::name_on`, a Train elsewhere), labelled with the Seeker's "SKR", and
/// "**A: Release**", each issued at once; Z goes back. Ungroupable (design change 0348 point 5: "the Chop Shop is
/// Groupable false"), a command goes to the one Chop Shop Tabbed to. The Seeker's panel is the BasicCombatUnit's, its W
/// Reverse available (slice M5-V2: it CanReverse), as the Guard's isn't.
#[test]
fn the_chop_shop_builds_seekers_from_its_kind_menus() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, Release, SetRallyPoint, TrainFrom};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let chop_shop = Selection::new([(id(1), CHOP_SHOP)]);
    let conditions = Conditions::default().with_pool([2, 0, 3, 0]);
    let default = panel(&chop_shop, InterfaceState::DefaultState, &conditions);
    let command = PanelButton::Command;
    let (x, c) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        counted(&default),
        [
            (slot(0, 0), command(KindMenu(Plain)), Common, Some(2)),
            (slot(0, 1), command(KindMenu(Constitution)), Disabled, Some(0)),
            (slot(0, 2), command(KindMenu(Metabolism)), Common, Some(3)),
            (slot(0, 3), command(KindMenu(Both)), Disabled, Some(0)),
            (slot(1, 0), command(EjectAll), Common, None),
            (x, command(CancelProduction), Common, None),
            (c, command(SetRallyPoint), Common, None),
        ]
    );
    assert_eq!(InterfaceTemplate::of(CHOP_SHOP), Some(InterfaceTemplate::PoolProducer));
    assert_eq!(press(&default, c), SlotPress::AwaitTarget(SetRallyPoint));
    for from in RecruitKind::ALL {
        let menu = panel(&chop_shop, InterfaceState::KindMenu(from), &conditions);
        assert_eq!(menu.name(slot(0, 0)).as_deref(), Some("Build Seeker"), "{from:?}");
    }
    assert_eq!(press(&default, slot(0, 2)), SlotPress::OpenKindMenu(Metabolism));
    let shown_menu = panel(&chop_shop, InterfaceState::KindMenu(Metabolism), &conditions);
    assert_eq!(
        shown(&shown_menu),
        [
            (slot(0, 0), command(TrainFrom(SEEKER, Metabolism)), Common),
            (slot(1, 0), command(Release(Metabolism)), Common),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let labels = [slot(0, 0), slot(1, 0)].map(|slot| shown_menu.get(slot).unwrap().label());
    assert_eq!(labels, ["SKR", "Rel."]);
    assert_eq!(
        press(&shown_menu, slot(0, 0)),
        SlotPress::Produce(
            TrainFrom(SEEKER, Metabolism),
            ProductionCommand::TrainFrom {
                kind: SEEKER,
                from: Metabolism
            }
        )
    );
    assert_eq!(press(&shown_menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(TrainFrom(SEEKER, Plain).name(), "Train Seeker");
    let two = Selection::new([(id(1), CHOP_SHOP), (id(2), CHOP_SHOP)]);
    assert_eq!(two.groups().len(), 2, "Ungroupable");
    assert_eq!(recipients(&two, TrainFrom(SEEKER, Plain)), [id(1)]);
    assert!(!is_common(&two, CancelProduction));

    assert_eq!(InterfaceTemplate::of(SEEKER), Some(InterfaceTemplate::BasicCombatUnit));
    assert!(offers(SEEKER, PanelCommand::Reverse) && !offers(GUARD, PanelCommand::Reverse));
    let seeker = Selection::new([(id(3), SEEKER)]);
    let shown_seeker = panel(&seeker, InterfaceState::DefaultState, &Conditions::default());
    let reverse = shown_seeker.get(slot(0, 1)).expect("W");
    assert_eq!((reverse.button, reverse.look), (command(PanelCommand::Reverse), Common));
    assert_eq!(
        press(&shown_seeker, slot(0, 1)),
        SlotPress::AwaitTarget(PanelCommand::Reverse)
    );
}
