//! The Syndicate's panels as plain data (slice M4-J2; design changes 0257 and 0260): the Tunnel's letters, its
//! EjectMenu with its counts and its greyed tiles, the Hideout's letters, its Guard and its Grenadier behind their
//! prerequisites (slices M4-K and M4-U), each greyed with why, and the Grenadier's combat grid with its Attack Ground
//! (slice M4-U). The ExpandMenu's are `super::expansions`'.

use std::collections::BTreeSet;

use sc_sim::TunnelCommand;
use sc_sim::defs::EJECT_MENU_ORDER;

use super::*;
use crate::command_ui::feedback::unavailable_text;

const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const AGENT: ObjectKind = ObjectKind::Agent;
const GRENADIER: ObjectKind = ObjectKind::Grenadier;

/// What the panel's `slot` says when pressed greyed out, if it is.
fn why_text(panel: &Panel, slot: CommandSlot) -> Option<&'static str> {
    panel.get(slot)?.why.map(unavailable_text)
}

/// `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState` (design change 0257): Q Upgrade Tunnel, W Expand
/// Tunnel, E Eject, X Cancel. Q sends the Upgrade (slice M4-G) and names its price; W opens the ExpandMenu (slice
/// M4-F) and E the EjectMenu; X cancels the Tunnel's operation, greyed while it performs none (`Reject::NoOperation`),
/// and sends the Cancel once it does (`factions/syndicate/tunnels.md > Expansions under construction`: "The Tunnel's X
/// cancels whatever operation it is running"), an upgrade's too. A greyed-out button does nothing and says why.
#[test]
fn the_tunnel_has_q_upgrade_w_expand_e_eject_and_x_cancel() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelOperation, EjectMenu, ExpandMenu, UpgradeTunnel};
    let tunnel = Selection::new([(id(1), TUNNEL)]);
    let conditions =
        (Conditions::default().refusing(CancelOperation, Reject::NoOperation(id(1)))).pricing(UpgradeTunnel, 2);
    let shown_panel = panel(&tunnel, InterfaceState::DefaultState, &conditions);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&shown_panel),
        [
            (slot(0, 0), c(UpgradeTunnel), Common),
            (slot(0, 1), c(ExpandMenu), Common),
            (slot(0, 2), c(EjectMenu), Common),
            (CommandSlot::CANCEL_PRODUCTION, c(CancelOperation), Disabled),
        ]
    );
    let labels: Vec<&str> = shown_panel.slots().filter_map(|(_, s)| s.map(|s| s.label())).collect();
    assert_eq!(labels, ["Upgr.", "Expand", "Eject", "Cancel"]);
    let names: Vec<String> = shown_panel
        .slots()
        .filter_map(|(slot, _)| shown_panel.name(slot))
        .collect();
    assert_eq!(names, ["Upgrade Tunnel", "Expand Tunnel", "Eject", "Cancel Operation"]);
    assert_eq!(shown_panel.get(slot(0, 0)).and_then(|s| s.price), Some(2));
    assert_eq!(why_text(&shown_panel, slot(0, 0)), None);
    assert_eq!(why_text(&shown_panel, slot(0, 1)), None);
    assert_eq!(why_text(&shown_panel, slot(0, 2)), None);
    assert_eq!(
        why_text(&shown_panel, CommandSlot::CANCEL_PRODUCTION),
        Some("Nothing to cancel")
    );
    assert_eq!(press(&shown_panel, CommandSlot::CANCEL_PRODUCTION), SlotPress::Nothing);
    assert_eq!(
        press(&shown_panel, slot(0, 0)),
        SlotPress::Tunnel(UpgradeTunnel, TunnelCommand::Upgrade)
    );
    assert_eq!(press(&shown_panel, slot(0, 1)), SlotPress::OpenExpandMenu);
    assert_eq!(press(&shown_panel, slot(0, 2)), SlotPress::OpenEjectMenu);
    assert_eq!(InterfaceTemplate::of(TUNNEL), Some(InterfaceTemplate::Tunnel));

    let building = panel(&tunnel, InterfaceState::DefaultState, &Conditions::default());
    assert_eq!(
        press(&building, CommandSlot::CANCEL_PRODUCTION),
        SlotPress::Tunnel(CancelOperation, TunnelCommand::Cancel)
    );
}

