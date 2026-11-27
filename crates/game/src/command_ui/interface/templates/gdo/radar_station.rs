//! The Radar Station's panel (`factions/gdo/structures/radar_station.md > Placeholder stats`, slice M6-GR): "**Scan**
//! (hotkey **Q**, targeted: left-click any point on the map): reveals a circle of radius **6 grid units** for **8
//! seconds** (128 frames); **cooldown 60 seconds** (960 frames); free. Each Radar Station has its own cooldown."
//! - Q awaits its point (`PanelCommand::Cast`), where a left-click anywhere on the map, seen or not, scans there from
//!   this station (`sc_sim::Command::Cast`, `sc_sim::AbilityArea::AnyPoint`); a click off the map is an invalid
//!   target (`crate::command_ui::resolution::target_resolution`). The cursor's circle shows the scan's 6 grid units
//!   round the point (`crate::command_ui::indicators`).
//! - Q is greyed while the station cools down (`sc_sim::Sim::can_cast`: `sc_sim::Reject::CoolingDown`), and pressed
//!   then says so ("Cooling down", `crate::command_ui::feedback::COOLING_DOWN_TEXT`).
//!
//! It is Ungroupable ("**Groupable false** (user, 2026-11-26; Q0156: each Scan press scans from one station, as the
//! Shrine; Q, Tab, Q scans two places)", design change 0422): the ActiveGroup is the one station Tabbed to, whose
//! cooldown a Scan starts. No rally point, so no C and no right-click
//! (`crate::command_ui::resolution::object_right_click`).

use sc_sim::AreaAbility;

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Radar Station's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::RadarStation);

/// Its DefaultState, slot by slot (`CommandSlot::all` order): Scan in Q.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = [
    Some(PanelCommand::Cast(AreaAbility::Scan)), None, None, None,
    None,                                        None, None, None,
    None,                                        None, None, None,
];
