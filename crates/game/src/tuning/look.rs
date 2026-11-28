//! (Slice M5-X2) The looks a playtest tunes, all placeholders until the art direction (Q0001) settles them: the
//! Shroud's clouds (`crate::shroud_view`), the bars over the units (`crate::combat_effects`: HP, Shields and energy)
//! and how high the air units fly over their shadows (`crate::object_render::flight`); (M6-SB) how the sludge's
//! patches end (`crate::patches_view`). Their colours and the other drawing sizes stay with the drawing code.

use sc_sim::Tick;

super::tunables! {
    /// The looks a playtest tunes (module docs; `client/look.json`).
    LookTuning;

    /// The Shroud's haze, its opacity out of 255 (`crate::shroud_view`): under half opaque, so that the units under
    /// it show. Placeholder (Q0001).
    cloud_alpha: u8 = 100,

    /// A release's flash over a cloud, its opacity out of 255 as it starts, under which the units still show
    /// (`crate::shroud_view`). Placeholder (Q0001).
    cloud_flash_alpha: f32 = 170.0,

    /// How long a release flashes, fading, ticks (`crate::shroud_view`). Placeholder (Q0001).
    release_ticks: Tick = 6,

    /// The height of a bar over a unit or a building (HP, Shields, energy), su (`crate::combat_effects`). Placeholder
    /// (Q0001).
    bar_height: f32 = 4.0,

    /// How far over the corners of a unit's silhouette (clear of a selection ring, which lies 7 su outside them:
    /// `crate::selection::markers`), or over a tile-based object's footprint, its HP bar lies, su
    /// (`crate::combat_effects`). Placeholder (Q0001).
    bar_gap: f32 = 9.0,

    /// The narrowest bar, su (`crate::combat_effects`). Placeholder (Q0001).
    bar_min_width: f32 = 24.0,

    /// How high an air unit's body is drawn above the ground under it, in terrain levels: its shadow's offset down
    /// the screen (`crate::coords::lift`: 16 su a level, so 48 su, more than a Sparrow's own height;
    /// `crate::object_render::flight`). Placeholder (Q0001).
    flight_lift_levels: f32 = 3.0,

    /// (M5-N11) How far before its run's end an aircraft on its takeoff run leaves the runway, su: 2 grid units, about
    /// half the Airstrip's runway (4 grid units long), so that a Sparrow rolls along its first half and rises along
    /// the rest (`crate::object_render::flight`). Placeholder (Q0001).
    lift_off_su: f32 = 128.0,

    /// (M6-SB) How long a patch of sludge shrinks before it goes, ticks: its last 2 s, as long as a unit stays slowed
    /// after it leaves one (`crate::patches_view`). Placeholder (Q0001).
    sludge_fade_ticks: Tick = 32,
}
