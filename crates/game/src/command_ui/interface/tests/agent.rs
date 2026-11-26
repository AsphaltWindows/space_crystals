//! The Agent's panel and a site's as plain data (slice M4-J1; design changes 0233, 0237, 0257 and 0359): the Agent's
//! letters, its Build Tunnel greyed while the bank can't pay with its price, its Drop Off greyed while it carries
//! nothing, its D Repair awaiting its target (slice M5-N8) and no Build Laser Turret (slice M5-Y), and a site's X
//! Cancel Construction alone, a Laser Turret's too.

use sc_sim::ConstructionCommand;
use sc_sim::construction::site::BuildPrice;

use super::*;
use crate::command_ui::feedback::unavailable_text;
use crate::selection::GroupType;

const AGENT: ObjectKind = ObjectKind::Agent;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const BUILD: PanelCommand = PanelCommand::Build(TUNNEL);
const TURRET: ObjectKind = ObjectKind::LaserTurret;
const REPAIR: PanelCommand = PanelCommand::Repair;

/// What the panel's `slot` says when pressed greyed out, if it is.
fn why_text(panel: &Panel, slot: CommandSlot) -> Option<&'static str> {
    panel.get(slot)?.why.map(unavailable_text)
}

/// `factions/syndicate/units/agent.md > Agent ObjectInterfaceState` (design changes 0257, 0359 and 0398): "**Q** Move,
/// **W** Build Tunnel (moved from A, 2026-11-27 [...]), **A** Attack (user, 2026-11-27: A is Attack on every unit
/// [...]) (an enemy, or Attack Move on ground), **S** Drop Off Resources, **D** Repair (2026-11-26, matching the
/// Recruit's panel), **X** Stop. No Hold Position or Patrol buttons." A awaits the target of an Attack. D, Repair
/// (slice M5-N8), awaits its target ("Left-click a damaged own Tunnel or Laser Turret to repair it"), with no price. W
/// enters the Tunnel's placement for the one Agent of the ActiveGroup, its price in Supplies shown, and no other button
/// has a price. Design change 0359's verification, "The Agent panel shows A: Build Tunnel with no turret option" (W
/// since 0398): the Agent offers no Build of a Laser Turret, in any state (the Tunnel's ExpandMenu does,
/// `super::expansions`).
#[test]
fn the_agent_has_q_move_w_build_tunnel_a_attack_s_drop_off_d_repair_and_x_stop() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{Attack, DropOff, Move, Stop};
    let agent = Selection::new([(id(1), AGENT)]);
    let conditions = Conditions::default().pricing(BUILD, BuildPrice::supplies(0));
    let shown_panel = panel(&agent, InterfaceState::DefaultState, &conditions);
    let c = PanelButton::Command;
    assert_eq!(
        shown(&shown_panel),
        [
            (slot(0, 0), c(Move), Common),
            (slot(0, 1), c(BUILD), Common),
            (slot(1, 0), c(Attack), Common),
            (slot(1, 1), c(DropOff), Disabled),
            (slot(1, 2), c(REPAIR), Common),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    let labels: Vec<&str> = shown_panel.slots().filter_map(|(_, s)| s.map(|s| s.label())).collect();
    assert_eq!(labels, ["Move", "T1", "Attack", "Drop", "Repair", "Stop"]);
    assert_eq!(shown_panel.name(slot(0, 1)).as_deref(), Some("Build Tunnel"));
    assert_eq!(press(&shown_panel, slot(1, 0)), SlotPress::AwaitTarget(Attack));
    assert_eq!(shown_panel.name(slot(1, 2)).as_deref(), Some("Repair"));
    assert_eq!(why_text(&shown_panel, slot(1, 2)), None);
    assert_eq!(press(&shown_panel, slot(1, 2)), SlotPress::AwaitTarget(REPAIR));
    assert_eq!(why_text(&shown_panel, slot(1, 1)), Some("Not carrying resources"));
    assert_eq!(press(&shown_panel, slot(1, 1)), SlotPress::Nothing);
    assert_eq!(press(&shown_panel, slot(0, 1)), SlotPress::AwaitBuild(TUNNEL));
    let price = |at| shown_panel.get(at).and_then(|s| s.price);
    assert_eq!(price(slot(0, 1)), Some(BuildPrice::supplies(0)));
    assert_eq!([slot(0, 0), slot(1, 0), slot(1, 2)].map(price), [None, None, None]);
    for command in [
        PanelCommand::HoldPosition,
        PanelCommand::Patrol,
        PanelCommand::Build(TURRET),
    ] {
        assert!(!offers(AGENT, command), "{command:?}");
    }
    let builds: Vec<PanelCommand> = (commands(AGENT).into_iter())
        .filter(|command| matches!(command, PanelCommand::Build(_)))
        .collect();
    assert_eq!(builds, [BUILD], "the Tunnel alone");
    assert!(offers(TUNNEL, PanelCommand::Expand(TURRET)));
    assert_eq!(recipients(&agent, BUILD), [id(1)]);
}

/// Design change 0233: "a Build order that can't be paid is refused": W is greyed while the gate would refuse the
/// Build for want of Supplies, and pressing it says so and starts no placement (design change 0162), its price still
/// shown. The site's check isn't the button's: the ghost judges each spot.
#[test]
fn build_tunnel_is_greyed_while_the_bank_cant_pay() {
    let agent = Selection::new([(id(1), AGENT)]);
    let conditions = Conditions::default()
        .pricing(BUILD, BuildPrice::supplies(2))
        .refusing(BUILD, Reject::InsufficientSupplies);
    let shown_panel = panel(&agent, InterfaceState::DefaultState, &conditions);
    let build = shown_panel.get(slot(0, 1)).unwrap();
    assert_eq!(build.look, ButtonLook::Disabled);
    assert_eq!(build.price, Some(BuildPrice::supplies(2)));
    assert_eq!(why_text(&shown_panel, slot(0, 1)), Some("Not enough Supplies"));
    assert_eq!(press(&shown_panel, slot(0, 1)), SlotPress::Nothing);
}

/// `factions/syndicate/units/agent.md > DefaultState commands`: Drop Off is "Always visible, greyed out when Agent is
/// not carrying resources": it awaits its Tunnel while one of its recipients carries a load (`Modes::carries`), and
/// goes to every selected Agent, the gate leaving out those that carry nothing; the Build goes to the Agent Tabbed to
/// alone (Agents are Ungroupable, "each Agent takes exactly one placement").
#[test]
fn drop_off_awaits_its_tunnel_while_an_agent_carries_a_load() {
    let agents = Selection::new([(id(1), AGENT), (id(2), AGENT)]);
    let loaded = Modes::default().with_loaded([id(2)]);
    let conditions = Conditions::with_modes(loaded);
    let shown_panel = panel(&agents, InterfaceState::DefaultState, &conditions);
    assert_eq!(shown_panel.get(slot(1, 1)).unwrap().look, ButtonLook::Common);
    assert_eq!(
        press(&shown_panel, slot(1, 1)),
        SlotPress::AwaitTarget(PanelCommand::DropOff)
    );
    assert_eq!(recipients(&agents, PanelCommand::DropOff), [id(1), id(2)]);
    assert_eq!(recipients(&agents, BUILD), [id(1)]);
}

/// Design change 0237 (`factions/syndicate/units/agent.md > While the Tunnel is under construction`): "It does
/// nothing but Cancel: [...] the only command is X Cancel Construction (as for a Colonists building being
/// transformed)". A site of any type has that panel, its Cancel going to the site as a construction command; it is
/// not a transformation's Cancel.
#[test]
fn a_site_shows_x_cancel_construction_alone() {
    let site = Selection::new([(id(1), GroupType::unfinished(TUNNEL))]);
    let conditions = Conditions::default().on_site();
    let shown_panel = panel(&site, InterfaceState::DefaultState, &conditions);
    let cancel = PanelCommand::CancelConstruction;
    assert_eq!(
        shown(&shown_panel),
        [(slot(2, 1), PanelButton::Command(cancel), ButtonLook::Common)]
    );
    assert_eq!(
        press(&shown_panel, slot(2, 1)),
        SlotPress::Construct(cancel, ConstructionCommand::CancelConstruction)
    );
    assert_eq!(shown_panel.name(slot(2, 1)).as_deref(), Some("Cancel Construction"));
    assert_eq!(recipients(&site, cancel), [id(1)]);
    for ty in [
        GroupType::unfinished(TUNNEL),
        GroupType::unfinished(ObjectKind::Hideout),
    ] {
        assert!(!offers(ty, PanelCommand::EjectMenu));
    }
    let turret = Selection::new([(id(1), GroupType::unfinished(TURRET))]);
    let shown_panel = panel(&turret, InterfaceState::DefaultState, &conditions);
    assert_eq!(
        shown(&shown_panel),
        [(slot(2, 1), PanelButton::Command(cancel), ButtonLook::Common)]
    );
}
