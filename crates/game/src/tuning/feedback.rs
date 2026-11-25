//! How long feedback shows, ticks: the flash of an order given, a refused command's message, a shot, a death, a blast
//! and a HUD notice. The design gives none of these times.

use sc_sim::Tick;

/// How long the flash of a given order lasts, ticks (`crate::command_ui::indicators`). Placeholder (not in the
/// design): half a second.
pub const FLASH_TICKS: u32 = 8;

/// How long a refused command's message shows, ticks (2 s; `crate::command_ui::feedback`). Placeholder (not in the
/// design): the design says "short".
pub const FEEDBACK_TICKS: u32 = 32;

/// How long a tracer shows, ticks (`crate::combat_effects`). Placeholder (not in the design) until the art direction
/// (Q0001).
pub const TRACER_TICKS: u32 = 5;

/// How long a death marker shows, ticks (`crate::combat_effects`). Placeholder (not in the design) until the art
/// direction (Q0001).
pub const DEATH_MARKER_TICKS: u32 = 40;

/// How long a blast's ring shows, ticks (`crate::combat_effects::blasts`, slice M4-O). Placeholder (not in the design)
/// until the art direction (Q0001).
pub const BLAST_TICKS: u32 = 12;

/// How long a HUD notice shows, ticks (5 s; `crate::hud::notices`). Placeholder (not in the design).
pub const NOTICE_TICKS: Tick = 80;
