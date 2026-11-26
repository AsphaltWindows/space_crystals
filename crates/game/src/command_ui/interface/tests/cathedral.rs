//! (M5-N4a) The Cathedral's and the Disciple's panels as plain data. The Cathedral's is the pool buildings'
//! (`factions/cults/structures/cathedral.md > Placeholder stats`: "Panel like the Armory's (choose a kind with Q/W/E/R
//! first, then; README > Choosing Recruits, Release and Eject All): **Q: Train Disciple**, **A: Release** in the kind
//! menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**"); the Disciple's the BasicCombatUnit's,
//! its attack commands greyed or not shown for want of a weapon, and its R, Shroud (`factions/cults/units/disciple.md >
//! Placeholder stats`: "ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **R: Shroud**"; slice M5-N4b). Both
//! in a game: `crates/game/tests/commands/cathedral.rs`.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, PlayerId, RecruitKind, SpawnAt};

use super::cults::counted;
use super::*;
use crate::command_ui::feedback::{NOT_ENOUGH_ENERGY_TEXT, NOT_FOR_TYPE_TEXT, unavailable_text};
use crate::command_ui::resolution::{UnitCapabilities, unit_target_order};

const CATHEDRAL: ObjectKind = ObjectKind::Cathedral;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;
const MILITANT: ObjectKind = ObjectKind::Militant;