/// Slice M4-G: the Tunnel's Q is greyed while the gate refuses its Upgrade (`Sim::can_upgrade`), with the gate's
/// reason: already at its top tier (`factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`: "Unavailable if
/// already Tier 3"), an operation under way, too few Supplies, or something that blocks the larger area, never what
/// (`factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`: "learns that something blocks it"). Pressed, it
/// does nothing and says why; its price still shows.
#[test]
fn the_tunnels_upgrade_is_greyed_with_the_gates_reason() {
    use PanelCommand::UpgradeTunnel;
    let tunnel = Selection::new([(id(1), TUNNEL)]);
    let cases = [
        (Reject::TopTier(id(1)), "Already at the top tier"),
        (Reject::OperationUnderWay(id(1)), "This Tunnel is busy"),
        (Reject::InsufficientSupplies, "Not enough Supplies"),
        (Reject::UpgradeBlocked(id(1)), "Something blocks the upgrade"),
    ];
    for (reject, text) in cases {
        let conditions = Conditions::default()
            .refusing(UpgradeTunnel, reject)
            .pricing(UpgradeTunnel, 4);
        let shown_panel = panel(&tunnel, InterfaceState::DefaultState, &conditions);
        let q = shown_panel.get(slot(0, 0)).expect("Q");
        assert_eq!(
            (q.button, q.look),
            (PanelButton::Command(UpgradeTunnel), ButtonLook::Disabled)
        );
        assert_eq!(q.price, Some(4));
        assert_eq!(why_text(&shown_panel, slot(0, 0)), Some(text), "{reject:?}");
        assert_eq!(press(&shown_panel, slot(0, 0)), SlotPress::Nothing);
    }
}

