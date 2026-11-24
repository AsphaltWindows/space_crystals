//! The economy (slice M2-B): the players' banks, their power and Unit Control, and the plates that mine —
//! `factions/gdo/overview.md > GDOResources`, `factions/gdo/economy.md > ExtractionPlate`.
//!
//! - **Banks** (`Player::crystals`, `Player::supplies`): stored, hashed and serialized. A spend is refused when the
//!   bank is short (`Player::spend_crystals`), income saturates (`Player::credit_crystals`), and every refund goes
//!   through [`Refund`] (plan decision T7). Supplies are a counter only (design change 0077).
//! - **Power** ([`PowerSummary`], `power.rs`) and **Unit Control** (`unit_control.rs`) are derived from the objects
//!   whenever they are needed, never stored, so they can't drift from them (T7): `Sim::power`,
//!   `Sim::unit_control`.
//! - **Mining** (`mining.rs`): a plate on a patch has an [`Extractor`] (`Object::extractor`), linked when the pair
//!   comes into being; it pays its owner in lumps at the end of each cycle (design change 0098), and the removal
//!   path takes a depleted patch away with its plate (`Sim::remove_objects`).
//! - **The phase** ([`step`]) runs after phase 5e and before 5f (plan decision T5, `ARCHITECTURE.md > Tick
//!   pipeline`). The constructors' progress is part of it (`crate::construction`, slice M2-D1).
//!
//! Everything is integer: rates are fixed point, [`RATIO_ONE`] being 100% (plan decision T6).

pub(crate) mod mining;
pub(crate) mod power;
pub(crate) mod unit_control;

use std::collections::BTreeMap;

pub use mining::Extractor;
pub use power::{PowerSummary, RATIO_ONE};
pub use unit_control::UNIT_CONTROL_CAP;

use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::player::Player;

/// The economy phase of one tick (plan decision T5), after the deaths of phase 5e: a building destroyed in this tick
/// counts for nothing in it, one placed by this tick's commands counts already (`factions/gdo/overview.md >
/// GDOResources > Power > When a building's Power counts`: from the tick it is placed until the tick it is
/// destroyed). In order:
/// 1. every player's power, from the structures present ([`PowerSummary`]);
/// 2. the constructors (`crate::construction::step`), then (M2-D2) the producers, each at its speed
///    ([`PowerSummary::speed`]);
/// 3. the plates mine (`mining::step`), the plates created this tick from the next one on;
/// 4. (M2-F) the defeat check.
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) {
    let power = power::summaries(objects.values());
    crate::construction::step(objects, &power, first_new, events);
    mining::step(objects, players, &power, first_new, events);
}

/// What cancelling gives back of a Space Crystals cost (plan decision T7: one helper for every refund).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Refund {
    /// The whole cost: a construction cancelled while it runs (`factions/gdo/structures.md > DeploymentCenter >
    /// Cancellation`, `factions/gdo/economy.md > ExtractionFacility > Cancellation`), a unit taken off a queue
    /// (`factions/gdo/structures.md > Barracks`: "X: Cancel Production ... refunds full cost").
    Full,
    /// A building cancelled once it is ready to place: "75% refund of Space Crystals cost (rounded down)"
    /// (`> Cancellation`).
    Ready,
}

impl Refund {
    /// The Space Crystals given back of `cost`.
    pub fn of(self, cost: u32) -> u32 {
        match self {
            Refund::Full => cost,
            // In 64 bits: no overflow for any cost.
            Refund::Ready => (u64::from(cost) * 3 / 4) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `> Cancellation`: 75% rounded down once ready (the plan's acceptance numbers for slice M2-D1).
    #[test]
    fn refunds_follow_the_cancellation_rules() {
        assert_eq!([75, 100, 200].map(|cost| Refund::Ready.of(cost)), [56, 75, 150]);
        assert_eq!([75, 100, 200].map(|cost| Refund::Full.of(cost)), [75, 100, 200]);
        assert_eq!(Refund::Ready.of(u32::MAX), 3_221_225_471);
        assert_eq!((Refund::Ready.of(0), Refund::Ready.of(1)), (0, 0));
    }
}
