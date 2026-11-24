mod colonists;
mod construction;
mod gathering;

use sc_sim::defs::AttackType;
use sc_sim::{Ability, ObjectId, ObjectKind, OrderKind, ProductionCommand, Reject, RiotShield};

use super::*;

const PK: ObjectKind = ObjectKind::Peacekeeper;
const GUARD: ObjectKind = ObjectKind::Guard;
const BARRACKS: ObjectKind = ObjectKind::Barracks;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn slot(row: u8, column: u8) -> CommandSlot {
    CommandSlot::new(row, column).unwrap()
}

fn shown(panel: &Panel) -> Vec<(CommandSlot, PanelButton, ButtonLook)> {
    panel
        .slots()
        .filter_map(|(slot, shown)| shown.map(|s| (slot, s.button, s.look)))
        .collect()
}

/// `> BasicCombatUnitInterfaceState > DefaultState commands`: the grid of a Peacekeeper and Guard selection.
/// Neither can reverse (infantry) nor target the ground (FullyConnected), so W and D are unavailable; every other
/// command is common to both types. The Peacekeepers, the ActiveGroup, add their Riot Shield in R, a group
/// command.
#[test]
fn a_mixed_infantry_selection_gets_the_basic_combat_unit_grid() {
    use ButtonLook::{Common, Group, Unavailable};
    use PanelCommand::*;
    let selection = Selection::new([(id(1), PK), (id(2), GUARD)]);
    let panel = panel(&selection, InterfaceState::DefaultState, &Conditions::default());
    let c = PanelButton::Command;
    assert_eq!(
        shown(&panel),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Unavailable),
            (slot(0, 2), c(HoldPosition), Common),
            (slot(0, 3), c(RiotShield), Group),
            (slot(1, 0), c(Attack), Common),
            (slot(1, 1), c(Patrol), Common),
            (slot(1, 2), c(AttackGround), Unavailable),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    // `> Standard Slot Assignments`: Z, C and V stay free for a unit; X is its Stop.
    for free in [CommandSlot::BACK, CommandSlot::RALLY_POINT, slot(2, 3)] {
        assert_eq!(panel.get(free), None);
    }
    assert_eq!(press(&panel, slot(1, 0)), SlotPress::AwaitTarget(Attack));
    assert_eq!(
        press(&panel, slot(0, 2)),
        SlotPress::Issue(HoldPosition, OrderKind::Hold)
    );
    assert_eq!(press(&panel, slot(2, 1)), SlotPress::Issue(Stop, OrderKind::Stop));
    assert_eq!(press(&panel, slot(0, 1)), SlotPress::Nothing, "unavailable");
    assert_eq!(press(&panel, CommandSlot::BACK), SlotPress::Nothing, "empty");
}

