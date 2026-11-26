//! The economy (slice M2-B): the players' banks, their power and army caps, and the plates that mine —
//! `factions/gdo/overview.md > GDOResources`, `factions/gdo/economy.md > ExtractionPlate`,
//! `factions/colonists/overview.md > ColonistResources`.
//!
//! - **Banks** (`Player::crystals`, `Player::supplies`, and the Colonists' `Player::alloys`, `Player::essence` and
//!   `Player::flux`, plan decision T21): stored, hashed and serialized. A spend is refused when the bank is short
//!   (`Player::spend_crystals`), income saturates (`Player::credit_crystals`), and every refund goes through
//!   [`Refund`] (plan decision T7). Supplies come in by the Agents' deliveries (`gathering.rs`, slice M4-I) and
//!   nothing spends them yet (design change 0077); the refined resources are counters only until refining exists
//!   (question Q0043 point 1).
//! - **Power** ([`PowerSummary`], `power.rs`) and the **army caps** ([`ArmyCap`], `army.rs`, plan decision T20: the
//!   GDO's Unit Control, the Colonists' Beacon Capacity, the Syndicate's Tunnel Space, slice M4-B) are derived from
//!   the objects whenever they are needed, never stored, so they can't drift from them (T7): `Sim::power`,
//!   `Sim::army_cap`.
//! - **Mining** (`mining.rs`): a plate on a patch has an [`Extractor`] (`Object::extractor`), linked when the pair
//!   comes into being; it pays its owner in lumps at the end of each cycle (design change 0098), and the removal
//!   path takes a depleted patch away with its plate (`Sim::remove_objects`).
//! - **Gathering** (`gathering.rs`, slice M3-H): the Colonists' workers carry what they mine to a drop-off, and
//!   the bank is paid there; the Syndicate's Agents carry Space Crystals and Supplies to their Tunnels' hatches
//!   (`drop_off.rs`, slices M4-D and M4-I), the Supplies from the Supply Delivery Stations (`stations.rs`).
//! - **Defeat** (`defeat.rs`, slice M2-F2): the last step of the phase. A player who loses their last structure is
//!   defeated, their units become neutral, and the defeat that leaves one side standing ends the game.
//! - **The phase** ([`step`]) runs after phase 5e and before 5f (plan decision T5, `ARCHITECTURE.md > Tick
//!   pipeline`). The constructors' progress is part of it (`crate::construction`, slice M2-D1).
//!
//! Everything is integer: rates are fixed point, [`RATIO_ONE`] being 100% (plan decision T6).

pub(crate) mod army;
pub mod defeat;
pub mod drop_off;
pub mod gathering;
pub(crate) mod mining;
pub(crate) mod power;
pub mod stations;

use std::collections::BTreeMap;

pub use army::{ArmyCap, ArmyResource};
pub use defeat::Outcome;
pub use gathering::Worker;
pub use mining::Extractor;
pub use power::{PowerSummary, RATIO_ONE};

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
/// 4. the workers' timed work (`gathering::step`, slice M3-H, plan decision T17): mining takes its load from the
///    patch, a pick-up its Supply from the station (slice M4-I), and a drop-off pays the bank. After the plates, so
///    that every bank change of the tick is made here, in one order;
/// 5. the defeat check, after the producers (`defeat::step`, called by `Sim::step`, which keeps the game's outcome).
///
/// Returns the Space Crystals patches that the workers emptied: they leave the map (`core/map.md >
/// SpaceCrystalsPatch`), which is the caller's to do, through the removal path.
pub(crate) fn step(
    objects: &mut BTreeMap<ObjectId, Object>,
    players: &mut [Player],
    first_new: ObjectId,
    events: &mut Vec<SimEvent>,
) -> Vec<ObjectId> {
    let power = power::summaries(objects.values());
    crate::construction::step(objects, &power, first_new, events);
    mining::step(objects, players, &power, first_new, events);
    gathering::step(objects, players, first_new, events)
}

/// What cancelling gives back of a Space Crystals cost (plan decision T7: one helper for every refund).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Refund {
    /// The whole cost: a construction cancelled while it runs (`factions/gdo/structures/deployment_center.md >
    /// Cancellation`, `factions/gdo/economy.md > ExtractionFacility > Cancellation`), a unit taken off a queue
    /// (`factions/gdo/structures/barracks.md`: "X: Cancel Production ... refunds full cost").
    Full,
    /// A building cancelled once it is ready to place: "75% refund of Space Crystals cost (rounded down)" (the same
    /// two Cancellation sections).
    Ready,
    /// A Tunnel's expansion cancelled while it is built (slice M4-F, `sim/expansions.rs`): its share,
    /// [`EXPANSION_CANCEL_REFUND`](crate::tuning::construction::EXPANSION_CANCEL_REFUND), rounded down
    /// (`factions/syndicate/tunnel_menus.md > Expansions under construction`, design change 0254). So is a Cults
    /// building's site (`factions/cults/economy.md > Cancel Construction`: "refunds 75% of the building's Space
    /// Crystals cost (rounded down)"; `construction::site::BuildPrice::cancelled`).
    Expansion,
}

impl Refund {
    /// The Space Crystals given back of `cost`.
    pub fn of(self, cost: u32) -> u32 {
        match self {
            Refund::Full => cost,
            // In 64 bits: no overflow for any cost.
            Refund::Ready => (u64::from(cost) * 3 / 4) as u32,
            Refund::Expansion => {
                let share = u64::from(crate::tuning::construction::EXPANSION_CANCEL_REFUND);
                (u64::from(cost) * share / 100) as u32
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `factions/gdo/structures/deployment_center.md > Cancellation`, `factions/gdo/economy.md > ExtractionFacility >
    /// Cancellation`: 75% rounded down once ready (the plan's acceptance numbers for slice M2-D1).
    #[test]
    fn refunds_follow_the_cancellation_rules() {
        assert_eq!([75, 100, 200].map(|cost| Refund::Ready.of(cost)), [56, 75, 150]);
        assert_eq!([75, 100, 200].map(|cost| Refund::Full.of(cost)), [75, 100, 200]);
        // Design change 0254 (slice M4-F): "Place a Hideout (200 SC), then cancel: 150 SC come back"; a kind costing
        // 150 gives back 112.
        assert_eq!([200, 150, 1].map(|cost| Refund::Expansion.of(cost)), [150, 112, 0]);
        assert_eq!(Refund::Expansion.of(u32::MAX), 3_221_225_471);
        assert_eq!(Refund::Ready.of(u32::MAX), 3_221_225_471);
        assert_eq!((Refund::Ready.of(0), Refund::Ready.of(1)), (0, 0));
    }
}