/// The Cathedral's DefaultState is a pool building's, the Armory's (`super::cults`): the four kinds with their counts of
/// free Recruits inside, A Eject All, X Cancel Production and C Set Rally Point. KindMenu\[kind\] holds its one recipe's
/// training, "**Q: Train Disciple**", labelled with the Disciple's "DE", and "**A: Release**", each issued at once; Z
/// goes back.
#[test]
fn the_cathedral_trains_disciples_from_its_kind_menus() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{CancelProduction, EjectAll, KindMenu, Release, SetRallyPoint, TrainFrom};
    use RecruitKind::{Both, Constitution, Metabolism, Plain};
    let cathedral = Selection::new([(id(1), CATHEDRAL)]);
    let conditions = Conditions::default().with_pool([1, 0, 0, 2]);
    let default = panel(&cathedral, InterfaceState::DefaultState, &conditions);
    let command = PanelButton::Command;
    let (x, c) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        counted(&default),
        [
            (slot(0, 0), command(KindMenu(Plain)), Common, Some(1)),
            (slot(0, 1), command(KindMenu(Constitution)), Disabled, Some(0)),
            (slot(0, 2), command(KindMenu(Metabolism)), Disabled, Some(0)),
            (slot(0, 3), command(KindMenu(Both)), Common, Some(2)),
            (slot(1, 0), command(EjectAll), Common, None),
            (x, command(CancelProduction), Common, None),
            (c, command(SetRallyPoint), Common, None),
        ]
    );
    assert_eq!(InterfaceTemplate::of(CATHEDRAL), Some(InterfaceTemplate::PoolProducer));
    assert_eq!(press(&default, slot(0, 3)), SlotPress::OpenKindMenu(Both));
    let menu = InterfaceState::KindMenu(Both);
    let shown_menu = panel(&cathedral, menu, &conditions);
    assert_eq!(
        shown(&shown_menu),
        [
            (slot(0, 0), command(TrainFrom(DISCIPLE, Both)), Common),
            (slot(1, 0), command(Release(Both)), Common),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let labels = [slot(0, 0), slot(1, 0)].map(|slot| shown_menu.get(slot).unwrap().label());
    assert_eq!(labels, ["DE", "Rel."]);
    assert_eq!(shown_menu.name(slot(0, 0)).as_deref(), Some("Train Disciple"));
    assert_eq!(
        press(&shown_menu, slot(0, 0)),
        SlotPress::Produce(
            TrainFrom(DISCIPLE, Both),
            ProductionCommand::TrainFrom {
                kind: DISCIPLE,
                from: Both
            }
        )
    );
    assert_eq!(
        press(&shown_menu, slot(1, 0)),
        SlotPress::Produce(Release(Both), ProductionCommand::Release(Both))
    );
    assert_eq!(press(&shown_menu, CommandSlot::BACK), SlotPress::Back);
    // Design change 0348 point 5: Ungroupable, so a command goes to the Cathedral Tabbed to.
    let two = Selection::new([(id(1), CATHEDRAL), (id(2), CATHEDRAL)]);
    assert_eq!(recipients(&two, TrainFrom(DISCIPLE, Plain)), [id(1)]);
    assert!(!is_common(&two, CancelProduction));
}

/// `factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState: BasicCombatUnitInterfaceState, plus
/// **R: Shroud**", read from a game with two Disciples. Q Move, E Hold Position, S Patrol and X Stop are its own; W
/// Reverse and D Attack Ground are unavailable, as an infantry's without a ground attack are; A Attack isn't shown: it
/// has no weapon ("AttackAttributes - None"), the gate refusing its Attack (`sc_sim::combat::takes_attack_orders`,
/// plan decision T10), and no selected unit can attack (design change 0399: "Only when **no** selected unit can attack
/// is A not shown"). R is Shroud (slice M5-N4b), labelled with its name, a
/// Shroud order at once. The Disciple is Ungroupable ("Groupable - false (user, 2026-11-26; Q0112: each R press starts
/// one Disciple's channel; Tab to the next)"): two are two groups, a common command goes to both, and R to the one of
/// the ActiveGroup alone. Below 10 energy R is greyed, "Not enough energy", as the gate refuses it
/// (`sc_sim::Reject::NotEnoughEnergy`: "needs at least **10 energy** to start"): after a channel that ran the
/// Disciple's 50 dry.
#[test]
fn the_disciple_greys_its_attack_commands_and_shrouds_with_r() {
    use ButtonLook::{Common, Disabled, Group, Unavailable};
    use PanelCommand::{AttackGround, HoldPosition, Move, Patrol, Reverse, Shroud, Stop};
    let player = PlayerId(1);
    let mut t = ScenarioBuilder::new(&vec![".".repeat(16); 16].join("\n"))
        .faction(player, Faction::Cults)
        .spawn(
            DISCIPLE,
            Some(player),
            SpawnAt::Point(bevy::math::Vec2::new(300.0, 300.0)),
        )
        .spawn(
            DISCIPLE,
            Some(player),
            SpawnAt::Point(bevy::math::Vec2::new(700.0, 300.0)),
        )
        .start();
    let one = Selection::new([(id(1), DISCIPLE)]);
    let conditions = Conditions::of(t.sim(), player, &one);
    let default = panel(&one, InterfaceState::DefaultState, &conditions);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&default),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Unavailable),
            (slot(0, 2), c(HoldPosition), Common),
            (slot(0, 3), c(Shroud), Common),
            (slot(1, 1), c(Patrol), Common),
            (slot(1, 2), c(AttackGround), Unavailable),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    let why = |panel: &Panel, at| panel.get(at).and_then(|shown| shown.why).map(unavailable_text);
    assert_eq!(default.get(slot(1, 0)), None, "no Attack, without a weapon");
    assert_eq!(why(&default, slot(1, 2)), Some(NOT_FOR_TYPE_TEXT), "Attack Ground");
    assert_eq!(why(&default, slot(0, 3)), None, "Shroud, with its 50 energy");
    let shroud = default.get(slot(0, 3)).unwrap();
    assert_eq!(shroud.label(), "Shroud");
    assert_eq!(default.name(slot(0, 3)).as_deref(), Some("Shroud"));
    assert_eq!(press(&default, slot(1, 0)), SlotPress::Nothing);
    assert_eq!(press(&default, slot(0, 3)), SlotPress::Issue(Shroud, OrderKind::Shroud));
    assert_eq!(press(&default, slot(1, 1)), SlotPress::AwaitTarget(Patrol));
    assert!(Shroud.queues() && Shroud.instance_only() && !Shroud.needs_target());
    let two = Selection::new([(id(1), DISCIPLE), (id(2), DISCIPLE)]);
    assert_eq!(two.groups().len(), 2, "Ungroupable");
    assert!(is_common(&two, Move));
    assert_eq!(recipients(&two, Stop), [id(1), id(2)]);
    assert!(!is_common(&two, Shroud));
    assert_eq!(recipients(&two, Shroud), [id(1)], "one Disciple a press");
    let both = panel(
        &two,
        InterfaceState::DefaultState,
        &Conditions::of(t.sim(), player, &two),
    );
    assert_eq!(both.get(slot(0, 3)).map(|shown| shown.look), Some(Group));
    let mut tabbed = two.clone();
    tabbed.cycle(true);
    assert_eq!(recipients(&tabbed, Shroud), [id(2)], "Tab to the next");
    t.order_at(1, player, &[id(1)], OrderKind::Shroud, false);
    t.run(180);
    let dry = Conditions::of(t.sim(), player, &one);
    let default = panel(&one, InterfaceState::DefaultState, &dry);
    assert_eq!(default.get(slot(0, 3)).map(|shown| shown.look), Some(Disabled));
    assert_eq!(why(&default, slot(0, 3)), Some(NOT_ENOUGH_ENERGY_TEXT));
    assert_eq!(press(&default, slot(0, 3)), SlotPress::Nothing);
    let other = Conditions::of(t.sim(), player, &tabbed);
    assert_eq!(other.unavailable(&tabbed, Shroud), None, "the other one has its energy");
}

