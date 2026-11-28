//! A Tunnel's upgrades in a game (slice M4-G; `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`,
//! `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`, `factions/syndicate/tunnel_area_and_space.md >
//! Tunnel Area Non-Overlap Rule`; design changes 0229, 0231, 0232, 0237, 0242, 0249, 0283 and 0284): "**Q** Upgrade
//! Tunnel — upgrades to next tier", as the Tunnel's one operation (`tunnel::Operation::Upgrading`).
//!
//! - **The price** ([`Sim::upgrade_price`]; one function, [`Sim::price_to`], for the order and every refund):
//!   "**Upgrade to Tier 2**: 2 + 2 x (number of T2+ Tunnels owned)", "**Upgrade to Tier 3**: 3 + 3 x (number of T3
//!   Tunnels owned)", the target tier's rule (`defs::UpgradeDef`), counting the owner's other Tunnels whose tier, or the
//!   tier they upgrade to, is the target or above: "Higher-tier Tunnels count toward lower-tier cost calculations", and
//!   "**Upgrades in progress count** ... Two upgrades started together cost 2 and 4, not 2 each". Never the Tunnel
//!   itself, so that the same function gives "the price that upgrade would cost if ordered right now (not counting
//!   itself)", what a cancel refunds. A site is at Tier 1 and never counts.
//! - **The gate** ([`Sim::can_upgrade`], `TunnelCommand::Upgrade`), in order: the Tunnel is the player's and in
//!   operation, as for every Tunnel command; a tier above its own is in its table ([`Reject::TopTier`]: "Unavailable
//!   if already Tier 3"); it performs no operation ([`Reject::OperationUnderWay`]: "or if currently performing an
//!   operation"); the bank pays the price ([`Reject::InsufficientSupplies`]). Then the truth ([`Sim::upgrade_blocked`],
//!   [`Reject::UpgradeBlocked`]): its area at the target tier lies wholly on the map and on Drillable cells (design
//!   change 0242: "An **upgrade** is likewise blocked if the larger area would include a non-Drillable or off-map
//!   cell"; 0283: 3 tiles to the map's edge for Tier 2, 4 for Tier 3), and overlaps no other Tunnel's area, whoever
//!   owns it, a site's included, each as it holds it, another upgrade's reserved area included ("**Upgrades** are
//!   blocked by any Tunnel's area, own or enemy (the upgrade check uses the truth; the upgrading Tunnel's owner learns
//!   that something blocks it)"; design change 0289 point 3). The refusal never says what blocks it.
//! - **The start** ([`Sim::start_upgrade`]): "**Paid when the upgrade is ordered**". From then on the Tunnel holds its
//!   area at the target tier against every other placement and upgrade ("**The future area is reserved**":
//!   `tunnel::TunnelState::reserved_tier`, `placement::Occupant::of`), while its tier stays the one it had: transit,
//!   the exits, Tunnel Space and reclaiming read it (design change 0249: "until the upgrade has fully finished, a Tunnel
//!   only accepts, and only lets out, units supported by the tier it had **before** the upgrade began";
//!   `factions/syndicate/tunnels.md > Losing Tunnels`: "a Tunnel that is mid-upgrade counts at its current
//!   (pre-upgrade) tier").
//! - **The frames** ([`Sim::step_upgrades`], in the economy phase right after the sites): one a tick, from the tick
//!   the command applies, as a recipe's (`docs/architecture/economy.md > Economy > Recipe timing`): the Tunnel exists already, so
//!   an upgrade of 480 frames ordered in tick `t` completes at the end of `t + 479` ("A Tunnel upgrade takes 30 s",
//!   design change 0229). A Tunnel that came into the game in the tick starts on it in the next (the spawn-tick rule).
//! - **Completion** ([`Sim::complete_upgrade`]): the Tunnel is at the target tier (`Sim::set_tier`): "**HP on
//!   completion** ...: max HP rises (600 → 800 → 1000) and current HP rises by the same amount", and its area, Tunnel
//!   Space and transit are the tier's. Then it takes back the neutral expansions its new area covers ("**Reclaiming
//!   happens** the moment the covering Tunnel, or its upgrade, **finishes**", `Sim::reclaim`, slice M4-H). A Laser
//!   Turret's conversion follows from the areas, with no call (slice S2).
//! - **Cancel** ([`Sim::cancel_upgrade`], the Tunnel's X: "Cancel Upgrade — cancels the in-progress Tunnel upgrade
//!   ... Supplies refunded at the current top price"): the operation slot is free again, and the owner gets the
//!   price back that the upgrade would cost now, not counting itself (design changes 0232 and 0284). "The Supplies paid
//!   for what the player still has always equal the formula's total", whatever the order of starts and cancels.
//! - **Destroyed mid-upgrade** (`Sim::remove_objects`): "the Supplies paid for the upgrade are lost". The operation
//!   goes with the Tunnel, and the removal path leaves its expansions neutral by its current area (`sim/tunnels.rs`).

