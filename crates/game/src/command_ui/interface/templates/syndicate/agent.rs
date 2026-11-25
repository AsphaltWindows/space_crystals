//! The Agent's panel (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`, slice M4-J1). Its panel
//! letters (design change 0257, answering Q0068): "**Q** Move, **A** Build Tunnel, **S** Drop Off Resources, **D**
//! Build Laser Turret (greyed until available), **X** Stop. No Attack, Hold Position, Patrol or AttackMove buttons; the
//! Agent attacks by right-click" (`> DefaultState commands` lists the same since design change 0284). A enters the
//! Tunnel's placement (`PanelCommand::Build`), S awaits its drop-off (`PanelCommand::DropOff`), and D the Laser
//! Turret's (slice M4-S1), "Visible but greyed out until the player owns an Optics Lab" (`> DefaultState commands`):
//! greyed with the gate's reason, "Requires an Optics Lab" (`sc_sim::Reject::Requires`), as A is while the bank can't
//! pay. Its right-click (`> Right-Click Resolution`: Gather, Drop Off, Enter, Attack, Move) is the sim's one rule, as
//! every unit's (`crate::command_ui::resolution::unit_right_click`).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Agent's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Agent);

/// `> Panel letters`, slot by slot: Q Move and X Stop, "as in BasicCombatUnitInterfaceState" (their slots there), A
/// Build Tunnel, S Drop Off and D Build Laser Turret.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Build, DropOff, Move, Stop};
    use sc_sim::ObjectKind::{LaserTurret, Tunnel};
    [
        Some(Move),          None,          None,                     None,
        Some(Build(Tunnel)), Some(DropOff), Some(Build(LaserTurret)), None,
        None,                Some(Stop),    None,                     None,
    ]
};