/// A Disciple among armed units (`systems/controls/interface.md > CommandPanel`, design changes 0397 to 0399): beside a
/// Militant, A Attack shows with either in focus, the Disciple's group too, and goes to both; an explicit Attack on an
/// object sends the Disciple, which can't attack, to the object, a Move to it ("on a target they move to it (as a Move
/// on that object)", `unit_target_order`), and an AttackMove takes it along, which plays out as a move for it ("on
/// ground they move to the spot (with the group, as for Attack Move)", design change 0076). Alone, or beside a
/// Peacekeeper (Shielded) only, no selected unit can attack, and no panel of theirs shows A ("Only when **no**
/// selected unit can attack is A not shown").
#[test]
fn an_attack_sends_the_disciple_to_its_target_and_an_attack_move_takes_it_along() {
    use sc_sim::Target;
    let mut mixed = Selection::new([(id(1), DISCIPLE), (id(2), MILITANT)]);
    let modes = Modes::default().with_unarmed([id(1)]);
    let conditions = Conditions::with_modes(modes.clone());
    assert!(conditions.attacks(&mixed));
    assert!(conditions.allows(&mixed, PanelCommand::Attack));
    for _ in 0..2 {
        let shown = panel(&mixed, InterfaceState::DefaultState, &conditions);
        let a = shown.get(slot(1, 0)).map(|a| (a.button, a.look));
        let attack = (PanelButton::Command(PanelCommand::Attack), ButtonLook::Common);
        assert_eq!(a, Some(attack), "{:?} in focus", mixed.active().map(|g| g.kind));
        assert_eq!(press(&shown, slot(1, 0)), SlotPress::AwaitTarget(PanelCommand::Attack));
        assert_eq!(recipients(&mixed, PanelCommand::Attack), [id(1), id(2)]);
        mixed.cycle(true);
    }
    let disciple = UnitCapabilities::of(id(1), DISCIPLE, &modes);
    let militant = UnitCapabilities::of(id(2), MILITANT, &modes);
    let attack = OrderKind::Attack(id(9));
    let to_it = OrderKind::Move(Target::Object(id(9)));
    assert_eq!(unit_target_order(disciple, attack), to_it, "moves to it");
    assert_eq!(unit_target_order(militant, attack), attack);
    let attack_move = OrderKind::AttackMove(bevy::math::Vec2::new(100.0, 100.0));
    assert_eq!(unit_target_order(disciple, attack_move), attack_move);
    let disciples = Selection::new([(id(1), DISCIPLE)]);
    assert!(!conditions.attacks(&disciples));
    let alone = panel(&disciples, InterfaceState::DefaultState, &conditions);
    assert_eq!(alone.get(slot(1, 0)), None, "no A");
    let mut shielded = Selection::new([(id(1), DISCIPLE), (id(3), ObjectKind::PeacekeeperShielded)]);
    let modes = Modes::from_iter([(id(3), ShieldMode::Raised)]).with_unarmed([id(1)]);
    let conditions = Conditions::with_modes(modes);
    assert!(!conditions.attacks(&shielded));
    for _ in 0..2 {
        let shown = panel(&shielded, InterfaceState::DefaultState, &conditions);
        assert_eq!(
            shown.get(slot(1, 0)),
            None,
            "{:?}: no A",
            shielded.active().map(|g| g.kind)
        );
        shielded.cycle(true);
    }
}
