//! The Tunnel's ExpandMenu and an expansion's placement as plain data (slice M4-F; design changes 0257 and 0291): the
//! menu's letters, its entries greyed with the gate's reasons, the placement it enters and where Z goes back to, and
//! the ghost held within the Tunnel Area.

use sc_sim::defs::{EXPAND_MENU, ExpandEntry};
use sc_sim::nav::TileRect;
use sc_sim::{TilePos, TunnelCommand};

use super::*;
use crate::command_ui::feedback::unavailable_text;

const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;

/// What the panel's `slot` says when pressed greyed out, if it is.
fn why_text(panel: &Panel, slot: CommandSlot) -> Option<&'static str> {
    panel.get(slot)?.why.map(unavailable_text)
}

/// Design change 0291's verification, "The ExpandMenu shows nine expansion slots and then the turret": the Tech Tree's
/// expansions in its order take Q, W, E, R, A, S, D, F and X (design change 0257: "entries take the grid slots in order
/// (Q, W, E, R, A, S, …)", Z being Back), the Hideout in Q, the eight not in the game yet greyed, and the Laser Turret
/// the tenth, C, greyed with its prerequisite. Each says what it builds and, greyed, why; Q enters the Hideout's
/// placement, and Z goes back to DefaultState.
#[test]
fn the_expand_menu_shows_nine_expansion_slots_and_then_the_turret() {
    use ButtonLook::{Back, Common, Unavailable};
    use PanelCommand::{Expand, Planned as P};
    use Planned::*;
    let tunnel = Selection::new([(id(1), TUNNEL)]);
    let menu = panel(&tunnel, InterfaceState::ExpandMenu, &Conditions::default());
    let c = PanelButton::Command;
    assert_eq!(
        shown(&menu),
        [
            (slot(0, 0), c(Expand(HIDEOUT)), Common),
            (slot(0, 1), c(P(OpticsLab)), Unavailable),
            (slot(0, 2), c(P(ChemicalPlant)), Unavailable),
            (slot(0, 3), c(P(Vault)), Unavailable),
            (slot(1, 0), c(P(VehicleAssembly)), Unavailable),
            (slot(1, 1), c(P(Forge)), Unavailable),
            (slot(1, 2), c(P(NullChamber)), Unavailable),
            (slot(1, 3), c(P(AerospaceAssembly)), Unavailable),
            (CommandSlot::BACK, PanelButton::Back, Back),
            (slot(2, 1), c(P(OrbitalCommand)), Unavailable),
            (slot(2, 2), c(P(LaserTurret)), Unavailable),
        ]
    );
    let labels: Vec<&str> = menu.slots().filter_map(|(_, s)| s.map(|s| s.label())).collect();
    assert_eq!(
        labels,
        ["HI", "OL", "CP", "VT", "VA", "FE", "NC", "AA", "Back", "OC", "LT"]
    );
    let names: Vec<String> = menu.slots().filter_map(|(slot, _)| menu.name(slot)).collect();
    let expected = [
        "Build Hideout",
        "Build Optics Lab",
        "Build Chemical Plant",
        "Build Vault",
        "Build Vehicle Assembly",
        "Build Forge",
        "Build Null Chamber",
        "Build Aerospace Assembly",
        "Back",
        "Build Orbital Command",
        "Build Laser Turret",
    ];
    assert_eq!(names, expected);
    assert_eq!(why_text(&menu, slot(0, 0)), None);
    assert_eq!(why_text(&menu, slot(0, 1)), Some("Not available yet"));
    assert_eq!(why_text(&menu, slot(2, 1)), Some("Not available yet"));
    assert_eq!(why_text(&menu, slot(2, 2)), Some("Requires an Optics Lab"));
    assert_eq!(press(&menu, slot(0, 0)), SlotPress::AwaitExpansion(HIDEOUT));
    assert_eq!(press(&menu, slot(0, 1)), SlotPress::Nothing);
    assert_eq!(press(&menu, slot(2, 2)), SlotPress::Nothing);
    assert_eq!(press(&menu, CommandSlot::BACK), SlotPress::Back);
    assert_eq!(InterfaceState::ExpandMenu.back(&tunnel), InterfaceState::DefaultState);
    assert_eq!(InterfaceState::ExpandMenu.pending(), None);
    assert!(!InterfaceState::ExpandMenu.takes_map_clicks());
}

