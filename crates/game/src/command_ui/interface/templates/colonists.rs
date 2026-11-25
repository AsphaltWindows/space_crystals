//! The Colonists' panels (slices M3-J1 and M3-J2, design change 0124): the Beacon's, with its two Warp In menus
//! ([`beacon`]), the Drone's ([`drone`]), the Egress Node's ([`egress_node`]), and the one of an object that transforms
//! ([`TRANSFORMING_GRID`]). The portals have the Production template (`super::production_grid`, slice M3-D), with
//! their units not in the game yet in their slots ([`robo_portal`], [`bio_portal`]: [`Planned`], `super::planned`).

pub mod beacon;
pub mod bio_portal;
pub mod drone;
pub mod egress_node;
pub mod robo_portal;

use sc_sim::{ObjectKind, transform};

use super::super::PanelCommand;
use super::Planned;

/// `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel` (design change 0116): "A
/// Beacon under deployment and a building under Warp In show only the Cancel command in their CommandPanel (no rally
/// point, no other commands)", in X ("Cancel's hotkey: X"): the panel of their group, the unfinished buildings of a
/// type (design change 0185: "with its own panel (e.g. Cancel only)").
#[rustfmt::skip]
pub const TRANSFORMING_GRID: [Option<PanelCommand>; 12] = [
    None, None,                                None, None,
    None, None,                                None, None,
    None, Some(PanelCommand::CancelTransform), None, None,
];

/// Whether objects of `kind` warp buildings in: the Beacon, with its Constructs List (`StructureDef::warp_in`).
pub(super) fn warps_in(kind: ObjectKind) -> bool {
    !transform::warp_in_recipes(kind).is_empty()
}
