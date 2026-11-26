//! (M5-N4a) The Cathedral's and the Disciple's panels as plain data. The Cathedral's is the pool buildings'
//! (`factions/cults/structures/cathedral.md > Placeholder stats`: "Panel like the Armory's (choose a kind with Q/W/E/R
//! first, then; README > Choosing Recruits, Release and Eject All): **Q: Train Disciple**, **A: Release** in the kind
//! menu; **A: Eject All**, **C: Set Rally Point**, **X: Cancel Production**"); the Disciple's the BasicCombatUnit's,
//! its attack commands greyed for want of a weapon, and its R, Shroud, holding its slot until its slice
//! (`factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState: BasicCombatUnitInterfaceState, plus
//! **R: Shroud**"). Both in a game: `crates/game/tests/commands/cathedral.rs`.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, PlayerId, RecruitKind, SpawnAt};

use super::cults::counted;
use super::*;
use crate::command_ui::feedback::{NOT_FOR_TYPE_TEXT, NOT_YET_TEXT, SHIELDS_UP_TEXT, unavailable_text};
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
/// **R: Shroud**", read from a game with a Disciple. Q Move, E Hold Position, S Patrol and X Stop are its own; W Reverse
/// and D Attack Ground are unavailable, as an infantry's without a ground attack are; A Attack is greyed, "Not
/// available for this unit": it has no weapon ("AttackAttributes - None"), and the gate refuses its Attack
/// (`sc_sim::combat::takes_attack_orders`, plan decision T10); R holds Shroud's slot, "Not available yet" (slice
/// M5-N4b's spell), labelled with its name. The Disciple is Ungroupable ("Groupable - false"): two are two groups, and a
/// common command goes to both.
#[test]
fn the_disciple_greys_its_attack_commands_and_holds_shroud_in_r() {
    use ButtonLook::{Common, Disabled, Unavailable};
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Planned as P, Reverse, Stop};
    let player = PlayerId(1);
    let t = ScenarioBuilder::new(&vec![".".repeat(16); 16].join("\n"))
        .faction(player, Faction::Cults)
        .spawn(
            DISCIPLE,
            Some(player),
            SpawnAt::Point(bevy::math::Vec2::new(300.0, 300.0)),
        )
        .spawn(
            DISCIPLE,
            Some(player),
            SpawnAt::Point(bevy::math::Vec2::new(400.0, 300.0)),
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
            (slot(0, 3), c(P(Planned::Shroud)), Unavailable),
            (slot(1, 0), c(Attack), Disabled),
            (slot(1, 1), c(Patrol), Common),
            (slot(1, 2), c(AttackGround), Unavailable),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    let why = |at| default.get(at).and_then(|shown| shown.why).map(unavailable_text);
    assert_eq!(why(slot(1, 0)), Some(NOT_FOR_TYPE_TEXT), "Attack, without a weapon");
    assert_eq!(why(slot(1, 2)), Some(NOT_FOR_TYPE_TEXT), "Attack Ground");
    assert_eq!(why(slot(0, 3)), Some(NOT_YET_TEXT), "Shroud, slice M5-N4b's");
    let shroud = default.get(slot(0, 3)).unwrap();
    assert_eq!(shroud.label(), "Shroud");
    assert_eq!(default.name(slot(0, 3)).as_deref(), Some("Shroud"));
    assert_eq!(press(&default, slot(1, 0)), SlotPress::Nothing);
    assert_eq!(press(&default, slot(0, 3)), SlotPress::Nothing);
    assert_eq!(press(&default, slot(1, 1)), SlotPress::AwaitTarget(Patrol));
    let two = Selection::new([(id(1), DISCIPLE), (id(2), DISCIPLE)]);
    assert_eq!(two.groups().len(), 2, "Ungroupable");
    assert!(is_common(&two, Move));
    assert_eq!(recipients(&two, Stop), [id(1), id(2)]);
}

/// A Disciple among armed units (design change 0075's rule for the units that take no Attack, `unit_target_order`):
/// beside a Militant, A Attack is available, the Militant's; an explicit Attack on an object leaves the Disciple out,
/// which keeps its orders, as the gate does (`sc_sim::combat::takes_attack_orders`), and an AttackMove takes it along
/// (design change 0076: "it plays out as a move for a unit that can't attack"). Beside a Peacekeeper (Shielded) only,
/// Attack is greyed for the shield: "Lower the Riot Shield first".
#[test]
fn an_attack_leaves_the_disciple_out_and_an_attack_move_takes_it_along() {
    let mixed = Selection::new([(id(1), DISCIPLE), (id(2), MILITANT)]);
    let modes = Modes::default().with_unarmed([id(1)]);
    let conditions = Conditions::with_modes(modes.clone());
    assert!(conditions.allows(&mixed, PanelCommand::Attack));
    assert_eq!(recipients(&mixed, PanelCommand::Attack), [id(1), id(2)]);
    let disciple = UnitCapabilities::of(id(1), DISCIPLE, &modes);
    let militant = UnitCapabilities::of(id(2), MILITANT, &modes);
    let attack = OrderKind::Attack(id(9));
    assert_eq!(unit_target_order(disciple, attack), None, "left out");
    assert_eq!(unit_target_order(militant, attack), Some(attack));
    let attack_move = OrderKind::AttackMove(bevy::math::Vec2::new(100.0, 100.0));
    assert_eq!(unit_target_order(disciple, attack_move), Some(attack_move));
    let disciples = Selection::new([(id(1), DISCIPLE)]);
    let why = conditions.unavailable(&disciples, PanelCommand::Attack);
    assert_eq!(why, Some(WhyUnavailable::NotForThisType));
    let shielded = Selection::new([(id(1), DISCIPLE), (id(3), ObjectKind::PeacekeeperShielded)]);
    let modes = Modes::from_iter([(id(3), ShieldMode::Raised)]).with_unarmed([id(1)]);
    let why = Conditions::with_modes(modes).unavailable(&shielded, PanelCommand::Attack);
    assert_eq!(why.map(unavailable_text), Some(SHIELDS_UP_TEXT));
}
