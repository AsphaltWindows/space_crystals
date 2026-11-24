//! Which door a finished unit leaves through (plan decision T19): `factions/colonists/structures.md > Bio Portal >
//! Exit Behavior`, which the Robo Portal follows over its four sides (**placeholder**, question Q0041 point 3):
//!
//! > A produced unit exits from whichever eligible A side is closest to the RallyPoint. If no RallyPoint is set, or
//! > the closest side is blocked (obstructed, not currently traversable), the unit exits from whichever eligible A
//! > side is unblocked, chosen at random among unblocked candidates.
//!
//! and `core/entities.md > Structure Type > Exit Doorway`: "a side whose doorway is occupied counts as blocked, and
//! the building pauses only when every eligible doorway is occupied".
//!
//! - **Closest**: the straight line from the middle of the door to the RallyPoint, ties going top, right, bottom,
//!   left as placed (**placeholder**, question Q0041 point 4). An object RallyPoint is where the owner last saw the
//!   object (`Rally::point`), so the choice never tells the owner where a hidden enemy is.
//! - **Blocked**: the door is occupied (a unit the building released is still in it), or obstructed: no way out
//!   of it is clear of buildings, cliffs and the map's edge (`Door::has_way_out`). Units don't obstruct a door: own
//!   idle ones step aside and enemies are shot (design change 0080).
//! - **Reading, to confirm** (reported to the Director): when every door that isn't occupied is obstructed, the
//!   building doesn't pause ("only when every eligible doorway is occupied"): the unit leaves through the closest of
//!   them if that one isn't occupied, else a random one, and stays stuck in its doorway as a Barracks' does against
//!   a wall (`> Exit Doorway`, point 5).
//! - **Random**: a draw from the sim's Rng, made only when two or more doors qualify, so a building with one exit
//!   side (the Barracks) never draws.

use glam::Vec2;

use super::Door;

/// One door of a building with a unit ready to leave, as the building finds it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct DoorState {
    pub door: Door,
    /// A unit the building released is still in it.
    pub occupied: bool,
    /// No way out of it is clear of solid tiles (`Door::has_way_out`).
    pub obstructed: bool,
}

/// The index in `doors` (the building's doors in `Doorway::doors`' order) of the door the next unit leaves through,
/// for a building whose RallyPoint lies at `rally`, if it has one (module docs). `draw(n)` draws a uniform index
/// below `n` from the sim's Rng; it is called only to choose among two or more doors. `None` while every door is
/// occupied: the building pauses.
pub(crate) fn choose(doors: &[DoorState], rally: Option<Vec2>, draw: impl FnOnce(u32) -> u32) -> Option<usize> {
    let free: Vec<usize> = (0..doors.len()).filter(|&i| !doors[i].occupied).collect();
    let unblocked: Vec<usize> = free.iter().copied().filter(|&i| !doors[i].obstructed).collect();
    let candidates = if unblocked.is_empty() { free } else { unblocked };
    if let Some(rally) = rally {
        let distance = |i: usize| doors[i].door.middle().distance_squared(rally);
        // `min_by` keeps the first of equal ones: the doors are in top, right, bottom, left order.
        let closest = (0..doors.len()).min_by(|&a, &b| distance(a).total_cmp(&distance(b)));
        if let Some(closest) = closest.filter(|closest| candidates.contains(closest)) {
            return Some(closest);
        }
    }
    match candidates.len() {
        0 => None,
        1 => Some(candidates[0]),
        n => Some(candidates[draw(n as u32) as usize]),
    }
}
