//! A Syndicate player's Supplies books (slice M4-L): [`SupplyBooks`], the Supplies' counterpart of
//! [`Books`](crate::testing::build_order::Books).

use std::collections::BTreeSet;

use crate::defs::Resource;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::sim::Sim;

/// A Syndicate player's Supplies for one step (`testing::syndicate`'s module docs): opened before it, closed after it,
/// as [`Books`](crate::testing::build_order::Books) keep their Space Crystals.
#[derive(Clone, Debug)]
pub struct SupplyBooks {
    player: PlayerId,
    supplies: u32,
    /// The player's sites as the step starts: a cancelled one's refund is theirs.
    sites: BTreeSet<ObjectId>,
}

impl SupplyBooks {
    pub fn open(sim: &Sim, player: PlayerId) -> Self {
        let supplies = sim.player(player).map_or(0, |p| p.supplies);
        let sites = (sim.objects())
            .filter(|o| o.owner == Some(player) && o.is_site())
            .map(|o| o.id)
            .collect();
        Self {
            player,
            supplies,
            sites,
        }
    }

    /// Closes the books after the step in which the player's commands paid `spent` Supplies, and returns the bank.
    /// Panics unless it changed by what the player's Agents delivered (their `SimEvent::ResourcesDelivered` of
    /// Supplies), less `spent`, plus the refunds: a Build's (`SimEvent::BuildRefunded` and `SimEvent::SiteCancelled`
    /// of the player's workers and sites, design change 0233) and an upgrade's (`SimEvent::UpgradeCancelled` of the
    /// player's Tunnels, design changes 0232 and 0284).
    pub fn close(self, sim: &Sim, spent: u32) -> u32 {
        let mine = |id: ObjectId| {
            let held = sim.held_unit(id).map(|held| &held.object);
            sim.object(id).or(held).is_some_and(|o| o.owner == Some(self.player))
        };
        let (mut delivered, mut refunded) = (0, 0);
        for e in sim.events() {
            match *e {
                SimEvent::ResourcesDelivered {
                    worker,
                    amount,
                    resource: Resource::Supplies,
                    ..
                } if mine(worker) => delivered += amount,
                SimEvent::BuildRefunded { worker, refund, .. } if mine(worker) => refunded += refund,
                SimEvent::SiteCancelled { site, refund, .. } if self.sites.contains(&site) => refunded += refund,
                SimEvent::UpgradeCancelled { tunnel, refund, .. } if mine(tunnel) => refunded += refund,
                _ => {}
            }
        }
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
