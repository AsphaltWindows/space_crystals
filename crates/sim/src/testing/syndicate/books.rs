//! A Syndicate player's Supplies books (slice M4-L): [`SupplyBooks`], the Supplies' counterpart of
//! [`Books`](crate::testing::build_order::Books).

use std::collections::BTreeSet;

use crate::defs::Resource;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::sim::Sim;
use crate::testing::build_order::{owned, refund_in};

/// A Syndicate player's Supplies for one step (`testing::syndicate`'s module docs): opened before it, closed after it,
/// as [`Books`](crate::testing::build_order::Books) keep their Space Crystals.
#[derive(Clone, Debug)]
pub struct SupplyBooks {
    player: PlayerId,
    supplies: u32,
    /// The player's objects as the step starts, whose deliveries and refunds go to them (`build_order::owned`).
    own: BTreeSet<ObjectId>,
}

impl SupplyBooks {
    pub fn open(sim: &Sim, player: PlayerId) -> Self {
        Self {
            player,
            supplies: sim.player(player).map_or(0, |p| p.supplies),
            own: owned(sim, player),
        }
    }

    /// Closes the books after the step in which the player's commands paid `spent` Supplies, and returns the bank.
    /// Panics unless it changed by what the player's Agents delivered (their `SimEvent::ResourcesDelivered` of
    /// Supplies), less `spent`, plus the refunds in Supplies (`build_order::refund_in`): a Tunnel's Build and site
    /// (`SimEvent::BuildRefunded` and `SimEvent::SiteCancelled`, design change 0233) and an upgrade's
    /// (`SimEvent::UpgradeCancelled`, design changes 0232 and 0284).
    pub fn close(self, sim: &Sim, spent: u32) -> u32 {
        let delivered: u32 = (sim.events().iter())
            .filter_map(|e| match *e {
                SimEvent::ResourcesDelivered {
                    worker,
                    amount,
                    resource: Resource::Supplies,
                    ..
                } => self.own.contains(&worker).then_some(amount),
                _ => None,
            })
            .sum();
        let refunded: u32 = (sim.events().iter())
            .filter_map(|e| refund_in(e, Resource::Supplies, &self.own))
            .sum();
        let bank = sim.player(self.player).map_or(0, |p| p.supplies);
        assert_eq!(
            i64::from(bank),
            i64::from(self.supplies) + i64::from(delivered) - i64::from(spent) + i64::from(refunded),
            "tick {}: player {}'s Supplies went from {} to {bank}, but the Agents delivered {delivered}, the commands \
             paid {spent} and {refunded} came back",
            sim.tick(),
            self.player.0,
            self.supplies,
        );
        bank
    }
}