use super::Sim;
use super::gate::{Checked, Reject};
use crate::defs::{ObjectKind, UpgradeDef};
use crate::economy::RATIO_ONE;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::nav::TileRect;
use crate::network;
use crate::object::Object;
use crate::progress::Progress;
use crate::tunnel::{self, Operation, area};

/// The upgrade of a `kind` to tier `to` (`TierDef::upgrade`): `None` for a kind without a tier table, a tier its
/// table hasn't, and the base tier, which no upgrade reaches.
fn upgrade_def(kind: ObjectKind, to: u8) -> Option<UpgradeDef> {
    tunnel::def_of(kind)?.tier(to)?.upgrade
}

/// `def`'s price with `others` of the owner's other Tunnels counted (`factions/syndicate/tunnels.md > Cost Scaling >
/// Upgrade Costs`): its base, and as much again per Tunnel for each.
fn price_at(def: UpgradeDef, others: usize) -> u32 {
    let others = u32::try_from(others).unwrap_or(u32::MAX);
    def.base.saturating_add(def.per_tunnel.saturating_mul(others))
}

impl Sim {
    /// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState`: whether `player`'s Tunnel `tunnel` may
    /// upgrade now, the gate's answer to an Upgrade (module docs), by which the panel greys its Q and says why. The
    /// Tunnel is the player's ([`Reject::NotOwned`]) and in operation ([`Reject::NotATunnel`]), as for every Tunnel
    /// command.
    pub fn can_upgrade(&self, player: PlayerId, tunnel: ObjectId) -> Result<(), Reject> {
        let target = self.referenced(tunnel, Some(player))?;
        if !network::is_exit(target) {
            return Err(Reject::NotATunnel(tunnel));
        }
        self.upgrade_available(player, target).map(|_| ())
    }

    /// The price of Tunnel `tunnel`'s upgrade now, in Supplies (module docs): of the one under way while it upgrades,
    /// what its X would refund, else of the one to the tier above, what its Q costs. `None` for an object that isn't a
    /// Tunnel of a player's, or one at its top tier.
    pub fn upgrade_price(&self, tunnel: ObjectId) -> Option<u32> {
        let o = self.object(tunnel)?;
        let (owner, state) = (o.owner?, o.tunnel.as_ref()?);
        let to = match state.operation.and_then(Operation::upgrade_to) {
            Some(to) => to,
            None => state.tier.checked_add(1)?,
        };
        self.price_to(owner, tunnel, o.kind, to)
    }

    /// The one price function (module docs): `player`'s upgrade of Tunnel `tunnel`, a `kind`, to tier `to`, if
    /// ordered now: the tier's rule, with every other Tunnel of `player`'s at `to` or above, or upgrading to it,
    /// counted (`TunnelState::reserved_tier`), never `tunnel` itself. `None` if no upgrade reaches `to`.
    ///
    /// [`TunnelState::reserved_tier`]: crate::tunnel::TunnelState::reserved_tier
    fn price_to(&self, player: PlayerId, tunnel: ObjectId, kind: ObjectKind, to: u8) -> Option<u32> {
        let def = upgrade_def(kind, to)?;
        let others = (self.objects.values())
            .filter(|o| o.owner == Some(player) && o.id != tunnel)
            .filter(|o| o.tunnel.as_ref().is_some_and(|state| state.reserved_tier() >= to))
            .count();
        Some(price_at(def, others))
    }

    /// The gate for `TunnelCommand::Upgrade` on `tunnel`, the player's Tunnel in operation (`Sim::check_tunnel`): it may
    /// upgrade now ([`Sim::can_upgrade`]), to the tier above its own, for the price.
    pub(super) fn check_upgrade(&self, player: PlayerId, tunnel: &Object) -> Result<Checked, Reject> {
        let (to, price) = self.upgrade_available(player, tunnel)?;
        Ok(Checked::Upgrade {
            tunnel: tunnel.id,
            to,
            price,
        })
    }

    /// [`Sim::can_upgrade`] once the Tunnel is known to be `player`'s in operation (module docs): a tier above, no
    /// operation under way, the Supplies, then the truth. The tier it would upgrade to, and the price.
    fn upgrade_available(&self, player: PlayerId, tunnel: &Object) -> Result<(u8, u32), Reject> {
        let state = tunnel.tunnel.as_ref().ok_or(Reject::NotATunnel(tunnel.id))?;
        let to = (state.tier.checked_add(1))
            .filter(|&to| upgrade_def(tunnel.kind, to).is_some())
            .ok_or(Reject::TopTier(tunnel.id))?;
        if state.operation.is_some() {
            return Err(Reject::OperationUnderWay(tunnel.id));
        }
        let price = (self.price_to(player, tunnel.id, tunnel.kind, to)).ok_or(Reject::TopTier(tunnel.id))?;
        if self.player(player).map_or(0, |p| p.supplies) < price {
            return Err(Reject::InsufficientSupplies);
        }
        if self.upgrade_blocked(tunnel, to) {
            return Err(Reject::UpgradeBlocked(tunnel.id));
        }
        Ok((to, price))
    }

