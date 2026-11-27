//! Gathering: how often a waiting worker looks round, and how long one whose return leg ended short stands before it
//! tries again (`factions/colonists/units/drone.md > Gathering`; `factions/syndicate/units/agent.md > Gathering`;
//! `crate::orders::gather`). The workers' loads, durations and reaches are their defs' (`defs::WorkerDef`).

use crate::units::Tick;

tunables! {
    /// The tunables of gathering (module docs).
    Gathering;

    /// `factions/colonists/units/drone.md > Patch Occupancy > Waiting and re-checking`, item 2 (design change 0132): a
    /// waiting worker looks round again at this interval, in the ticks that are multiples of it. The design's
    /// placeholder: 16 frames, 1 second.
    ///
    /// **One phase for every worker** (the design gives the interval, not its phase): all those waiting look round in
    /// the same tick, so a patch that freed up meanwhile goes to the lowest id of them (design change 0133 with the
    /// Director's choice, `orders::gather::Claims`), and the others wait on. Counted from each worker's own start, they
    /// looked round in different ticks, and every one of them that did before the first got there went for the same
    /// patch: measured on the reference field, that cost 0.5 Space Crystals per second with 20 and 24 Drones, and up to
    /// 100 ticks of the worst gap between deliveries (`tests/gathering_soak.rs`).
    recheck_ticks: Tick = 16,

    /// A worker whose leg back to its drop-off ended short of it stands for this many ticks before it tries again
    /// (`orders::gather`'s module docs), and it never gives up: design change 0255 settles that for Agents ("Agents
    /// waiting at the hatch wait forever") and, by its implementation note, for Drones. The interval is a placeholder
    /// the design doesn't name: 2 seconds.
    ///
    /// **Not the wait of an Agent short of its hatch** (design change 0315, `factions/syndicate/units/agent.md >
    /// Gathering > Gathering rules`, "Waiting at the hatch": out of reach it "keeps actively moving toward the hatch"):
    /// that leg doesn't end behind standing units, and begins again at once when the mover gives it up. For an Agent
    /// the interval is left only for a hatch it has no way to (`orders::gather::legs::return_leg`).
    leg_retry_ticks: Tick = 32,
}