/// The menu is the sim's list, entry by entry (`sc_sim::defs::EXPAND_MENU`): every entry has its command in its slot,
/// a type in the game its Expand and one not in it yet its placeholder, so a slice that brings an expansion moves no
/// letter.
#[test]
fn the_expand_menu_is_the_sims_list() {
    use templates::syndicate::expand_menu::{command, grid};
    let grid = grid();
    let slots: Vec<CommandSlot> = CommandSlot::all().filter(|&s| s != CommandSlot::BACK).collect();
    assert_eq!(EXPAND_MENU.len(), 10);
    for (entry, slot) in EXPAND_MENU.iter().zip(slots) {
        let shown = grid[slot_index(slot)];
        assert_eq!(shown, command(*entry), "{entry:?}");
        match entry {
            ExpandEntry::Kind(kind) => assert_eq!(shown, Some(PanelCommand::Expand(*kind))),
            ExpandEntry::Planned(name) => {
                let Some(PanelCommand::Planned(planned)) = shown else {
                    panic!("{name}: no placeholder");
                };
                assert_eq!(planned.name(), *name);
            }
        }
    }
    assert_eq!(grid[slot_index(CommandSlot::BACK)], None);
    assert_eq!(grid[slot_index(slot(2, 3))], None, "V is free");
}

/// `factions/syndicate/tunnels.md > ExpandMenu`: an expansion is greyed while this Tunnel can't build it, with the
/// gate's reason (`Sim::can_expand`; design change 0162): an operation under way ("begins construction if the Tunnel
/// is not already performing an operation"), a Tier Requirement above this Tunnel's tier ("Expansion types whose Tier
/// Requirement exceeds this Tunnel's tier are visible but greyed out"), or a bank that can't pay. A greyed entry does
/// nothing when pressed.
#[test]
fn an_expansion_is_greyed_with_the_gates_reason() {
    let tunnel = Selection::new([(id(1), TUNNEL)]);
    let expand = PanelCommand::Expand(HIDEOUT);
    for (reject, says) in [
        (Reject::OperationUnderWay(id(1)), "This Tunnel is busy"),
        (
            Reject::TierRequirement { kind: HIDEOUT, tier: 2 },
            "This Tunnel's tier is too low",
        ),
        (Reject::InsufficientCrystals, "Not enough Space Crystals"),
    ] {
        let conditions = Conditions::default().refusing(expand, reject);
        let menu = panel(&tunnel, InterfaceState::ExpandMenu, &conditions);
        assert_eq!(menu.get(slot(0, 0)).map(|s| s.look), Some(ButtonLook::Disabled));
        assert_eq!(why_text(&menu, slot(0, 0)), Some(says), "{reject:?}");
        assert_eq!(press(&menu, slot(0, 0)), SlotPress::Nothing);
    }
}

