//! Construction and the transformations it is made of: the Colonists' Warp In and Beacon Deployment
//! (`factions/colonists/economy.md > Warp In`, `> Beacon Deployment`; `crate::transform`), the GDO's construction
//! (`crate::construction`). The costs and times are the defs' recipes (`defs::Recipe`).

/// Design change 0111: after a cancel the HP is "max(1, original_max × current_before ÷ max_before)", so that "a
/// nearly dead transforming object" survives.
pub const MIN_HP_AFTER_CANCEL: f32 = 1.0;

/// `factions/syndicate/tunnel_menus.md > Expansions under construction` (design change 0254): "while building, the
/// expansion offers only **X Cancel Construction**, refunding **75%** of its cost (rounded down). The Tunnel's X
/// cancels whatever operation it is running": the share of an expansion's Space Crystals cost that cancelling its
/// construction gives back, in percent, rounded down (`economy::Refund::Expansion`, slice M4-F).
pub const EXPANSION_CANCEL_REFUND: u32 = 75;

/// Design change 0339 (`factions/cults/economy.md > Cults Building Mechanics > Group construction rules`): a Recruit
/// "builds from there (within its 8 su reach of the edge, as for mining)", the reach of its mining and drop-off
/// (`factions/cults/units/recruit.md > Gathering`: "MiningReach: 8 space units", "DropOffReach: 8 space units"), space
/// units, edge to edge: a worker on an Assist builds its site in each tick it stands this near the footprint, and a
/// group's site starts once one of its Recruits has got this near (`crate::spot_work`, slice M5-E). The builders' wall
/// round a footprint (`placement::Wall`, [`BUILD_STANDOFF_SU`] off it) is within it, even at its corners; the rows
/// further out are not.
pub const BUILD_REACH_SU: f32 = 8.0;

/// **Placeholder (not in the design: how far from the footprint and from each other builders stand)**, space units:
/// a builder (`crate::spot_work`, slice M5-E) keeps this much room round its box, so that it stands this far clear of
/// the footprint, on the builders' wall within [`BUILD_REACH_SU`], and twice this far from the builders beside it.
/// Against the footprint and each other, as the spots of a unit put on the map are, a Recruit walking up to a spot
/// between two builders already there found no room to get in, and one walking along the wall to a spot further round
/// was pressed against them (slice M5-E's runs of 8 to 30 Recruits); the snag that slice M5-F1 fixed for the walk into
/// a building by the same margin, half the reach (`holding::ENTRY_STANDOFF_SU`).
pub const BUILD_STANDOFF_SU: f32 = BUILD_REACH_SU / 2.0;

/// **Placeholder (not in the design: how finely a builder looks along the wall for room)**, space units: design change
/// 0339's wall-hug (`placement::Wall::hug`) tries the points of the wall this far apart, from the part of the
/// footprint closest to the builder, so that a builder stands within this of the room the one beside it keeps
/// ([`BUILD_STANDOFF_SU`]). A Recruit's step in a tick.
pub const HUG_STEP_SU: f32 = 4.0;

/// **Placeholder (not in the design: how soon a builder that couldn't get within reach looks again)**: in the ticks
/// that are multiples of this, a worker working from a spot beside a footprint whose walk is over out of reach of the
/// footprint looks again for a free spot within reach, and walks there (`crate::spot_work`, slice M5-E). The
/// gatherers' re-check (`tuning::gathering::RECHECK_TICKS`).
pub const RESPOT_TICKS: crate::units::Tick = 16;
