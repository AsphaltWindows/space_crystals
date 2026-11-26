//! Production (`crate::production`): what every producer's queue shares. The recipes and exits are the producers'
//! defs' (`defs::ProductionDef`).

/// `factions/gdo/structures/barracks.md > BarracksInstanceState`: "BuildQueue: array of ObjectEnum (max 20,
/// including the unit currently in production)" (design change 0081). Every producer's but a pool producer's (slice
/// M5-F2), whose capacity bounds its queue instead (design change 0347 point 5: "Queues are limited by capacity, not by
/// a separate 20").
pub const QUEUE_LIMIT: usize = 20;

/// `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All`: a Release is "very fast (**4
/// frames**, placeholder)" (design change 0337).
pub const RELEASE_FRAMES: u32 = 4;
