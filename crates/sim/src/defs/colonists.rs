//! The Colonists — `factions/colonists/`: their own rules, and one module per unit (`units/`,
//! `factions/colonists/units/*.md`) and per structure (`structures/`, `factions/colonists/structures/*.md`, the
//! economy buildings in `factions/colonists/economy.md`), with their full stat blocks (slice M3-A).
//!
//! What the stat blocks name and later slices read: MaxShields (slice M3-B, `crate::shields`), BeaconCapacityCost
//! (M3-C), the Beacon's Warp In list (M3-E). The HoverVehicle's DragMovement attributes are the Drone's movement
//! (`movement::drag`, slice M3-G). "BeaconCapacity provided - 5" is no field: every Colonists
//! building provides it (`factions/colonists/economy.md > Warp In > Beacon Capacity Retention`), a faction rule M3-C
//! derives (plan decision T14): [`BEACON_CAPACITY_PER_BUILDING`].
//!
//! The Colonists have neither Power nor a build area: their stat blocks list no Power nor BuildRadiusExtension, so
//! both are 0 (plan decision T21), and a Beacon may be deployed with "no range limit from other Colonists buildings"
//! (`factions/colonists/economy.md > Beacon Deployment > Placement`).

pub(crate) mod structures;
pub(crate) mod units;

/// `factions/colonists/overview.md > ColonistResources > Beacon Capacity`: "Total Beacon Capacity caps at 200"
/// (`economy::army`).
pub const BEACON_CAPACITY_CAP: u32 = 200;

/// `factions/colonists/economy.md > Beacon > BeaconCapacity provided - 5`, which every Colonists building provides
/// (`> Warp In > Beacon Capacity Retention`: "Every Colonists building provides 5 BeaconCapacity"; `economy::army`).
pub const BEACON_CAPACITY_PER_BUILDING: u32 = 5;

#[cfg(test)]
mod tests;