/// `> AwaitingTarget`: the panel marks the PendingCommand, Z is Back, and the other commands wait.
#[test]
fn awaiting_a_target_marks_the_pending_command_and_offers_back() {
    let selection = Selection::new([(id(1), PK)]);
    let panel = panel(
        &selection,
        InterfaceState::awaiting(PanelCommand::Attack),
        &Conditions::default(),
    );
    let look = |slot| panel.get(slot).map(|s| s.look);
    assert_eq!(look(slot(1, 0)), Some(ButtonLook::Pending));
    assert_eq!(look(slot(0, 0)), Some(ButtonLook::Disabled));
    assert_eq!(look(slot(0, 1)), Some(ButtonLook::Unavailable));
    assert_eq!(look(CommandSlot::BACK), Some(ButtonLook::Back));
    assert_eq!(press(&panel, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(press(&panel, slot(1, 0)), SlotPress::Nothing, "already pending");
    assert_eq!(press(&panel, slot(0, 2)), SlotPress::Nothing, "waits");
}

/// Resources have no ObjectInterfaceState: a selected patch shows no commands.
#[test]
fn a_resource_has_no_commands() {
    let selection = Selection::new([(id(8), ObjectKind::CrystalPatch)]);
    assert!(panel(&selection, InterfaceState::DefaultState, &Conditions::default()).is_empty());
    assert!(
        panel(
            &Selection::default(),
            InterfaceState::DefaultState,
            &Conditions::default()
        )
        .is_empty()
    );
    assert_eq!(InterfaceTemplate::of(ObjectKind::CrystalPatch), None);
}

/// `> CommandPanel`: a common command goes to the whole selection; a command only the ActiveGroup's type offers
/// goes to the ActiveGroup. A resource stands in for a type that lacks Attack; the Riot Shield, which Guards lack,
/// is `the_riot_shield_toggles_in_slot_r`'s. A right-click leaves an object without commands alone (design change
/// 0062: buildings aren't affected).
#[test]
fn common_commands_go_to_the_selection_and_group_commands_to_the_active_group() {
    use crate::command_ui::resolution::{MapTarget, capabilities, unit_right_click};
    let selection = Selection::new([(id(1), PK), (id(2), GUARD), (id(3), PK)]);
    assert_eq!(
        recipients(&selection, PanelCommand::HoldPosition),
        [id(1), id(2), id(3)]
    );
    let mut mixed = Selection::new([(id(1), PK), (id(3), PK), (id(8), ObjectKind::CrystalPatch)]);
    // The patch's group comes first (`ObjectKind` order): make the Peacekeepers active.
    mixed.cycle(true);
    assert_eq!(mixed.active().map(|k| k.kind), Some(PK));
    let panel = panel(&mixed, InterfaceState::DefaultState, &Conditions::default());
    assert_eq!(panel.get(slot(1, 0)).map(|s| s.look), Some(ButtonLook::Group));
    assert_eq!(recipients(&mixed, PanelCommand::Attack), [id(1), id(3)]);
    let ground = MapTarget::ground(bevy::math::Vec2::new(100.0, 100.0));
    let moved: Vec<ObjectId> = capabilities(&mixed, &Modes::default())
        .into_iter()
        .filter(|(_, unit)| unit_right_click(*unit, &ground).is_some())
        .map(|(id, _)| id)
        .collect();
    assert_eq!(moved, [id(1), id(3)]);
}

/// `> ObjectInterfaceState`: AwaitingTarget holds while its command is still offered and the selection
/// commandable.
#[test]
fn awaiting_a_target_holds_while_the_active_group_offers_its_command() {
    let selection = Selection::new([(id(1), PK), (id(2), GUARD)]);
    let attack = InterfaceState::awaiting(PanelCommand::Attack);
    assert!(still_valid(attack, &selection, true, &Conditions::default()));
    assert!(
        !still_valid(attack, &selection, false, &Conditions::default()),
        "no longer the player's"
    );
    let reverse = InterfaceState::awaiting(PanelCommand::Reverse);
    assert!(
        !still_valid(reverse, &selection, true, &Conditions::default()),
        "infantry can't reverse"
    );
    let patch = Selection::new([(id(8), ObjectKind::CrystalPatch)]);
    assert!(!still_valid(attack, &patch, true, &Conditions::default()));
    assert!(still_valid(
        InterfaceState::DefaultState,
        &patch,
        false,
        &Conditions::default()
    ));
}

/// `factions/gdo/units.md > Peacekeeper > Riot Shield`: R raises the shields of the Peacekeepers it goes to, or
/// lowers them when they are all up or on their way up; the button says which. Guards have no R.
#[test]
fn the_riot_shield_toggles_in_slot_r() {
    let r = slot(0, 3);
    let raise = |raise| SlotPress::UseAbility(PanelCommand::RiotShield, Ability::RiotShield { raise });
    let pks = Selection::new([(id(1), PK), (id(2), PK)]);
    let modes =
        |shields: [RiotShield; 2]| Conditions::with_modes(Modes::from_iter([(id(1), shields[0]), (id(2), shields[1])]));
    let down = panel(&pks, InterfaceState::DefaultState, &modes([RiotShield::Lowered; 2]));
    let shown = down.get(r).unwrap();
    assert_eq!(
        (shown.look, shown.on, shown.label()),
        (ButtonLook::Common, false, "Raise Shield")
    );
    assert_eq!(press(&down, r), raise(true));
    for going_up in [
        [RiotShield::Raised, RiotShield::Raised],
        [RiotShield::Raising { left: 2 }, RiotShield::Raised],
    ] {
        let up = panel(&pks, InterfaceState::DefaultState, &modes(going_up));
        assert_eq!(
            (up.get(r).unwrap().on, up.get(r).unwrap().label()),
            (true, "Lower Shield")
        );
        assert_eq!(press(&up, r), raise(false));
    }
    for differing in [
        [RiotShield::Raised, RiotShield::Lowered],
        [RiotShield::Raised, RiotShield::Lowering { left: 1 }],
    ] {
        let some = panel(&pks, InterfaceState::DefaultState, &modes(differing));
        assert_eq!(press(&some, r), raise(true), "{differing:?}: all of them up");
    }
    let guards = Selection::new([(id(3), GUARD)]);
    assert_eq!(
        panel(&guards, InterfaceState::DefaultState, &Conditions::default()).get(r),
        None
    );
    assert!(!offers(GUARD, PanelCommand::RiotShield) && offers(PK, PanelCommand::RiotShield));
    let mixed = Selection::new([(id(1), PK), (id(2), PK), (id(3), GUARD)]);
    assert_eq!(recipients(&mixed, PanelCommand::RiotShield), [id(1), id(2)]);
}

/// `factions/gdo/units.md > Peacekeeper > Riot Shield`: "the player must lower the shield before ordering an
/// attack". Attack is disabled while none of its recipients can attack, and a pending Attack ends; a Guard in the
/// selection still can.
#[test]
fn attack_waits_for_a_recipient_that_can_attack() {
    let a = slot(1, 0);
    let attack = InterfaceState::awaiting(PanelCommand::Attack);
    let shielded = Conditions::with_modes(Modes::from_iter([
        (id(1), RiotShield::Raised),
        (id(2), RiotShield::Lowering { left: 1 }),
    ]));
    let pks = Selection::new([(id(1), PK), (id(2), PK)]);
    let panel_of = |selection: &Selection| panel(selection, InterfaceState::DefaultState, &shielded);
    assert_eq!(panel_of(&pks).get(a).map(|s| s.look), Some(ButtonLook::Disabled));
    assert_eq!(press(&panel_of(&pks), a), SlotPress::Nothing);
    assert!(!still_valid(attack, &pks, true, &shielded));
    let with_guard = Selection::new([(id(1), PK), (id(2), PK), (id(3), GUARD)]);
    assert_eq!(
        press(&panel_of(&with_guard), a),
        SlotPress::AwaitTarget(PanelCommand::Attack)
    );
    assert!(still_valid(attack, &with_guard, true, &shielded));
    let one_down = Modes::from_iter([(id(1), RiotShield::Raised), (id(2), RiotShield::Lowered)]);
    assert!(one_down.can_attack(id(2)) && !one_down.can_attack(id(1)) && one_down.can_attack(id(3)));
    assert_eq!(
        panel(&pks, InterfaceState::DefaultState, &Conditions::with_modes(one_down))
            .get(a)
            .map(|s| s.look),
        Some(ButtonLook::Common)
    );
}

#[test]
fn only_tail_and_doubly_disjointed_attacks_target_the_ground() {
    use sc_sim::defs::FullyConnectedSubtype::Ranged;
    assert!(!can_target_ground(AttackType::FullyConnected(Ranged)));
    assert!(!can_target_ground(AttackType::HeadDisjointed));
    assert!(can_target_ground(AttackType::TailDisjointed));
    assert!(can_target_ground(AttackType::DoublyDisjointed));
    assert!(!offers(PK, PanelCommand::AttackGround) && !offers(GUARD, PanelCommand::Reverse));
}

/// `factions/gdo/structures.md > Barracks > ObjectInterfaceState[Barracks]` with `> Standard Slot Assignments`:
/// Q trains a Peacekeeper and X cancels the last unit queued, both at once; C awaits the rally point's target,
/// with Z Back and the other commands waiting meanwhile. W, the Rocketman's, stays empty until it is in the game.
#[test]
fn a_barracks_trains_in_q_cancels_in_x_and_rallies_in_c() {
    use ButtonLook::{Back, Common, Disabled, Pending};
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    let (q, x, c) = (slot(0, 0), CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    let barracks = Selection::new([(id(4), BARRACKS)]);
    let default = panel(&barracks, InterfaceState::DefaultState, &Conditions::default());
    let command = PanelButton::Command;
    assert_eq!(
        shown(&default),
        [
            (q, command(Train(PK)), Common),
            (x, command(CancelProduction), Common),
            (c, command(SetRallyPoint), Common),
        ]
    );
    let labels = [q, x, c].map(|slot| default.get(slot).unwrap().label());
    assert_eq!(labels, ["Train PK", "Cancel", "Rally"]);
    assert_eq!(
        press(&default, q),
        SlotPress::Produce(Train(PK), ProductionCommand::Train(PK))
    );
    assert_eq!(
        press(&default, x),
        SlotPress::Produce(CancelProduction, ProductionCommand::CancelProduction)
    );
    assert_eq!(press(&default, c), SlotPress::AwaitTarget(SetRallyPoint));
    assert_eq!(press(&default, slot(0, 1)), SlotPress::Nothing, "no Rocketman yet");

    let rally = InterfaceState::awaiting(SetRallyPoint);
    let awaiting = panel(&barracks, rally, &Conditions::default());
    assert_eq!(
        shown(&awaiting),
        [
            (q, command(Train(PK)), Disabled),
            (CommandSlot::BACK, PanelButton::Back, Back),
            (x, command(CancelProduction), Disabled),
            (c, command(SetRallyPoint), Pending),
        ]
    );
    assert_eq!(press(&awaiting, q), SlotPress::Nothing, "waits");
    assert_eq!(press(&awaiting, CommandSlot::BACK), SlotPress::Back);
    assert!(still_valid(rally, &barracks, true, &Conditions::default()));
    let units = Selection::new([(id(1), PK)]);
    assert!(
        !still_valid(rally, &units, true, &Conditions::default()),
        "units have no rally point"
    );
    // A Train the gate refuses doesn't end the targeting of a rally point: it needs nothing of the queue.
    let full = Conditions::default().refusing(Train(PK), Reject::QueueFull);
    assert!(still_valid(rally, &barracks, true, &full));
}

/// Plan decision T23: every type that produces units has the Production template, its grid built from its
/// `ProductionDef`: its units in Q, W and on, X Cancel Production and C Set Rally Point. The Barracks' is the grid of
/// `> ObjectInterfaceState[Barracks]`; the Robo Portal trains the Drone in Q, the Bio Portal the Sabre Legionnaire,
/// and their units not in the game yet hold their slots (design change 0124: the Robo Portal's W Scout, the Bio
/// Portal's W Lancer and E psionic unit; `gathering` has their panel).
#[test]
fn every_producer_gets_the_production_panel_of_its_recipes() {
    use PanelCommand::{CancelProduction, Planned as P, SetRallyPoint, Train};
    for &kind in ObjectKind::ALL {
        let production = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
        assert_eq!(
            InterfaceTemplate::of(kind) == Some(InterfaceTemplate::Production),
            production.is_some(),
            "{kind:?}"
        );
    }
    let produces = |unit, planned: &[Planned]| {
        let mut grid = [None; 12];
        grid[0] = Some(Train(unit));
        for (slot, &planned) in grid[1..].iter_mut().zip(planned) {
            *slot = Some(P(planned));
        }
        grid[slot_index(CommandSlot::CANCEL_PRODUCTION)] = Some(CancelProduction);
        grid[slot_index(CommandSlot::RALLY_POINT)] = Some(SetRallyPoint);
        grid
    };
    assert_eq!(grid(BARRACKS), produces(PK, &[]));
    assert_eq!(
        grid(ObjectKind::RoboPortal),
        produces(ObjectKind::Drone, &[Planned::Scout])
    );
    assert_eq!(
        grid(ObjectKind::BioPortal),
        produces(ObjectKind::SabreLegionnaire, &[Planned::Lancer, Planned::PsionicUnit])
    );
    let labels = [PK, ObjectKind::Drone, ObjectKind::SabreLegionnaire].map(|unit| Train(unit).label());
    assert_eq!(labels, ["Train PK", "Train DR", "Train SL"]);
}

/// Design changes 0081 and 0143 (`> ObjectInterfaceState[Barracks]`: "Only available if BuildQueue has fewer than
/// 20 entries and player has sufficient Space Crystals (Unit Control is checked when production starts)"; "Only
/// available if BuildQueue is not empty"): a production command the command gate would refuse is disabled, and
/// pressing it does nothing; the others stay available.
#[test]
fn a_production_command_the_gate_would_refuse_is_disabled() {
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    let (q, x, c) = (slot(0, 0), CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    let barracks = Selection::new([(id(4), BARRACKS), (id(5), BARRACKS)]);
    let look = |panel: &Panel, slot| panel.get(slot).map(|s| s.look);
    for reject in [Reject::QueueFull, Reject::InsufficientCrystals] {
        let refused = Conditions::default().refusing(Train(PK), reject);
        assert_eq!(refused.refusal(Train(PK)), Some(reject));
        let shown = panel(&barracks, InterfaceState::DefaultState, &refused);
        assert_eq!(look(&shown, q), Some(ButtonLook::Disabled), "{reject:?}");
        assert_eq!(press(&shown, q), SlotPress::Nothing);
        assert_eq!(look(&shown, x), Some(ButtonLook::Common));
        assert_eq!(press(&shown, c), SlotPress::AwaitTarget(SetRallyPoint));
    }
    let empty = Conditions::default().refusing(CancelProduction, Reject::NothingQueued);
    let shown = panel(&barracks, InterfaceState::DefaultState, &empty);
    assert_eq!(look(&shown, x), Some(ButtonLook::Disabled));
    assert_eq!(press(&shown, x), SlotPress::Nothing);
    assert_eq!(look(&shown, q), Some(ButtonLook::Common));
}

/// No smart-casting (`> SelectionGroup > No smart-casting`, design changes 0071 and 0075): three Barracks are one
/// SelectionGroup (Groupable), and each production command goes to all of them, the sim ordering their payments
/// (0083). Beside Peacekeepers, the Barracks' commands are group commands: they go to the Barracks while those are
/// the ActiveGroup, and the panel shows the Peacekeepers' while those are.
#[test]
fn production_commands_go_to_every_selected_barracks() {
    use PanelCommand::{CancelProduction, Move, SetRallyPoint, Train};
    let three = Selection::new([(id(4), BARRACKS), (id(5), BARRACKS), (id(6), BARRACKS)]);
    for command in [Train(PK), CancelProduction, SetRallyPoint] {
        assert_eq!(recipients(&three, command), [id(4), id(5), id(6)], "{command:?}");
    }
    let mut mixed = Selection::new([(id(1), PK), (id(2), PK), (id(4), BARRACKS), (id(5), BARRACKS)]);
    let units = panel(&mixed, InterfaceState::DefaultState, &Conditions::default());
    assert_eq!(
        units.get(slot(0, 0)).map(|s| s.button),
        Some(PanelButton::Command(Move))
    );
    assert_eq!(units.get(slot(0, 0)).map(|s| s.look), Some(ButtonLook::Group));
    assert_eq!(recipients(&mixed, Move), [id(1), id(2)]);
    mixed.cycle(true);
    let producers = panel(&mixed, InterfaceState::DefaultState, &Conditions::default());
    assert_eq!(
        producers.get(slot(0, 0)).map(|s| (s.button, s.look)),
        Some((PanelButton::Command(Train(PK)), ButtonLook::Group))
    );
    assert_eq!(recipients(&mixed, Train(PK)), [id(4), id(5)]);
    assert_eq!(recipients(&mixed, SetRallyPoint), [id(4), id(5)]);
}
