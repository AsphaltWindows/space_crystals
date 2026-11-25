//! Construction and the transformations it is made of: the Colonists' Warp In and Beacon Deployment
//! (`factions/colonists/economy.md > Warp In`, `> Beacon Deployment`; `crate::transform`), the GDO's construction
//! (`crate::construction`). The costs and times are the defs' recipes (`defs::Recipe`).

/// Design change 0111: after a cancel the HP is "max(1, original_max × current_before ÷ max_before)", so that "a
/// nearly dead transforming object" survives.
pub const MIN_HP_AFTER_CANCEL: f32 = 1.0;
