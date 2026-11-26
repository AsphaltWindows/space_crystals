//! The Agent's panel (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState`, slice M4-J1; design changes
//! 0359 and 0398). Its panel letters: "**Q** Move, **W** Build Tunnel (moved from A, 2026-11-27 [...]), **A** Attack
//! (user, 2026-11-27: A is Attack on every unit [...]) (an enemy, or Attack Move on ground), **S** Drop Off Resources,
//! **D** Repair (2026-11-26, matching the Recruit's panel), **X** Stop. No Hold Position or Patrol buttons. These
//! letters replace the older ones listed below where they differ" (its Unit Commands' "Build Tunnel — via panel button
//! A"). W enters the Tunnel's placement (`PanelCommand::Build`), A awaits the target of an Attack, as
//! BasicCombatUnitInterfaceState's does, and S awaits its drop-off (`PanelCommand::DropOff`). D, "Left-click a damaged
//! own Tunnel or Laser Turret to repair it" (`> DefaultState commands`), awaits its target as the Recruit's does
//! (`PanelCommand::Repair`, slice M5-N8): the panel's way to repair a Tunnel, which a right-click enters. Slice M4-S1's
//! D, the Agent's Build Laser Turret, is gone with design change 0359: a turret is ordered from a Tunnel's ExpandMenu
//! alone (`super::tunnel`). Its right-click (`> Right-Click Resolution`: Gather, Drop Off, Enter, Repair, Attack, Move)
//! is the sim's one rule, as every unit's (`crate::command_ui::resolution::unit_right_click`).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Agent's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Agent);

/// `> Panel letters`, slot by slot: Q Move, A Attack and X Stop, "as in BasicCombatUnitInterfaceState" (their slots
/// there), W Build Tunnel, S Drop Off and D Repair, the Recruit's slots (`super::super::cults::recruit::GRID`).
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Attack, Build, DropOff, Move, Repair, Stop};
    use sc_sim::ObjectKind::Tunnel;
    [
        Some(Move),   Some(Build(Tunnel)), None,         None,
        Some(Attack), Some(DropOff),       Some(Repair), None,
        None,         Some(Stop),          None,         None,
    ]
};
