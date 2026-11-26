//! The Altar's panel as plain data (slice M5-N1b, `factions/cults/augmentation.md > Altar >
//! ObjectInterfaceState[Altar]`): "a three-stage selection: **unit type → current AugmentationState → Augmentation to
//! apply**", Z back at every stage, each command to the one Altar Tabbed to (design change 0348 point 5), and its
//! counts read from a game. The panel in a game, its keys and its InfoPanel, is `crates/game/tests/commands/altar.rs`'s;
//! the augmenting itself the sim's (`crates/sim/tests/altar`).

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    AugmentOrder, Augmentation, Command, DebugCommand, Faction, HoldCommand, PlayerId, RecruitKind, SpawnAt, TilePos,
};

use super::cults::counted;
use super::*;
use crate::command_ui::feedback::{NO_CRYSTALS_TEXT, NO_RECRUIT_TEXT, QUEUE_FULL_TEXT, unavailable_text};

const ALTAR: ObjectKind = ObjectKind::Altar;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const GUNNER: ObjectKind = ObjectKind::Gunner;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;

/// `> ObjectInterfaceState[Altar] > DefaultState`: "Unit type tiles (StateOnlyTransition to AugmentMenu[UnitType]) —
/// each tile shows the count of that unit type in StoredUnits; disabled if there are none: **Q: Recruit**, **W:
/// Militant**, **E: Gunner**, **R: Disciple**", then "**A: Eject All**", "**X: Cancel Production**" and "**C: Set
/// Rally Point**". A type with units opens its AugmentMenu; one with none is greyed, saying so; A ejects at once, X
/// cancels at once and C awaits the rally point. A tile shows its type's tag and is named in full.
#[test]
fn the_altar_picks_a_unit_type_first() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{AugmentMenu, CancelProduction, EjectAll, SetRallyPoint};
    let altar = Selection::new([(id(1), ALTAR)]);
    let conditions = (Conditions::default())
        .with_stored(RECRUIT, [1, 0, 0, 0])
        .with_stored(MILITANT, [2, 1, 0, 0])
        .with_stored(DISCIPLE, [0, 0, 0, 1]);
    let default = panel(&altar, InterfaceState::DefaultState, &conditions);
    let command = PanelButton::Command;
    let (q, w, e, r, a) = (slot(0, 0), slot(0, 1), slot(0, 2), slot(0, 3), slot(1, 0));
    let (x, c) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        counted(&default),
        [
            (q, command(AugmentMenu(RECRUIT)), Common, Some(1)),
            (w, command(AugmentMenu(MILITANT)), Common, Some(3)),
            (e, command(AugmentMenu(GUNNER)), Disabled, Some(0)),
            (r, command(AugmentMenu(DISCIPLE)), Common, Some(1)),
            (a, command(EjectAll), Common, None),
            (x, command(CancelProduction), Common, None),
            (c, command(SetRallyPoint), Common, None),
        ]
    );
    let labels = [q, w, e, r].map(|slot| default.get(slot).unwrap().label());
    assert_eq!(labels, ["RT", "MT", "GU", "DE"]);
    let names = [q, w, e, r].map(|slot| default.name(slot).unwrap());
    assert_eq!(names, ["Recruit", "Militant", "Gunner", "Disciple"]);
    assert_eq!(press(&default, w), SlotPress::OpenAugmentMenu(MILITANT));
    assert_eq!(press(&default, e), SlotPress::Nothing);
    let why = default.get(e).and_then(|shown| shown.why);
    assert_eq!(why.map(unavailable_text), Some(NO_RECRUIT_TEXT));
    assert_eq!(press(&default, a), SlotPress::Hold(EjectAll, HoldCommand::EjectAll));
    assert_eq!(
        press(&default, x),
        SlotPress::Produce(CancelProduction, ProductionCommand::CancelProduction)
    );
    assert_eq!(press(&default, c), SlotPress::AwaitTarget(SetRallyPoint));
    assert_eq!(InterfaceTemplate::of(ALTAR), Some(InterfaceTemplate::Altar));
}

