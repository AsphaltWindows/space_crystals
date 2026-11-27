//! (M5-N6) The Chop Shop's panel as plain data: the pool buildings', as the Airstrip's is
//! (`factions/cults/structures/chop_shop.md > Placeholder stats`: "Panel like the Armory's (choose a kind with Q/W/E/R
//! first, then; README > Choosing Recruits, Release and Eject All): **Q: Build Seeker**, **W: Build Hand of Heaven**,
//! **E: Build Prayer-mobile** (greyed "Requires a Temple" until one stands [...]), **A: Release** in the kind menu;
//! **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**", design change 0444; (M5-N10) W, the Hand of
//! Heaven's; (M5-N11) E, the Prayer-mobile's). In a game: `crates/game/tests/commands/chop_shop.rs`.

use sc_sim::RecruitKind;

use super::cults::counted;
use super::*;
use crate::command_ui::feedback::unavailable_text;

const CHOP_SHOP: ObjectKind = ObjectKind::ChopShop;
const SEEKER: ObjectKind = ObjectKind::Seeker;
const HAND: ObjectKind = ObjectKind::HandOfHeaven;
const PRAYER_MOBILE: ObjectKind = ObjectKind::PrayerMobile;

/// The Chop Shop's DefaultState is a pool building's: the four kinds with their counts of free Recruits inside, A Eject
/// All, X Cancel Production and C Set Rally Point. KindMenu\[kind\] holds its recipes' trainings in their order,
/// "**Q: Build Seeker**", (M5-N10) "**W: Build Hand of Heaven**" and (M5-N11) "**E: Build Prayer-mobile**"
/// ([`e_builds_a_prayer_mobile_once_a_temple_stands`]), so named on its panel (`PanelCommand::name_on`, Trains
/// elsewhere) and labelled with the units' "SKR", "HOH" and "PM", and "**A: Release**", each issued at once; Z goes
/// back. Ungroupable (design change 0348 point 5: "the Chop Shop is Groupable false"), a command goes to the one Chop
/// Shop Tabbed to. The Seeker's panel is the BasicCombatUnit's, its W Reverse available (slice M5-V2: it CanReverse),
/// as the Guard's isn't.
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
        assert_eq!(
            menu.name(slot(0, 1)).as_deref(),
            Some("Build Hand of Heaven"),
            "{from:?}"
        );
    }
    assert_eq!(press(&default, slot(0, 2)), SlotPress::OpenKindMenu(Metabolism));
    let shown_menu = panel(&chop_shop, InterfaceState::KindMenu(Metabolism), &conditions);
    assert_eq!(
        shown(&shown_menu),
        [
            (slot(0, 0), command(TrainFrom(SEEKER, Metabolism)), Common),
            (slot(0, 1), command(TrainFrom(HAND, Metabolism)), Common),
            (slot(0, 2), command(TrainFrom(PRAYER_MOBILE, Metabolism)), Common),
            (slot(1, 0), command(Release(Metabolism)), Common),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let labels = [slot(0, 0), slot(0, 1), slot(0, 2), slot(1, 0)].map(|slot| shown_menu.get(slot).unwrap().label());
    assert_eq!(labels, ["SKR", "HOH", "PM", "Rel."]);
    for (at, kind) in [(slot(0, 0), SEEKER), (slot(0, 1), HAND), (slot(0, 2), PRAYER_MOBILE)] {
        assert_eq!(
            press(&shown_menu, at),
            SlotPress::Produce(
                TrainFrom(kind, Metabolism),
                ProductionCommand::TrainFrom { kind, from: Metabolism }
            )
        );
    }
    assert_eq!(press(&shown_menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(TrainFrom(SEEKER, Plain).name(), "Train Seeker");
    assert_eq!(TrainFrom(HAND, Plain).name(), "Train Hand of Heaven");
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

/// (M5-N11) Design change 0444's Verification: "Chop Shop kind menu with 3+ Recruits inside: Q Seeker, W Hand of
/// Heaven, E Prayer-mobile greyed until a Temple exists" (`factions/cults/structures/chop_shop.md > Production`:
/// "**Prayer-mobile** (wheeled caster; 2 Recruits; requires a Temple)"). With three Plain Recruits free inside and no
/// Temple, KindMenu\[kind\]'s Q is the Seeker's training and W the Hand of Heaven's (slice M5-N10), both lit, and E the
/// Prayer-mobile's, named "Build Prayer-mobile" in every kind's menu and labelled with its "PM", greyed as the gate
/// refuses it (`Reject::Requires`), saying "Requires a Temple" and pressing nothing; Q builds a Seeker meanwhile. Once
/// a Temple stands, E is lit and builds a Prayer-mobile of a Recruit of the kind at once.
#[test]
fn e_builds_a_prayer_mobile_once_a_temple_stands() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::TrainFrom;
    use RecruitKind::Plain;
    let chop_shop = Selection::new([(id(1), CHOP_SHOP)]);
    let e = slot(0, 2);
    let pool = Conditions::default().with_pool([3, 0, 0, 0]);
    let no_temple = pool
        .clone()
        .refusing(TrainFrom(PRAYER_MOBILE, Plain), Reject::Requires(ObjectKind::Temple));
    let menu = panel(&chop_shop, InterfaceState::KindMenu(Plain), &no_temple);
    let command = PanelButton::Command;
    let look = |panel: &Panel, at| panel.get(at).map(|shown| (shown.button, shown.look));
    assert_eq!(
        look(&menu, slot(0, 0)),
        Some((command(TrainFrom(SEEKER, Plain)), Common))
    );
    assert_eq!(
        look(&menu, e),
        Some((command(TrainFrom(PRAYER_MOBILE, Plain)), Disabled))
    );
    assert_eq!(
        look(&menu, slot(0, 1)),
        Some((command(TrainFrom(HAND, Plain)), Common)),
        "W: the Hand of Heaven's, of three Recruits"
    );
    assert_eq!(menu.get(e).unwrap().label(), "PM");
    for from in RecruitKind::ALL {
        let menu = panel(&chop_shop, InterfaceState::KindMenu(from), &pool);
        assert_eq!(menu.name(e).as_deref(), Some("Build Prayer-mobile"), "{from:?}");
    }
    let why = menu.get(e).and_then(|shown| shown.why);
    assert_eq!(why, Some(WhyUnavailable::Refused(Reject::Requires(ObjectKind::Temple))));
    assert_eq!(why.map(unavailable_text), Some("Requires a Temple"));
    assert_eq!(press(&menu, e), SlotPress::Nothing);
    assert!(matches!(press(&menu, slot(0, 0)), SlotPress::Produce(..)));

    let with_temple = panel(&chop_shop, InterfaceState::KindMenu(Plain), &pool);
    assert_eq!(
        look(&with_temple, e),
        Some((command(TrainFrom(PRAYER_MOBILE, Plain)), Common))
    );
    assert_eq!(
        press(&with_temple, e),
        SlotPress::Produce(
            TrainFrom(PRAYER_MOBILE, Plain),
            ProductionCommand::TrainFrom {
                kind: PRAYER_MOBILE,
                from: Plain
            }
        )
    );
    assert!(offers(CHOP_SHOP, TrainFrom(PRAYER_MOBILE, Plain)));
    assert_eq!(TrainFrom(PRAYER_MOBILE, Plain).name(), "Train Prayer-mobile");
}

/// (M5-N10) The Hand of Heaven's panel is the BasicCombatUnit's (`factions/cults/units/hand_of_heaven.md > Placeholder
/// stats`: "ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **CanTargetGround** (Attack Ground), as every
/// attack that can miss"): its DoublyDisjointed missile can target the ground, so D, Attack Ground, is available and
/// awaits its ground point, as the Seeker's machine gun and the Guard's rifle can't; and it CanReverse, a
/// TrackedVehicle (slice M5-V2), so W, Reverse, is too.
#[test]
fn the_hand_of_heaven_has_attack_ground_and_reverse() {
    use ButtonLook::Common;
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    assert_eq!(InterfaceTemplate::of(HAND), Some(InterfaceTemplate::BasicCombatUnit));
    assert!(offers(HAND, AttackGround) && offers(HAND, Reverse));
    assert!(!offers(SEEKER, AttackGround) && !offers(GUARD, AttackGround));
    let hand = Selection::new([(id(1), HAND)]);
    let shown_hand = panel(&hand, InterfaceState::DefaultState, &Conditions::default());
    let command = PanelButton::Command;
    assert_eq!(
        shown(&shown_hand),
        [
            (slot(0, 0), command(Move), Common),
            (slot(0, 1), command(Reverse), Common),
            (slot(0, 2), command(HoldPosition), Common),
            (slot(1, 0), command(Attack), Common),
            (slot(1, 1), command(Patrol), Common),
            (slot(1, 2), command(AttackGround), Common),
            (slot(2, 1), command(Stop), Common),
        ]
    );
    assert_eq!(press(&shown_hand, slot(1, 2)), SlotPress::AwaitTarget(AttackGround));
    assert_eq!(press(&shown_hand, slot(0, 1)), SlotPress::AwaitTarget(Reverse));
}
