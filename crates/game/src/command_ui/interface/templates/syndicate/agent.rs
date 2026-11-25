//! The Agent's panel (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`, slice M4-J1). Its panel
//! letters (design change 0257, answering Q0068): "**Q** Move, **A** Build Tunnel, **S** Drop Off Resources, **D**
//! Build Laser Turret (greyed until available), **X** Stop. No Attack, Hold Position, Patrol or AttackMove buttons; the
//! Agent attacks by right-click". These replace the doc's older "B: Drop Off Resources" and "Build Laser Turret (hotkey
//! TBD)" (Q0085: the newer rule wins). A enters the Tunnel's placement (`PanelCommand::Build`), S awaits its drop-off
//! (`PanelCommand::DropOff`), and D holds its slot, greyed with its reason, until slice M4-S1 brings the Laser Turret
//! ([`Planned::LaserTurret`]). Its right-click (`> Right-Click Resolution`: Gather, Drop Off, Enter, Attack, Move) is
//! the sim's one rule, as every unit's (`crate::command_ui::resolution::unit_right_click`).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate, Planned};

/// The Agent's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Agent);

/// `> Panel letters`, slot by slot: Q Move and X Stop, "as in BasicCombatUnitInterfaceState" (their slots there), A
/// Build Tunnel, S Drop Off and D the Laser Turret's placeholder entry.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Build, DropOff, Move, Planned as P, Stop};
    use sc_sim::ObjectKind::Tunnel;
    [
        Some(Move),          None,          None,                          None,
        Some(Build(Tunnel)), Some(DropOff), Some(P(Planned::LaserTurret)), None,
        None,                Some(Stop),    None,                          None,
    ]
};