/// `factions/syndicate/tunnels.md > EjectMenu`: a tile per unit type of the sim's list, in grid order (design change
/// 0257: Agent Q, Guard W, Grenadier E), each with its count, greyed while the gate refuses its Eject, with the gate's
/// reason; a tile ejects at once and leaves the menu open; Z goes back to DefaultState.
#[test]
fn the_eject_menu_shows_the_network_in_grid_order_with_counts() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::Eject;
    let tunnel = Selection::new([(id(1), TUNNEL)]);
    let conditions = Conditions::default()
        .with_network([(AGENT, 3)])
        .refusing(Eject(GUARD), Reject::NotInNetwork(GUARD))
        .refusing(Eject(GRENADIER), Reject::NotInNetwork(GRENADIER));
    let menu = panel(&tunnel, InterfaceState::EjectMenu, &conditions);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&menu),
        [
            (slot(0, 0), c(Eject(AGENT)), Common),
            (slot(0, 1), c(Eject(GUARD)), Disabled),
            (slot(0, 2), c(Eject(GRENADIER)), Disabled),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let counts: Vec<Option<u32>> = menu.slots().filter_map(|(_, s)| s.map(|s| s.count)).collect();
    assert_eq!(counts, [Some(3), Some(0), Some(0), None]);
    let labels: Vec<&str> = menu.slots().filter_map(|(_, s)| s.map(|s| s.label())).collect();
    assert_eq!(labels[..3], ["AG", "GD", "GR"]);
    assert_eq!(menu.name(slot(0, 2)).as_deref(), Some("Eject Grenadier"));
    assert_eq!(menu.name(slot(0, 0)).as_deref(), Some("Eject Agent"));
    assert_eq!(why_text(&menu, slot(0, 1)), Some("None in the Tunnel Network"));
    let eject = TunnelCommand::Eject { kind: AGENT };
    assert_eq!(press(&menu, slot(0, 0)), SlotPress::Tunnel(Eject(AGENT), eject));
    assert_eq!(press(&menu, slot(0, 1)), SlotPress::Nothing);
    assert_eq!(press(&menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(InterfaceState::EjectMenu.back(&tunnel), InterfaceState::DefaultState);
    assert_eq!(InterfaceState::EjectMenu.pending(), None);
    assert!(!InterfaceState::EjectMenu.takes_map_clicks());

    // Design change 0288 point 1: while every Agent only just walked in, its tile is greyed, and says so; a Tunnel
    // whose tier doesn't let a type out greys it too ("visible but greyed out (disabled)").
    let in_transit = Conditions::default()
        .with_network([(AGENT, 1), (GUARD, 1)])
        .refusing(Eject(AGENT), Reject::InTransit(AGENT))
        .refusing(
            Eject(GUARD),
            Reject::NoTransit {
                tunnel: id(1),
                kind: Some(GUARD),
            },
        );
    let menu = panel(&tunnel, InterfaceState::EjectMenu, &in_transit);
    assert_eq!(why_text(&menu, slot(0, 0)), Some("Still in transit"));
    assert_eq!(why_text(&menu, slot(0, 1)), Some("This Tunnel's tier is too low"));
    assert_eq!(
        menu.get(slot(0, 0)).map(|s| (s.look, s.count)),
        Some((Disabled, Some(1)))
    );
}

/// The menu holds while a Tunnel is the ActiveGroup, and only then; a Tunnel's EjectMenu and tiles are its own
/// ("from this Tunnel's Side A"): with two Tunnels selected (Ungroupable), they go to the one Tabbed to.
#[test]
fn the_eject_menu_belongs_to_the_active_tunnel() {
    let conditions = Conditions::default();
    let tunnels = Selection::new([(id(1), TUNNEL), (id(2), TUNNEL)]);
    assert!(still_valid(InterfaceState::EjectMenu, &tunnels, true, &conditions));
    assert!(!still_valid(InterfaceState::EjectMenu, &tunnels, false, &conditions));
    let hideout = Selection::new([(id(3), HIDEOUT)]);
    assert!(!still_valid(InterfaceState::EjectMenu, &hideout, true, &conditions));
    let active = tunnels.active_group().unwrap().instances;
    assert_eq!(active.len(), 1, "a Tunnel is Ungroupable");
    for command in [PanelCommand::EjectMenu, PanelCommand::Eject(AGENT)] {
        assert_eq!(recipients(&tunnels, command), active);
    }
    // Another type's panel shows its own grid in the EjectMenu, until the state is validated.
    let shown_panel = panel(&hideout, InterfaceState::EjectMenu, &conditions);
    assert_eq!(
        shown_panel.get(slot(0, 0)).map(|s| s.button),
        Some(PanelButton::Command(PanelCommand::Train(AGENT)))
    );
    // The tiles are the Tunnel's commands, and no one else's.
    let offered: BTreeSet<PanelCommand> = commands(TUNNEL)
        .into_iter()
        .filter(|c| matches!(c, PanelCommand::Eject(_)))
        .collect();
    let listed: BTreeSet<PanelCommand> = EJECT_MENU_ORDER.iter().map(|&k| PanelCommand::Eject(k)).collect();
    assert_eq!(offered, listed);
    assert!(offers(TUNNEL, PanelCommand::Eject(GUARD)) && !offers(HIDEOUT, PanelCommand::Eject(GUARD)));
    assert!(!offers(TUNNEL, PanelCommand::Eject(ObjectKind::Peacekeeper)));
}

/// Design change 0257: "entries take the grid slots in order (Q, W, E, R, A, S, …)", every slot but Z's, Back: an
/// eleventh type would take V.
#[test]
fn the_eject_menu_takes_the_slots_in_order_but_z() {
    use templates::syndicate::eject_menu::{grid, grid_of};
    let kinds = [AGENT; 11];
    let filled: Vec<CommandSlot> = CommandSlot::all()
        .zip(grid_of(&kinds))
        .filter_map(|(slot, entry)| entry.map(|_| slot))
        .collect();
    let all_but_z: Vec<CommandSlot> = CommandSlot::all().filter(|&s| s != CommandSlot::BACK).collect();
    assert_eq!(filled, all_but_z);
    let first_three: Vec<Option<PanelCommand>> = grid()[..3].to_vec();
    let ejects = [AGENT, GUARD, GRENADIER].map(|kind| Some(PanelCommand::Eject(kind)));
    assert_eq!(first_three, ejects, "Agent Q, Guard W, Grenadier E");
}

/// `factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`: Q Build Agent, W Build Guard, E Build
/// Grenadier (design change 0260), X Cancel Production, C Set Rally Point. W and E are the Guard's recipe (slice M4-K)
/// and the Grenadier's (slice M4-U), each greyed while the gate refuses it for want of its prerequisite ("W: Build
/// Guard [...] Only available if the player owns at least one Optics Lab"; "E: Build Grenadier [...] Only available if
/// the player owns at least one Chemical Plant": `Reject::Requires`), and each trains its unit once the gate takes it.
#[test]
fn the_hideout_has_q_w_e_x_and_c() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{CancelProduction, SetRallyPoint, Train};
    let hideout = Selection::new([(id(1), HIDEOUT)]);
    let no_lab = Conditions::default()
        .refusing(Train(GUARD), Reject::Requires(ObjectKind::OpticsLab))
        .refusing(Train(GRENADIER), Reject::Requires(ObjectKind::ChemicalPlant));
    let shown_panel = panel(&hideout, InterfaceState::DefaultState, &no_lab);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&shown_panel),
        [
            (slot(0, 0), c(Train(AGENT)), Common),
            (slot(0, 1), c(Train(GUARD)), Disabled),
            (slot(0, 2), c(Train(GRENADIER)), Disabled),
            (CommandSlot::CANCEL_PRODUCTION, c(CancelProduction), Common),
            (CommandSlot::RALLY_POINT, c(SetRallyPoint), Common),
        ]
    );
    let labels: Vec<&str> = shown_panel.slots().filter_map(|(_, s)| s.map(|s| s.label())).collect();
    assert_eq!(labels, ["AG", "GD", "GR", "Cancel", "Rally"]);
    assert_eq!(why_text(&shown_panel, slot(0, 1)), Some("Requires an Optics Lab"));
    assert_eq!(why_text(&shown_panel, slot(0, 2)), Some("Requires a Chemical Plant"));
    assert_eq!(shown_panel.name(slot(0, 1)).as_deref(), Some("Train Guard"));
    assert_eq!(shown_panel.name(slot(0, 2)).as_deref(), Some("Train Grenadier"));
    assert_eq!(press(&shown_panel, slot(0, 1)), SlotPress::Nothing);
    assert_eq!(press(&shown_panel, slot(0, 2)), SlotPress::Nothing);
    assert_eq!(
        press(&shown_panel, slot(0, 0)),
        SlotPress::Produce(Train(AGENT), ProductionCommand::Train(AGENT))
    );
    assert_eq!(
        press(&shown_panel, CommandSlot::RALLY_POINT),
        SlotPress::AwaitTarget(SetRallyPoint)
    );
    // An Optics Lab and a Chemical Plant owned: W trains Guards, and E Grenadiers.
    let owned = Conditions::default();
    let shown_panel = panel(&hideout, InterfaceState::DefaultState, &owned);
    for (slot, kind) in [(slot(0, 1), GUARD), (slot(0, 2), GRENADIER)] {
        assert_eq!(shown_panel.get(slot).map(|s| (s.look, s.why)), Some((Common, None)));
        assert_eq!(
            press(&shown_panel, slot),
            SlotPress::Produce(Train(kind), ProductionCommand::Train(kind))
        );
    }
}