    /// Whether something blocks Tunnel `tunnel`'s upgrade to tier `to`, by the truth (module docs): its Tunnel Area at
    /// `to` reaches a cell off the map or not Drillable (`tunnel::area::breaches`), or overlaps the area another
    /// Tunnel holds (`tunnel::area::reserved`: a site's, another upgrade's target area). True for an object that isn't
    /// a placed Tunnel with that tier.
    ///
    /// **The owner's lingering Tunnel ghosts don't block it** (`Sim::pending_builds`): this reads `factions/syndicate/
    /// tunnels.md > Tunnel Area Non-Overlap Rule` literally. A pending ghost "reserves its whole Tier 1 area against its
    /// **owner's** further Tunnel placements", and "**Upgrades** are blocked by any Tunnel's area": a ghost is no
    /// Tunnel, and an upgrade no placement. So an upgrade may grow over its owner's ghost, whose build then fails on
    /// arrival, by the truth (`Sim::arrival_check`). No other player's ghost could block it either: it is that player's
    /// alone.
    fn upgrade_blocked(&self, tunnel: &Object, to: u8) -> bool {
        let Some(target) = self.area_at(tunnel, to) else {
            return true;
        };
        if area::breaches(&self.map, target).next().is_some() {
            return true;
        }
        (self.objects.values())
            .filter(|o| o.id != tunnel.id)
            .filter_map(area::reserved)
            .any(|other| area::overlaps(target, other))
    }

    /// The Tunnel Area of `tunnel` at tier `to`: `None` for an object that isn't a placed Tunnel, or a tier its table
    /// hasn't.
    fn area_at(&self, tunnel: &Object, to: u8) -> Option<TileRect> {
        let placement = tunnel.placement?;
        let radius = tunnel::def_of(tunnel.kind)?.tier(to)?.area_radius;
        let footprint = TileRect::new(placement.origin, placement.w, placement.h);
        Some(area::area(footprint, radius))
    }

