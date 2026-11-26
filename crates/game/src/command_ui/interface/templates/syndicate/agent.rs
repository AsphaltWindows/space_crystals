//! The Agent's panel (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`, slice M4-J1; design change
//! 0359). Its panel letters: "**Q** Move, **A** Build Tunnel (user, 2026-11-26: Agents no longer build Laser Turrets,
//! so the Construct menu of 0326 is reverted), **S** Drop Off Resources, **D** Repair (2026-11-26, matching the
//! Recruit's panel), **X** Stop. No Attack, Hold Position, Patrol or AttackMove buttons; the Agent attacks by
//! right-click". A enters the Tunnel's placement (`PanelCommand::Build`) and S awaits its drop-off
//! (`PanelCommand::DropOff`). D, "Left-click a damaged own Tunnel or Laser Turret to repair it" (`> DefaultState
//! commands`), comes with slice N8, as the Recruit's does: until then it holds its slot greyed (`Planned::Repair`).
//! Slice M4-S1's D, the Agent's Build Laser Turret, is gone with design change 0359: a turret is ordered from a Tunnel's
//! ExpandMenu alone (`super::tunnel`). Its right-click (`> Right-Click Resolution`: Gather, Drop Off, Enter, Attack,
//! Move; Repair from slice N8) is the sim's one rule, as every unit's
//! (`crate::command_ui::resolution::unit_right_click`).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate, Planned};

/// The Agent's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Agent);

/// `> Panel letters`, slot by slot: Q Move and X Stop, "as in BasicCombatUnitInterfaceState" (their slots there), A
/// Build Tunnel, S Drop Off and D Repair, the Recruit's slots (`super::super::cults::recruit::GRID`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Build, DropOff, Move, Stop};
    use sc_sim::ObjectKind::Tunnel;
    const REPAIR: PanelCommand = PanelCommand::Planned(Planned::Repair);
    [
        Some(Move),          None,          None,         None,
        Some(Build(Tunnel)), Some(DropOff), Some(REPAIR), None,
        None,                Some(Stop),    None,         None,
    ]
};
