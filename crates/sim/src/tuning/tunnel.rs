//! The Tunnel Network's ways in and out (`factions/syndicate/tunnels.md > Tunnel Network`, `> EjectMenu`;
//! `crate::network`): where Side A's paths lie, how near a unit goes in, how far it walks out, and (M6-SD) where the
//! Tier 3 air doors lie, how big they are and how often an aircraft passes one. The transit delay and the regeneration
//! inside are the Tunnel's def's (`defs::TunnelDef`), and how long an aircraft rises and descends at a door its type's
//! (`defs::UnitDef::air_door_ticks`). And the decay of the expansions a lost Tunnel leaves neutral
//! (`factions/syndicate/tunnels.md > Losing Tunnels`, `sim/tunnels.rs`), and the regeneration of those it keeps (slice
//! M5-N8).

use crate::units::{SU_PER_GU, Tick};

tunables! {
    /// The tunables of tunnel (module docs).
    Tunnel;

    /// How far Side A's paths lie from the middle of the side, space units: the exit path towards Side B, the entry
    /// path towards Side D (design change 0248). The middles of the two paths the client draws
    /// (`object_render::tunnel`, which reads this), 60 su apart, so that a unit up to 60 su wide on one never touches
    /// one on the other ("entering and exiting units never block each other"). **Placeholder**: the design gives the
    /// halves, not where in them (question Q0063 point 2 proposed "centred on its half"; the drawing of slice M4-A came
    /// first).
    path_offset_su: f32 = 30.0,

    /// Question Q0063 point 2, as proposed (design change 0247: "As proposed"): "A unit goes in when its silhouette is
    /// within 8 su of the entry spot (DropOffReach's value, as a placeholder)".
    entry_reach_su: f32 = 8.0,

    /// `factions/syndicate/tunnel_menus.md > EjectMenu` (design change 0250): "With no rally point, a unit walks about
    /// 1 grid unit straight out of Side A and idles".
    walk_out_su: f32 = SU_PER_GU,

    /// `factions/syndicate/tunnels.md > Losing Tunnels` (design change 0258 point 1, answering Q0069): a neutral
    /// expansion "slowly loses HP until destroyed. Decay is a percentage of MaxHP per second ...: 1% per second,
    /// placeholder (a Hideout lasts 100 s); the number is to be tuned". Whole percents of its MaxHP per second
    /// (`sim/tunnels.rs`, which counts it out exactly per tick). **Placeholder** (design change 0258: "1% per second is
    /// a placeholder").
    decay_percent_per_second: u32 = 1,

    /// (M5-N8) `factions/syndicate/structures/README.md > Tunnel Expansions` (design changes 0329 and 0330): expansions
    /// are "**Not repaired; they regenerate**": "a completed expansion **regenerates HP gradually** while it is part of
    /// a Tunnel system (its Tunnel is alive). Placeholder rate (designer): **1% of MaxHP per second**, always on,
    /// **including during combat**". The share of its MaxHP regained a second (`sim/expansions.rs`,
    /// `crate::regeneration`). **Placeholder** (design change 0329).
    expansion_regen_per_second: f32 = 0.01,

    /// (M6-SD) How far each of a Tunnel's two air doors lies from its centre, space units, on the line across it from
    /// Side D to Side B, halfway between Side A and Side C (`network::Paths::air_entry`, `network::Paths::air_exit`):
    /// the entry door towards D, the exit door towards B (`factions/syndicate/tunnel_menus.md > EjectMenu`, Q0064: "The
    /// **air doors are on the Tunnel's roof**: the entry door over the entry path's half of the Tunnel, the exit door
    /// over the exit path's half"). Each door so lies in the middle of its half, clear of the tag in the middle and of
    /// the hatches, where the client draws it (`object_render::tunnel`, which reads this). Only a Tier 3 Tunnel has
    /// them ("**Tier 3**: adds **two horizontal circular sliding doors**", `factions/syndicate/tunnels.md > Tunnel >
    /// Look by tier`). **Placeholder (not in the design: the points)**.
    air_door_offset_su: f32 = 52.0,

    /// (M6-SD) The radius of an air door, space units: an aircraft is over a door while its centre is within it
    /// (`network::over_door`; the plan's D46, as design change 0431 keeps it: "an air unit goes in once its centre is
    /// over the entry door"), and the client draws it so: a 48 su disc, for the Phantom's 44 su. **Placeholder (not in
    /// the design: the doors' size)**.
    air_door_radius_su: f32 = 24.0,

    /// (M6-SD) `factions/syndicate/tunnel_menus.md > EjectMenu` (design change 0251): "since aircraft don't collide,
    /// the door has its own **exit rate: one aircraft per 8 frames (0.5 s), placeholder** (the same rate for the air
    /// entry door)": the ticks from one aircraft's coming out of a Tunnel's air exit door to the next's
    /// (`TunnelState::air_out`), and from one aircraft's descent into its air entry door to the next's
    /// (`TunnelState::air_in`). **Placeholder** (the design's).
    air_door_interval_ticks: Tick = 8,
}
