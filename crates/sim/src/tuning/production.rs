//! Production (`crate::production`): what every producer's queue shares. The recipes and exits are the producers'
//! defs' (`defs::ProductionDef`).

tunables! {
    /// The tunables of production (module docs).
    Production;

    /// `factions/gdo/structures/barracks.md > BarracksInstanceState`: "BuildQueue: array of ObjectEnum (max 20,
    /// including the unit currently in production)" (design change 0081). Every producer's but a pool producer's (slice
    /// M5-F2), whose capacity bounds its queue instead (design change 0347 point 5: "Queues are limited by capacity,
    /// not by a separate 20").
    queue_limit: usize = 20,

    /// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: a Release is "very fast (**4
    /// frames**, placeholder)" (design change 0337).
    release_frames: u32 = 4,

    /// (M6-GF) **Placeholder (not in the design: how finely a released vehicle looks round its taken rally point)**,
    /// space units: design change 0451's search for "the free spot nearest the point where it fits"
    /// (`core/entities.md > Entrance and exit > Exit Doorway` point 7, `sim/producers/park.rs`) tries the rings round the
    /// point this far apart, and the points of each ring this far apart along it.
    parking_step_su: f32 = 8.0,

    /// (M6-GF) **Placeholder (not in the design: how far from its taken rally point a released vehicle may park)**,
    /// grid units: the search of [`Production::parking_step_su`] goes this far out from the point, the radius of
    /// MovingToLocation's "Occupied target location" (`systems/controls/command_behaviors.md`: "within **3 grid
    /// units** of TargetLocation"). A vehicle that finds no free spot within it drives to the point as before.
    parking_radius_gu: f32 = 3.0,

    /// (M6-GF) **Placeholder (not in the design: how much room a released vehicle wants on its way to a parking
    /// spot)**, space units: the spot is one it drives to straight from the doorway without meeting the vehicles
    /// parked round the point, the lane there as wide as the vehicle and this much more on either side
    /// (`sim/producers/park.rs`, *Free*).
    parking_lane_margin_su: f32 = 8.0,
}
