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
//!   it gives the worker its own spot beside the footprint, the first free one of design change 0339's search
//!   (`factions/cults/economy.md > Cults Building Mechanics > Group construction rules`: "the **part of the footprint
//!   closest to it**; if that part is taken (blocked), it **wall-hugs** along the footprint until it reaches a part
//!   where it can work"), on the builders' wall first, within reach; further out, where it waits and doesn't count,
//!   once that wall is full; and it sends the worker there, round the footprint.
//! - **The building** is the sites' step's (`sim/sites.rs`): in each tick the site gains one frame for each worker on
//!   an Assist of it that stands within reach of its footprint's edge, its walk over (0339: "Every frame, each Recruit
//!   that is building during that frame adds 1. Recruits still walking, waiting for a spot or doing anything else add
//!   nothing"); and a worker keeps building when attacked, as a gatherer keeps gathering (0339: "builders **keep
//!   building** (as gatherers keep gathering)"): an order holds the unit, and no auto-targeting takes it away.
//! - **The end**: the order completes as its site does (`Sim::complete_site`), and its queued orders go on from the
//!   next tick (a Shift chain's next Build); it ends with its site if the site goes, cancelled or destroyed (the
//!   removal path: the order names it), and the worker idles or takes its queued orders (design change 0340,
//!   `factions/cults/economy.md > Cults Building Mechanics > Group construction refunds`: "Recruits with more orders
//!   queued (e.g. Shift-queued buildings) **go on to them**; with none, they go idle"). Never complete by itself.

use super::state::SpotWork;

/// An Assist starts (`orders::start`): the worker stands until the spots' step sends it to its spot (module docs).
pub(super) fn begin() -> SpotWork {
    SpotWork::default()
}

/// One tick of an Assist: nothing for the behaviour to do (module docs). Never complete.
pub(super) fn run(_state: &mut SpotWork) -> bool {
    false
}