/// The ExpandMenu and its entries are the Tunnel's own ("**this** Tunnel's tier"): with two Tunnels selected
/// (Ungroupable), they go to the one Tabbed to; the menu holds while a Tunnel is the ActiveGroup, and only then.
/// Only a Tunnel offers an expansion, and only one in the game.
#[test]
fn the_expand_menu_belongs_to_the_active_tunnel() {
    let conditions = Conditions::default();
    let tunnels = Selection::new([(id(1), TUNNEL), (id(2), TUNNEL)]);
    assert!(still_valid(InterfaceState::ExpandMenu, &tunnels, true, &conditions));
    assert!(!still_valid(InterfaceState::ExpandMenu, &tunnels, false, &conditions));
    let hideout = Selection::new([(id(3), HIDEOUT)]);
    assert!(!still_valid(InterfaceState::ExpandMenu, &hideout, true, &conditions));
    let active = tunnels.active_group().unwrap().instances;
    assert_eq!(active.len(), 1, "a Tunnel is Ungroupable");
    for command in [
        PanelCommand::ExpandMenu,
        PanelCommand::Expand(HIDEOUT),
        PanelCommand::CancelOperation,
    ] {
        assert_eq!(recipients(&tunnels, command), active, "{command:?}");
    }
    assert!(commands(TUNNEL).contains(&PanelCommand::Expand(HIDEOUT)));
    assert!(offers(TUNNEL, PanelCommand::Expand(HIDEOUT)));
    assert!(!offers(HIDEOUT, PanelCommand::Expand(HIDEOUT)));
    assert!(!offers(TUNNEL, PanelCommand::Expand(ObjectKind::Barracks)));
    assert_eq!(PanelCommand::CancelOperation.tunnel(), Some(TunnelCommand::Cancel));
    assert_eq!(PanelCommand::Expand(HIDEOUT).tunnel(), None, "it waits for its click");
}

/// `factions/syndicate/tunnels.md > ExpandMenu`: "Click an expansion type: [...] enters AwaitingPlacement for that
/// expansion"; `factions/syndicate/tunnels.md > Expansions under construction`: "**Z**: returns to ExpandMenu". While
/// it is placed, the menu stays on the panel with the expansion marked and the others waiting, and the placement holds
/// while its Tunnel is the ActiveGroup.
#[test]
fn an_expansion_is_placed_from_the_menu_and_z_goes_back_to_it() {
    let tunnel = Selection::new([(id(1), TUNNEL)]);
    let placing = Placing::expansion(id(1), HIDEOUT);
    assert_eq!(placing.builder, Builder::Tunnel(id(1)));
    assert_eq!(placing.command(), PanelCommand::Expand(HIDEOUT));
    assert!(placing.follows_cursor());
    let placing_state = InterfaceState::AwaitingPlacement(placing);
    assert_eq!(placing_state.back(&tunnel), InterfaceState::ExpandMenu);
    let conditions = Conditions::default();
    assert!(still_valid(placing_state, &tunnel, true, &conditions));
    let other = Selection::new([(id(2), TUNNEL)]);
    assert!(
        !still_valid(placing_state, &other, true, &conditions),
        "another Tunnel's"
    );
    let shown_panel = panel(&tunnel, placing_state, &conditions);
    let look = |slot| shown_panel.get(slot).map(|s| s.look);
    assert_eq!(look(slot(0, 0)), Some(ButtonLook::Pending));
    assert_eq!(look(slot(0, 1)), Some(ButtonLook::Unavailable));
    assert_eq!(look(CommandSlot::BACK), Some(ButtonLook::Back));
}

/// `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`: "A ghost preview of the expansion follows the
/// cursor within the Tunnel Area": the footprint nearest the cursor's that lies wholly within the area, the cursor's
/// own while it does.
#[test]
fn the_ghost_stays_within_the_tunnel_area() {
    let area = TileRect::new(TilePos::new(14, 13), 8, 8);
    let within = |x, y| super::super::placing::within(TilePos::new(x, y), (2, 2), area);
    assert_eq!(within(16, 15), TilePos::new(16, 15), "inside");
    assert_eq!(within(20, 19), TilePos::new(20, 19), "against the south-east corner");
    assert_eq!(within(30, 15), TilePos::new(20, 15), "east of it");
    assert_eq!(within(16, -4), TilePos::new(16, 13), "north of it");
    assert_eq!(within(0, 40), TilePos::new(14, 19), "south-west of it");
    let turned = super::super::placing::within(TilePos::new(40, 40), (3, 1), area);
    assert_eq!(turned, TilePos::new(19, 20), "a footprint as turned");
}
