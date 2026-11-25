//! Construction and the transformations it is made of: the Colonists' Warp In and Beacon Deployment
//! (`factions/colonists/economy.md > Warp In`, `> Beacon Deployment`; `crate::transform`), the GDO's construction
//! (`crate::construction`). The costs and times are the defs' recipes (`defs::Recipe`).

/// Design change 0111: after a cancel the HP is "max(1, original_max × current_before ÷ max_before)", so that "a
/// nearly dead transforming object" survives.
pub const MIN_HP_AFTER_CANCEL: f32 = 1.0;

/// `factions/syndicate/tunnels.md > Expansions under construction` (design change 0254): "while building, the
/// expansion offers only **X Cancel Construction**, refunding **75%** of its cost (rounded down). The Tunnel's X
/// cancels whatever operation it is running": the share of an expansion's Space Crystals cost that cancelling its
/// construction gives back, in percent, rounded down (`economy::Refund::Expansion`, slice M4-F).
pub const EXPANSION_CANCEL_REFUND: u32 = 75;