/// `> ObjectInterfaceState[Altar] > AugmentMenu[UnitType]`: "StateOnlyTransition to AugmentMenu[UnitType, State]. Each
/// tile shows the count of unreserved units of that type and state in StoredUnits; disabled if there are none: **Q:
/// Unaugmented**, **W: Enhanced Constitution only**, **E: Improved Metabolism only**, **R: Fully augmented** — enabled
/// when there are any (only Release is possible); **Z**: returns to DefaultState (StateOnlyTransition)". The menu stays
/// open while its type's units change, none left included, and ends with another selection.
#[test]
fn an_augment_menu_shows_the_four_states_of_its_type() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::AugmentState;
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let altar = Selection::new([(id(1), ALTAR)]);
    let conditions = Conditions::default().with_stored(MILITANT, [2, 1, 0, 0]);
    let menu = InterfaceState::AugmentMenu(MILITANT);
    let shown_menu = panel(&altar, menu, &conditions);
    let command = PanelButton::Command;
    let (q, w, e, r) = (slot(0, 0), slot(0, 1), slot(0, 2), slot(0, 3));
    assert_eq!(
        counted(&shown_menu),
        [
            (q, command(AugmentState(MILITANT, Plain)), Common, Some(2)),
            (w, command(AugmentState(MILITANT, Constitution)), Common, Some(1)),
            (e, command(AugmentState(MILITANT, Metabolism)), Disabled, Some(0)),
            (r, command(AugmentState(MILITANT, Both)), Disabled, Some(0)),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );
    let labels = [q, w, e, r].map(|slot| shown_menu.get(slot).unwrap().label());
    assert_eq!(labels, ["Plain", "Const.", "Metab.", "Both"]);
    let names = [q, w, e, r].map(|slot| shown_menu.name(slot).unwrap());
    let full = [
        "Unaugmented",
        "Enhanced Constitution only",
        "Improved Metabolism only",
        "Fully augmented",
    ];
    assert_eq!(names, full);
    assert_eq!(
        press(&shown_menu, w),
        SlotPress::OpenAugmentStateMenu(MILITANT, Constitution)
    );
    assert_eq!(press(&shown_menu, e), SlotPress::Nothing);
    let why = shown_menu.get(e).and_then(|shown| shown.why);
    assert_eq!(why, Some(WhyUnavailable::Refused(Reject::NoRecruit(Metabolism))));
    assert_eq!(press(&shown_menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(menu.back(&altar), InterfaceState::DefaultState);

    // "R: Fully augmented — enabled when there are any".
    let fully = Conditions::default().with_stored(MILITANT, [0, 0, 0, 1]);
    let shown_fully = panel(&altar, menu, &fully);
    assert_eq!(shown_fully.get(r).map(|s| (s.look, s.count)), Some((Common, Some(1))));
    assert_eq!(press(&shown_fully, r), SlotPress::OpenAugmentStateMenu(MILITANT, Both));

    assert!(still_valid(menu, &altar, true, &Conditions::default()), "none left");
    let armory = Selection::new([(id(2), ObjectKind::Armory)]);
    assert!(!still_valid(menu, &armory, true, &conditions));
    assert!(!still_valid(menu, &altar, false, &conditions), "not commandable");
}

/// `> ObjectInterfaceState[Altar] > AugmentMenu[UnitType, State]`: "Shows only the Augmentations the selected group is
/// missing. Each click queues one order (CommandIssuingTransition) and stays in this menu so further clicks queue more.
/// From Unaugmented: **Q: Enhanced Constitution**, **W: Improved Metabolism**, **E: Both**. From Enhanced Constitution
/// only: **W: Improved Metabolism**. From Improved Metabolism only: **Q: Enhanced Constitution**", and "In every state:
/// **A: Release**" (design change 0337); "**Z**: returns to AugmentMenu[UnitType]". An order and a Release go at once;
/// one the gate refuses is greyed, saying why (`> Augmentation Orders`: "Available only if the group still has an
/// unreserved unit, the AugmentationQueue is not full, and the player has sufficient Space Crystals"); the menu stays
/// open with no unit of its state left.
#[test]
fn a_state_menu_shows_the_orders_its_units_lack_then_release() {
    use AugmentOrder::{EnhancedConstitution, ImprovedMetabolism};
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{Augment, ReleaseOf};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let altar = Selection::new([(id(1), ALTAR)]);
    let command = PanelButton::Command;
    let (q, w, e, a) = (slot(0, 0), slot(0, 1), slot(0, 2), slot(1, 0));
    let back = (CommandSlot::BACK, PanelButton::Back, Back);
    let menu = |state| InterfaceState::AugmentStateMenu(RECRUIT, state);
    let shown_in = |state| shown(&panel(&altar, menu(state), &Conditions::default()));
    let order = |slot, state, order| (slot, command(Augment(RECRUIT, state, order)), Common);
    let release = |state| (a, command(ReleaseOf(RECRUIT, state)), Common);
    assert_eq!(
        shown_in(Plain),
        [
            order(q, Plain, EnhancedConstitution),
            order(w, Plain, ImprovedMetabolism),
            order(e, Plain, AugmentOrder::Both),
            release(Plain),
            back,
        ]
    );
    assert_eq!(
        shown_in(Constitution),
        [order(w, Constitution, ImprovedMetabolism), release(Constitution), back]
    );
    assert_eq!(
        shown_in(Metabolism),
        [order(q, Metabolism, EnhancedConstitution), release(Metabolism), back]
    );
    assert_eq!(shown_in(Both), [release(Both), back], "only Release is possible");

    let plain = panel(&altar, menu(Plain), &Conditions::default());
    let labels = [q, w, e, a].map(|slot| plain.get(slot).unwrap().label());
    assert_eq!(labels, ["Const.", "Metab.", "Both", "Rel."]);
    let names = [q, w, e, a].map(|slot| plain.name(slot).unwrap());
    let full = [
        "Enhanced Constitution",
        "Improved Metabolism",
        "Both Augmentations",
        "Release",
    ];
    assert_eq!(names, full);
    let augment = ProductionCommand::Augment {
        kind: RECRUIT,
        from: Plain,
        order: ImprovedMetabolism,
    };
    assert_eq!(
        press(&plain, w),
        SlotPress::Produce(Augment(RECRUIT, Plain, ImprovedMetabolism), augment)
    );
    let release = ProductionCommand::ReleaseOf {
        kind: RECRUIT,
        from: Plain,
    };
    assert_eq!(press(&plain, a), SlotPress::Produce(ReleaseOf(RECRUIT, Plain), release));
    assert_eq!(press(&plain, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(menu(Plain).back(&altar), InterfaceState::AugmentMenu(RECRUIT));

    // The gate's refusals grey the orders, saying why: a full queue, too few Space Crystals.
    let refused = (Conditions::default())
        .refusing(Augment(RECRUIT, Plain, EnhancedConstitution), Reject::QueueFull)
        .refusing(
            Augment(RECRUIT, Plain, AugmentOrder::Both),
            Reject::InsufficientCrystals,
        );
    let greyed = panel(&altar, menu(Plain), &refused);
    let looks = [q, w, e].map(|slot| greyed.get(slot).map(|shown| shown.look));
    assert_eq!(looks, [Some(Disabled), Some(Common), Some(Disabled)]);
    let why = |slot| (greyed.get(slot)?.why).map(unavailable_text);
    assert_eq!(
        [q, w, e].map(why),
        [Some(QUEUE_FULL_TEXT), None, Some(NO_CRYSTALS_TEXT)]
    );
    assert_eq!(press(&greyed, q), SlotPress::Nothing);

    // Its last unit of the state used: the menu holds, its entries greyed by the gate ("Clicks stay in the third
    // stage").
    let used = (Conditions::default())
        .refusing(Augment(RECRUIT, Plain, ImprovedMetabolism), Reject::NoRecruit(Plain))
        .refusing(ReleaseOf(RECRUIT, Plain), Reject::NoRecruit(Plain));
    assert!(still_valid(menu(Plain), &altar, true, &used));
    let armory = Selection::new([(id(2), ObjectKind::Armory)]);
    assert!(!still_valid(menu(Plain), &armory, true, &used));
}

/// Design change 0348 point 5 (`factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: "Cults
/// production buildings (Armory, Altar, Cathedral, Chop Shop, Airstrip) are Ungroupable: a command goes to one
/// building"): two selected Altars are two SelectionGroups, and every command of the panel goes to the one Tabbed to.
#[test]
fn each_command_goes_to_the_altar_tabbed_to() {
    use PanelCommand::{Augment, AugmentMenu, AugmentState, CancelProduction, EjectAll, ReleaseOf, SetRallyPoint};
    use RecruitKind::Plain;
    let mut two = Selection::new([(id(1), ALTAR), (id(2), ALTAR)]);
    assert_eq!(two.groups().len(), 2);
    let commands = [
        AugmentMenu(MILITANT),
        AugmentState(MILITANT, Plain),
        Augment(MILITANT, Plain, AugmentOrder::EnhancedConstitution),
        ReleaseOf(MILITANT, Plain),
        EjectAll,
        CancelProduction,
        SetRallyPoint,
    ];
    for altar in [id(1), id(2)] {
        assert!(two.in_active_group(altar));
        for command in commands {
            assert_eq!(recipients(&two, command), [altar], "{command:?}");
            assert!(!is_common(&two, command), "{command:?}");
        }
        two.cycle(true);
    }
}

/// The counts are the Altar's free units of each type and state (`Sim::pool_counts_of`), read from a game: two Plain
/// Recruits and a Militant with Enhanced Constitution go in, and an order reserves one Recruit (design change 0337: "the
/// count inside (unreserved)"). Q Recruit shows 1, W Militant 1, E and R are greyed; AugmentMenu\[Recruit\]'s Q
/// shows 1, AugmentMenu\[Militant\]'s W 1. With 30 Space Crystals left, Both (50) is greyed by the gate, the single
/// orders (25 each) not.
#[test]
fn the_counts_are_the_altars_free_units() {
    use ButtonLook::{Common, Disabled};
    use RecruitKind::{Constitution, Plain};
    let player = PlayerId(1);
    let near = |x, y| SpawnAt::Point(TilePos::new(x, y).center());
    let mut t = ScenarioBuilder::new(&vec![".".repeat(30); 22].join("\n"))
        .faction(player, Faction::Cults)
        .spawn(ALTAR, Some(player), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(RECRUIT, Some(player), near(17, 10))
        .spawn(RECRUIT, Some(player), near(17, 11))
        .spawn(MILITANT, Some(player), near(18, 10))
        .start();
    let units = [id(2), id(3), id(4)];
    let augmentation = Augmentation::of(Constitution);
    t.debug_at(
        1,
        DebugCommand::SetAugmentation {
            id: id(4),
            augmentation,
        },
    );
    let bank = DebugCommand::SetResources {
        player,
        crystals: 55,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    t.order_at(1, player, &units, sc_sim::OrderKind::Enter(id(1)), false);
    for _ in 0..400 {
        if t.sim().held_in(id(1)).len() == units.len() {
            break;
        }
        t.run(1);
    }
    assert_eq!(t.sim().held_in(id(1)).len(), units.len(), "all inside");
    let now = t.sim().tick() + 1;
    let order = ProductionCommand::Augment {
        kind: RECRUIT,
        from: Plain,
        order: AugmentOrder::EnhancedConstitution,
    };
    let objects = vec![id(1)];
    t.at(
        now,
        player,
        Command::Production {
            objects,
            command: order,
        },
    );
    t.run_until(now);
    assert_eq!(t.sim().pool_counts_of(id(1), RECRUIT), [1, 0, 0, 0], "one reserved");

    let altar = Selection::new([(id(1), ALTAR)]);
    let conditions = Conditions::of(t.sim(), player, &altar);
    let default = panel(&altar, InterfaceState::DefaultState, &conditions);
    let tiles = [slot(0, 0), slot(0, 1), slot(0, 2), slot(0, 3)];
    let shown_tile = |panel: &Panel, slot| panel.get(slot).map(|shown| (shown.look, shown.count));
    assert_eq!(
        tiles.map(|slot| shown_tile(&default, slot)),
        [(Common, 1), (Common, 1), (Disabled, 0), (Disabled, 0)].map(|(look, n)| Some((look, Some(n))))
    );
    let recruits = panel(&altar, InterfaceState::AugmentMenu(RECRUIT), &conditions);
    let militants = panel(&altar, InterfaceState::AugmentMenu(MILITANT), &conditions);
    let counts = |panel: &Panel| tiles.map(|slot| panel.get(slot).and_then(|shown| shown.count));
    assert_eq!(counts(&recruits), [1, 0, 0, 0].map(Some));
    assert_eq!(counts(&militants), [0, 1, 0, 0].map(Some));

    let orders = panel(&altar, InterfaceState::AugmentStateMenu(RECRUIT, Plain), &conditions);
    let looks = [slot(0, 0), slot(0, 1), slot(0, 2), slot(1, 0)].map(|slot| orders.get(slot).map(|s| s.look));
    assert_eq!(looks, [Common, Common, Disabled, Common].map(Some));
    let why = orders.get(slot(0, 2)).and_then(|shown| shown.why);
    assert_eq!(why.map(unavailable_text), Some(NO_CRYSTALS_TEXT));
}
