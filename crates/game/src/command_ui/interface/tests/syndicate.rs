//! The Syndicate's panels as plain data (slice M4-J2; design changes 0257 and 0260): the Tunnel's letters, its
//! EjectMenu with its counts and its greyed tiles, and the Hideout's letters, its Guard behind its prerequisite (slice
//! M4-K) and its Grenadier not in the game yet, each greyed with why (design change 0162). The ExpandMenu's are
//! `super::expansions`'.

use std::collections::BTreeSet;

use sc_sim::TunnelCommand;
use sc_sim::defs::EJECT_MENU_ORDER;

use super::*;
use crate::command_ui::feedback::unavailable_text;

const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const AGENT: ObjectKind = ObjectKind::Agent;

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
/// 0257: Agent Q, Guard W), each with its count, greyed while the gate refuses its Eject, with the gate's reason; a
/// tile ejects at once and leaves the menu open; Z goes back to DefaultState.
#[test]
fn the_eject_menu_shows_the_network_in_grid_order_with_counts() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::Eject;
    let tunnel = Selection::new([(id(1), TUNNEL)]);
    let conditions = Conditions::default()
        .with_network([(AGENT, 3)])
        .refusing(Eject(GUARD), Reject::NotInNetwork(GUARD));
    let menu = panel(&tunnel, InterfaceState::EjectMenu, &conditions);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&menu),
        [
            (slot(0, 0), c(Eject(AGENT)), Common),
            (slot(0, 1), c(Eject(GUARD)), Disabled),
            (CommandSlot::BACK, PanelButton::Back, Back),
        ]
    );
    let counts: Vec<Option<u32>> = menu.slots().filter_map(|(_, s)| s.map(|s| s.count)).collect();
    assert_eq!(counts, [Some(3), Some(0), None]);
    assert_eq!(
        (
            menu.get(slot(0, 0)).unwrap().label(),
            menu.get(slot(0, 1)).unwrap().label()
        ),
        ("AG", "GD")
    );
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
    let first_two: Vec<Option<PanelCommand>> = grid()[..2].to_vec();
    assert_eq!(
        first_two,
        [Some(PanelCommand::Eject(AGENT)), Some(PanelCommand::Eject(GUARD))]
    );
}

/// `factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`: Q Build Agent, W Build Guard, E Build
/// Grenadier (design change 0260), X Cancel Production, C Set Rally Point. W is the Guard's recipe (slice M4-K), greyed
/// while the gate refuses it for want of an Optics Lab ("W: Build Guard [...] Only available if the player owns at
/// least one Optics Lab": `Reject::Requires`), and trains Guards once the gate takes it. The Grenadier isn't the
/// Hideout's yet: its slot is greyed with its prerequisite while the player owns no Chemical Plant, and as not in the
/// game yet once they own one.
#[test]
fn the_hideout_has_q_w_e_x_and_c() {
    use ButtonLook::{Common, Disabled, Unavailable};
    use PanelCommand::{CancelProduction, Planned as P, SetRallyPoint, Train};
    let hideout = Selection::new([(id(1), HIDEOUT)]);
    let no_lab = Conditions::default().refusing(Train(GUARD), Reject::Requires(ObjectKind::OpticsLab));
    let shown_panel = panel(&hideout, InterfaceState::DefaultState, &no_lab);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&shown_panel),
        [
            (slot(0, 0), c(Train(AGENT)), Common),
            (slot(0, 1), c(Train(GUARD)), Disabled),
            (slot(0, 2), c(P(Planned::Grenadier)), Unavailable),
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
    assert_eq!(
        press(&shown_panel, slot(0, 0)),
        SlotPress::Produce(Train(AGENT), ProductionCommand::Train(AGENT))
    );
    assert_eq!(
        press(&shown_panel, CommandSlot::RALLY_POINT),
        SlotPress::AwaitTarget(SetRallyPoint)
    );
    // An Optics Lab and a Chemical Plant owned: W trains Guards, and E isn't in the game yet.
    let owned = Conditions::default().owning(ObjectKind::ChemicalPlant);
    let shown_panel = panel(&hideout, InterfaceState::DefaultState, &owned);
    assert_eq!(
        shown_panel.get(slot(0, 1)).map(|s| (s.look, s.why)),
        Some((Common, None))
    );
    assert_eq!(
        press(&shown_panel, slot(0, 1)),
        SlotPress::Produce(Train(GUARD), ProductionCommand::Train(GUARD))
    );
    assert_eq!(why_text(&shown_panel, slot(0, 2)), Some("Not available yet"));
}
