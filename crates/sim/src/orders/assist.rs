//! An Assist (slice M5-E, `OrderKind::Assist`): a worker builds a site from outside, from its own spot beside the
//! footprint — `factions/cults/economy.md > Assist Construction Command` ("Left Click on an in-progress building";
//! "Once the selected Recruits reach the building they join construction from outside, increasing build speed"), and
//! `factions/cults/economy.md > Cults Building Mechanics` ("Construction speed scales proportionally with the number of
//! Recruits currently assigned"). Every Build of a group's becomes one as its site starts (plan decision T59,
//! `sim/builds/shared.rs`).
//!
//! The behaviour holds the worker's spot ([`SpotWork`]) and does nothing else: the order is carried out by the steps
//! around phase 5a, which read and write what a behaviour may not.
//! - **The spot and the walk** are the spots' step's, right after this phase (`crate::spot_work`, `Sim::step_spots`):
//!   it gives the worker its own spot beside the footprint, the first free one of T62's search (plan decision T62:
//!   ring 0 against the footprint first, **Placeholder (Q0097 point 1)**: "each builder at its own spot within 8 su of
//!   the footprint's edge"; further out, where it waits and doesn't count, once ring 0 is full, **Placeholder (Q0097
//!   point 8)**), and sends it there.
//! - **The building** is the sites' step's (`sim/sites.rs`): in each tick the site gains one frame for each worker on
//!   an Assist of it that stands within reach of its footprint's edge (**Placeholder (Q0097 point 5)**: "only those at
//!   the site"); one still on its way doesn't count, and a worker keeps building when attacked, as a gatherer keeps
//!   gathering (**Placeholder (Q0097 point 6)**): an order holds the unit, and no auto-targeting takes it away.
//! - **The end**: the order completes as its site does (`Sim::complete_site`), and its queued orders go on from the
//!   next tick (a Shift chain's next Build); it ends with its site if the site goes, cancelled or destroyed (the
//!   removal path: the order names it), and the worker idles or takes its queued orders (**Placeholder (Q0098 point
//!   4)**). Never complete by itself.

use super::state::SpotWork;

/// An Assist starts (`orders::start`): the worker stands until the spots' step sends it to its spot (module docs).
pub(super) fn begin() -> SpotWork {
    SpotWork::default()
}

/// One tick of an Assist: nothing for the behaviour to do (module docs). Never complete.
pub(super) fn run(_state: &mut SpotWork) -> bool {
    false
}
