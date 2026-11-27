//! How long feedback shows, ticks: the flash of an order given, a refused command's message, a shot, a death, a blast
//! and a HUD notice, none of whose times the design gives; and (slice M5-MM2) the minimap's alerts, how long a ping
//! shows and how often an area pings, whose placeholders the design gives (`systems/controls/hud.md > Minimap >
//! Alerts`).

use sc_sim::Tick;

super::tunables! {
    /// How long feedback shows (module docs; `client/feedback.json`).
    FeedbackTuning;

    /// How long the flash of a given order lasts, ticks (`crate::command_ui::indicators`). Placeholder (not in the
    /// design): half a second.
    flash_ticks: u32 = 8,

    /// How long a refused command's message shows, ticks (2 s; `crate::command_ui::feedback`). Placeholder (not in the
    /// design): the design says "short".
    feedback_ticks: u32 = 32,

    /// How long a tracer shows, ticks (`crate::combat_effects`). Placeholder (not in the design) until the art
    /// direction (Q0001).
    tracer_ticks: u32 = 5,

    /// How long a death marker shows, ticks (`crate::combat_effects`). Placeholder (not in the design) until the art
    /// direction (Q0001).
    death_marker_ticks: u32 = 40,

    /// How long a blast's ring shows, ticks (`crate::combat_effects::blasts`, slice M4-O). Placeholder (not in the
    /// design) until the art direction (Q0001).
    blast_ticks: u32 = 12,

    /// How long a HUD notice shows, ticks (5 s; `crate::hud::notices`). Placeholder (not in the design).
    notice_ticks: Tick = 80,

    /// (M5-MM2) How long a minimap alert's ping shows, ticks (2 s; `crate::hud::minimap::alerts`):
    /// `systems/controls/hud.md > Minimap > Alerts`: "a ring that shrinks and fades over about **2 s**
    /// (placeholder)". At most the throttle ([`FeedbackTuning::ping_throttle_ticks`]).
    ping_ticks: Tick = 32,

    /// (M5-MM2) The alerts' throttle, ticks (10 s): `systems/controls/hud.md > Minimap > Alerts`: "at most **one ping
    /// per area every 10 s** (placeholder; an area: within about 8 grid units of an earlier ping still in its 10 s)".
    ping_throttle_ticks: Tick = 160,

    /// (M5-MM2) The throttle's area, grid units: within this distance of a ping made in the last
    /// [`FeedbackTuning::ping_throttle_ticks`], a hit pings no more (`systems/controls/hud.md > Minimap > Alerts`:
    /// "within about 8 grid units", a placeholder).
    ping_area_gu: f32 = 8.0,

    /// (M5-N11) How long a cast shows, ticks: a second (`crate::combat_effects::casts`). Placeholder (not in the
    /// design) until the art direction (Q0001).
    cast_ticks: u32 = 16,

    /// (M5-X2) How long a data directory's refusal shows in the feedback line, ticks (6 s; `crate::data`,
    /// `crate::command_ui::feedback`). Placeholder (not in the design): long enough to read a file's path and a
    /// member's.
    refusal_ticks: u32 = 96,
}