/// `factions/syndicate/units/grenadier.md > ObjectInterfaceState: BasicCombatUnitInterfaceState` (slice M4-U): the
/// Grenadier's panel is the generic combat grid, whose D, Attack Ground, it can take, as every attack that can miss
/// (`systems/combat.md > AttackType > CanTargetGround`: true for a DoublyDisjointed one; slice M4-O), where the
/// Guard's laser can't. Common to Grenadiers, D awaits its target and goes to them all; beside a Guard, it is their
/// group's command, which goes to the Grenadiers alone while they are the ActiveGroup. Like every infantry, it can't
/// reverse.
#[test]
fn the_grenadier_gets_the_combat_grid_and_attack_ground() {
    use ButtonLook::{Common, Unavailable};
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    let grenadiers = Selection::new([(id(1), GRENADIER), (id(2), GRENADIER)]);
    let shown_panel = panel(&grenadiers, InterfaceState::DefaultState, &Conditions::default());
    let c = PanelButton::Command;
    assert_eq!(
        shown(&shown_panel),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(Reverse), Unavailable),
            (slot(0, 2), c(HoldPosition), Common),
            (slot(1, 0), c(Attack), Common),
            (slot(1, 1), c(Patrol), Common),
            (slot(1, 2), c(AttackGround), Common),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(press(&shown_panel, slot(1, 2)), SlotPress::AwaitTarget(AttackGround));
    assert_eq!(recipients(&grenadiers, AttackGround), [id(1), id(2)]);
    assert_eq!(
        InterfaceTemplate::of(GRENADIER),
        Some(InterfaceTemplate::BasicCombatUnit)
    );
    assert!(offers(GRENADIER, AttackGround) && !offers(GUARD, AttackGround));

    let mut with_guard = Selection::new([(id(1), GRENADIER), (id(3), GUARD)]);
    assert!(!is_common(&with_guard, AttackGround));
    while !with_guard.in_active_group(id(1)) {
        assert!(with_guard.cycle(true), "Tab");
    }
    assert_eq!(recipients(&with_guard, AttackGround), [id(1)]);
}
