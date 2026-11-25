//! Production (`crate::production`): what every producer's queue shares. The recipes and exits are the producers'
//! defs' (`defs::ProductionDef`).

/// `factions/gdo/structures.md > Barracks > BarracksInstanceState`: "BuildQueue: array of ObjectEnum (max 20,
/// including the unit currently in production)" (design change 0081). Every producer's, so far.
pub const QUEUE_LIMIT: usize = 20;