    /// Tunnel `tunnel` starts upgrading to tier `to` for `player`, as the gate resolved it (module docs): the bank pays
    /// `price`, and the upgrade is the Tunnel's operation, none of its frames done; a `SimEvent::UpgradeStarted`.
    pub(super) fn start_upgrade(&mut self, player: PlayerId, tunnel: ObjectId, to: u8, price: u32) {
        let Some(def) = (self.objects.get(&tunnel)).and_then(|o| upgrade_def(o.kind, to)) else {
            return;
        };
        // The gate made sure that the bank pays.
        let bank = self.players.iter_mut().find(|p| p.id == player);
        if !bank.is_some_and(|bank| bank.spend_supplies(price)) {
            return;
        }
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut()) {
            let progress = Progress::new(def.frames);
            state.operation = Some(Operation::Upgrading { progress, to });
        }
        self.events.push(SimEvent::UpgradeStarted { tunnel, to, price });
    }

    /// The upgrades' step of the economy phase, right after the sites (module docs): each Tunnel from before this tick
    /// (`first_new`) that upgrades, in id order, does a frame at full speed; one whose frames are done completes
    /// ([`Sim::complete_upgrade`]).
    pub(super) fn step_upgrades(&mut self, first_new: ObjectId) {
        let mut completed: Vec<(ObjectId, u8)> = Vec::new();
        for o in self.objects.range_mut(..first_new).map(|(_, o)| o) {
            let Some(state) = o.tunnel.as_mut() else {
                continue;
            };
            if let Some(Operation::Upgrading { progress, to }) = &mut state.operation
                && progress.step(RATIO_ONE)
            {
                completed.push((o.id, *to));
            }
        }
        for (tunnel, to) in completed {
            self.complete_upgrade(tunnel, to);
        }
    }

    /// Tunnel `tunnel`'s upgrade to tier `to` is complete (module docs): it is at that tier, the HP rising with the
    /// MaxHP, its operation over (`Sim::set_tier`); a `SimEvent::UpgradeCompleted`. Then it takes back the neutral
    /// expansions it now covers (`Sim::reclaim`).
    fn complete_upgrade(&mut self, tunnel: ObjectId, to: u8) {
        self.set_tier(tunnel, to);
        self.events.push(SimEvent::UpgradeCompleted { tunnel, tier: to });
        self.reclaim(tunnel);
    }

    /// Cancel Upgrade on `tunnel` (its X, `Sim::cancel_operation`; module docs): the upgrade ends, and the Tunnel's owner
    /// gets back the price it would cost now, not counting itself ([`Sim::price_to`]); a `SimEvent::UpgradeCancelled`.
    /// Nothing for a Tunnel that doesn't upgrade.
    pub(super) fn cancel_upgrade(&mut self, tunnel: ObjectId) {
        let Some((owner, kind, to)) = (self.objects.get(&tunnel))
            .and_then(|o| Some((o.owner?, o.kind, o.tunnel.as_ref()?.operation?.upgrade_to()?)))
        else {
            return;
        };
        let refund = self.price_to(owner, tunnel, kind, to).unwrap_or(0);
        if let Some(state) = self.objects.get_mut(&tunnel).and_then(|o| o.tunnel.as_mut()) {
            state.operation = None;
        }
        if let Some(bank) = self.players.iter_mut().find(|p| p.id == owner) {
            bank.credit_supplies(refund);
        }
        self.events.push(SimEvent::UpgradeCancelled { tunnel, to, refund });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Command, DebugCommand, PlayerCommand, TickInputs, TunnelCommand};
    use crate::player::Faction;
    use crate::scenario::SpawnAt;
    use crate::sim::SimConfig;
    use crate::testing::ScenarioBuilder;
    use crate::testing::invariants::Invariants;
    use crate::units::TilePos;

    /// Design change 0231's verification: "A Tunnel at 500/600 HP upgraded to Tier 2 ends at 700/800", through the
    /// gate and the 480 frames, its HP set as no command can: 500/600 through the frames, 700/800 at the end of the
    /// 480th.
    #[test]
    fn a_tunnel_at_500_of_600_ends_its_upgrade_at_700_of_800() {
        let (player, tunnel) = (PlayerId(1), ObjectId(1));
        let mut scenario = ScenarioBuilder::new(&vec![".".repeat(24); 16].join("\n"))
            .faction(player, Faction::Syndicate)
            .spawn(ObjectKind::Tunnel, Some(player), SpawnAt::Tile(TilePos::new(10, 6)))
            .build();
        scenario.setup.config = SimConfig::DEBUG;
        let mut sim = Sim::new(&scenario);
        let mut invariants = Invariants::new(&sim);
        let mut step = |sim: &mut Sim, command: Option<Command>| {
            let commands = command.map(|command| PlayerCommand { player, command });
            let inputs = TickInputs {
                commands: commands.into_iter().collect(),
            };
            sim.step(&inputs);
            if let Err(violation) = invariants.check(sim, &inputs) {
                panic!("{violation:?}");
            }
        };
        let supplies = DebugCommand::SetResources {
            player,
            crystals: 0,
            supplies: 2,
            alloys: 0,
            essence: 0,
            flux: 0,
        };
        step(&mut sim, Some(Command::Debug(supplies)));
        sim.objects.get_mut(&tunnel).expect("the Tunnel").hp = Some(500.0);
        let hp = |sim: &Sim| (sim.objects[&tunnel].hp, sim.objects[&tunnel].max_hp());
        let upgrade = Command::Tunnel {
            tunnel,
            command: TunnelCommand::Upgrade,
        };
        step(&mut sim, Some(upgrade));
        assert_eq!(sim.player(player).map(|p| p.supplies), Some(0), "paid when ordered");
        for _ in 1..480 {
            assert_eq!(hp(&sim), (Some(500.0), Some(600.0)), "tick {}", sim.tick());
            step(&mut sim, None);
        }
        assert_eq!(hp(&sim), (Some(700.0), Some(800.0)));
    }

    /// The price rule (`factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`): "2 + 2 x (number of T2+
    /// Tunnels owned)" and "3 + 3 x (number of T3 Tunnels owned)", saturating.
    #[test]
    fn the_price_is_the_base_and_as_much_again_per_tunnel() {
        let tier_2 = upgrade_def(ObjectKind::Tunnel, 2).expect("an upgrade reaches Tier 2");
        let tier_3 = upgrade_def(ObjectKind::Tunnel, 3).expect("an upgrade reaches Tier 3");
        let prices = |def| [0, 1, 2, 3].map(|others| price_at(def, others));
        assert_eq!(prices(tier_2), [2, 4, 6, 8]);
        assert_eq!(prices(tier_3), [3, 6, 9, 12]);
        assert_eq!(price_at(tier_2, usize::MAX), u32::MAX);
        assert_eq!(
            upgrade_def(ObjectKind::Tunnel, 1),
            None,
            "nothing upgrades to the base tier"
        );
        assert_eq!(upgrade_def(ObjectKind::Tunnel, 4), None, "no Tier 4");
        assert_eq!(upgrade_def(ObjectKind::Hideout, 2), None, "no tier table");
    }
}
