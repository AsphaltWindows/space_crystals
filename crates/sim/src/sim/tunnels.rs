//! Losing Tunnels (slice M4-H; `factions/syndicate/tunnels.md > Losing Tunnels`, design changes 0254 and 0258, which
//! answers question Q0069): what a Tunnel leaves behind when it leaves the game, the decay of the expansions it leaves
//! neutral, and their reclaiming by a Tunnel that finishes over them.
//!
//! - **The removal path** (plan decision T38: one way out of the game, `Sim::remove_objects`, whatever takes the
//!   Tunnel: combat, a debug removal, a cancelled site). It notes the Tunnels among what goes as they were
//!   ([`Sim::lost_tunnels`]), and once they are gone ([`Sim::lose_tunnels`]) handles each, in id order:
//!   - its **unfinished expansions**, the sites it builds ([`built_by`]), are destroyed, with no refund
//!     (`factions/syndicate/tunnel_menus.md > Expansions under construction`, design change 0254: "**Its Tunnel
//!     destroyed mid-build**: the unfinished expansion is destroyed, no refund");
//!   - its **completed expansions** become neutral ([`Sim::neutralise`]): "when a Tunnel is destroyed, every expansion
//!     in its Tunnel Area becomes neutral — no longer owned by the Syndicate player". They are its owner's underground
//!     structures, sites aside, that lie wholly inside its area at its current tier ([`area::of`], [`area::encloses`]):
//!     those whose parent it was (`tunnel::area::parent`; an owner's Tunnel Areas never overlap). A Tunnel that was
//!     still a site has none. "**In-progress work is destroyed**": the unit in production
//!     (`Production::in_production`, begun or finished inside) is lost. "**Queued work is refunded**": the units not
//!     yet begun, their full cost to the owner, each with a `SimEvent::ProductionCancelled` (the lost one's with a
//!     refund of 0). The rally point is cleared, and the decay starts.
//!
//!   Then the **stranded units** ([`Sim::strand`]), after all of the call's removals: "whenever a Tunnel is destroyed,
//!   any unit inside the Tunnel Network that can no longer exit through any remaining Tunnel (no remaining Tunnel's
//!   tier is sufficient for its unit base category) is **destroyed**", by the one rule of the ways out
//!   (`network::admits`: a Tunnel in operation, `Object::is_operational`, whose tier's Transit lists the unit's base:
//!   "A Tunnel that is still under construction does not count as an exit, and a Tunnel that is mid-upgrade counts at
//!   its current (pre-upgrade) tier"). Each leaves the game with a `SimEvent::Removed` (`Sim::remove_held`). A unit
//!   held at a site (`network::Place::Site`) isn't in the network, and stays.
//! - **Neutral expansions** are everyone's (`factions/syndicate/tunnels.md > Losing Tunnels`):
//!   - "no longer hidden (visible to everyone, no detection needed)": the hidden predicate's neutral arm
//!     (`vision::hidden`, plan decision T32), so whoever sees their tiles sees them and remembers them
//!     (`vision::memory`), their former owner included;
//!   - "**Attacking them**: only with an **explicit Attack** order; right-click never attacks a neutral object and
//!     auto-targeting ignores them" (`systems/controls/commands.md > Attack`): the rules of every neutral object
//!     already (the default command, `orders::resolution`; auto-targeting, `combat`);
//!   - "a neutral expansion still **blocks its cells**: nothing can be built there, and they **count as occupied**
//!     for that Tunnel's Tunnel Space" (design change 0258 point 4): any structure blocks a placement over it
//!     (`placement`), and Tunnel Space counts neutral expansions (`economy::army`).
//! - **The decay** ([`Sim::step_decay`], in the economy phase, `Object::decay`): "They **slowly lose HP** until
//!   destroyed. **Decay is a percentage of MaxHP per second**: 1% per second, placeholder (a Hideout lasts 100 s)"
//!   ([`DECAY_PERCENT_PER_SECOND`]). A `Progress` counts it out exactly in fixed point: [`DECAY_TICKS`] ticks of
//!   decay at 1% a second, a tick taking `DECAY_PERCENT_PER_SECOND` of them. Its HP follows the share of the MaxHP the
//!   decay has left, falling by as much as it does each tick, and never above it ([`decayed`]), so that an expansion
//!   at full HP lasts exactly the decay's length whatever the rounding: from full HP, a Hideout dies in its 1600th
//!   tick of decay (100 s at 16 ticks a second), the first being the tick its Tunnel fell, as the first frame of a
//!   transformation is the tick that starts it (`crate::transform`). It dies as in combat, its HP at 0, with a
//!   `SimEvent::Died` (owner `None`) before it goes, and the grids follow (`paths::sync`).
//! - **Reclaiming** ([`Sim::reclaim`]): "if a Tunnel is built whose Tunnel Area fully covers a neutral expansion, and
//!   that Tunnel's tier meets the expansion's Tier Requirement, the expansion becomes owned by that Tunnel's owner —
//!   the original Syndicate player or an enemy Syndicate player — and stops decaying"; "**Reclaiming happens** the
//!   moment the covering Tunnel, or its upgrade, **finishes**; a Tunnel under construction reclaims nothing". A site's
//!   completion calls it (`Sim::complete_site`), and an upgrade's will (slice M4-G). An expansion only partly
//!   covered, or above the tier, stays neutral and keeps decaying. A reclaimed expansion is hidden again: from the
//!   other players (`vision::hidden`), whose records of it go once they see its tiles (`vision::memory`), and from
//!   its new owner's memory at once ([`Vision::forget_own`]: a player never remembers its own objects). No event
//!   says so: none would be any player's but the new owner's business, and the owner sees it as its own.
//! - **Defeat** follows the shared rule (`economy::defeat`, `factions/syndicate/tunnels.md > Losing Tunnels`:
//!   "Losing the last Tunnel therefore defeats the Syndicate only if they own no Laser Turrets"): the expansions a
//!   lost Tunnel leaves neutral no longer count, and a defeated player's units in the network are destroyed
//!   (`core/game_rules.md > On Defeat`, design change 0258 point 5: `Sim::clear_defeated_networks`). A player who
//!   lives on with no Tunnel rebuilds for nothing (`factions/syndicate/tunnels.md > Cost Scaling`: `Sim::build_price`).
//!
//! [`Vision::forget_own`]: crate::vision::Vision::forget_own

use super::Sim;
use crate::construction::site::{Builder, expansion_def};
use crate::defs::ObjectKind;
use crate::economy::{RATIO_ONE, Refund};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::nav::TileRect;
use crate::network;
use crate::object::Object;
use crate::paths;
use crate::progress::Progress;
use crate::tuning::tunnel::DECAY_PERCENT_PER_SECOND;
use crate::tunnel::{BASE_TIER, area};
use crate::units::TICKS_PER_SECOND;
use crate::vision::Hiding;

/// The length of the decay at 1% of the MaxHP a second, in ticks: the whole MaxHP in 100 seconds. A tick of decay
/// counts [`DECAY_PERCENT_PER_SECOND`] of them (module docs).
pub(crate) const DECAY_TICKS: u32 = 100 * TICKS_PER_SECOND;

/// A Tunnel the removal path takes ([`Sim::lost_tunnels`]), as it was before it went.
#[derive(Copy, Clone, Debug)]
pub(super) struct LostTunnel {
    id: ObjectId,
    owner: Option<PlayerId>,
    /// Its Tunnel Area at its current tier, the area its expansions lie in; `None` for a site, which has none. (Not
    /// whether it was an exit: a Tunnel mid-upgrade is one, and one out of operation for any reason would still have
    /// its expansions.)
    area: Option<TileRect>,
}

/// Whether `o` is an expansion (`factions/syndicate/structures/README.md > Tunnel Expansions`): an underground
/// structure (`StructureDef::underground`, plan decision T32), the Hideout so far.
fn is_expansion(o: &Object) -> bool {
    o.def().structure.as_ref().is_some_and(|s| s.underground)
}

/// The footprint of `o`, if it has one.
fn footprint(o: &Object) -> Option<TileRect> {
    o.placement.map(|p| TileRect::new(p.origin, p.w, p.h))
}

/// Design change 0254: whether a site whose builder is `builder` is one of Tunnel `tunnel`'s unfinished expansions,
/// which are destroyed with it (module docs). An Agent's site (`Builder::Held`, a Tunnel) never is; an expansion's
/// site (`Builder::Tunnel`, slice M4-F: `sim/expansions.rs`) is, built by `tunnel`.
fn built_by(builder: Builder, tunnel: ObjectId) -> bool {
    match builder {
        Builder::Held(_) | Builder::Recruits => false,
        Builder::Tunnel(by) => by == tunnel,
    }
}

/// `factions/syndicate/structures/README.md > Expansion Requirements`: the Tier Requirement of an expansion of `kind`,
/// the lowest tier of a Tunnel that may reclaim it (module docs), its def's as a Tunnel builds it
/// (`ExpansionDef::tier`, slice M4-F: the Hideout's 1, `factions/syndicate/structures/hideout.md > Tier Requirement -
/// 1`); the base tier for a kind no Tunnel builds.
fn tier_requirement(kind: ObjectKind) -> u8 {
    expansion_def(kind).map_or(BASE_TIER, |def| def.tier)
}

/// The HP of an expansion at `hp` out of `max`, after a tick of decay took its progress from `before` to `after`
/// (module docs): it falls by as much as the share of the MaxHP the decay has left, and never above that share. The
/// shares from the whole ticks left, in 64 bits: exact for a whole MaxHP, so that an expansion at full HP loses
/// exactly its share each tick.
fn decayed(hp: f32, max: f32, before: &Progress, after: &Progress) -> f32 {
    let left = |p: &Progress| match p.total {
        0 => 0.0,
        total => f64::from(max) * f64::from(total - p.ticks_done()) / f64::from(total),
    };
    let after_left = left(after);
    ((f64::from(hp) - (left(before) - after_left)).min(after_left)) as f32
}

impl Sim {
    /// The Tunnels among `gone`, which the removal path is about to take (module docs), as they are now, in id order.
    pub(super) fn lost_tunnels(&self, gone: &[ObjectId]) -> Vec<LostTunnel> {
        (gone.iter())
            .filter_map(|id| self.objects.get(id))
            .filter(|o| o.tunnel.is_some())
            .map(|o| LostTunnel {
                id: o.id,
                owner: o.owner,
                area: area::of(o).filter(|_| !o.is_site()),
            })
            .collect()
    }

    /// The removal path's rules for the Tunnels `lost` that just left the game (module docs): each one's unfinished
    /// expansions destroyed and its completed ones neutral, in id order; then the units its owner's network can no
    /// longer let out.
    pub(super) fn lose_tunnels(&mut self, lost: &[LostTunnel]) {
        if lost.is_empty() {
            return;
        }
        let mut unfinished: Vec<ObjectId> = Vec::new();
        for tunnel in lost {
            unfinished.extend(self.unfinished_expansions(tunnel.id));
            if let (Some(owner), Some(area)) = (tunnel.owner, tunnel.area) {
                for id in self.expansions_in(owner, area) {
                    self.neutralise(id);
                }
            }
        }
        // Design change 0254: "destroyed, no refund". Their builder, the Tunnel, is gone already.
        if !unfinished.is_empty() {
            self.remove_objects(&unfinished);
        }
        let mut owners: Vec<PlayerId> = lost.iter().filter_map(|tunnel| tunnel.owner).collect();
        owners.sort_unstable();
        owners.dedup();
        self.strand(&owners);
    }

    /// Tunnel `tunnel`'s unfinished expansions: the sites it builds ([`built_by`]), in id order.
    fn unfinished_expansions(&self, tunnel: ObjectId) -> Vec<ObjectId> {
        (self.objects.values())
            .filter(|o| o.construction.is_some_and(|site| built_by(site.builder, tunnel)))
            .map(|o| o.id)
            .collect()
    }

    /// `owner`'s completed expansions wholly inside `area`, in id order.
    fn expansions_in(&self, owner: PlayerId, area: TileRect) -> Vec<ObjectId> {
        (self.objects.values())
            .filter(|o| o.owner == Some(owner) && is_expansion(o) && !o.is_site())
            .filter(|o| footprint(o).is_some_and(|f| area::encloses(area, f)))
            .map(|o| o.id)
            .collect()
    }

    /// Expansion `id` becomes neutral (module docs): its work in progress lost, its queue refunded, its rally point
    /// cleared; it starts to decay.
    fn neutralise(&mut self, id: ObjectId) {
        self.abandon_work(id);
        if let Some(o) = self.objects.get_mut(&id) {
            o.owner = None;
            o.decay = Some(Progress::new(DECAY_TICKS));
        }
    }

    /// `factions/syndicate/tunnels.md > Losing Tunnels`: "**In-progress work is destroyed**: any production or research
    /// in progress in the expansion when its Tunnel is destroyed is lost"; "**Queued work is refunded**: units or
    /// research queued in the expansion but not yet started are removed and their full cost is refunded to the
    /// player". Producer `id`'s whole queue goes, each unit with a `SimEvent::ProductionCancelled`: the unit in
    /// production (`Production::in_production`: begun, or finished and waiting inside) refunds nothing, the others
    /// their full cost to its owner. Its rally point goes too: a neutral producer has none.
    fn abandon_work(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get_mut(&id) else {
            return;
        };
        let owner = o.owner;
        let def = o.def().structure.as_ref().and_then(|s| s.production.as_ref());
        let (Some(production), Some(def)) = (o.production.as_mut(), def) else {
            return;
        };
        let begun = production.in_production().is_some();
        production.rally = None;
        for (i, (entry, reserved)) in production.take_queue().into_iter().enumerate() {
            let refund = match i == 0 && begun {
                true => 0,
                false => Refund::Full.of(entry.cost(def)),
            };
            if let Some(player) = self.players.iter_mut().find(|p| Some(p.id) == owner) {
                player.credit_crystals(refund);
            }
            self.events
                .extend(super::producers::cancelled(id, entry, &reserved, refund));
        }
    }

    /// `factions/syndicate/tunnels.md > Losing Tunnels > Stranded units` (module docs): the units in the networks of
    /// `owners` that no Tunnel of their owner can let out any more leave the game.
    fn strand(&mut self, owners: &[PlayerId]) {
        let mut stranded: Vec<ObjectId> = Vec::new();
        for &owner in owners {
            let exits: Vec<&Object> = (self.objects.values())
                .filter(|o| o.owner == Some(owner) && network::is_exit(o))
                .collect();
            let lets_out = |kind: ObjectKind| exits.iter().any(|t| network::admits(t, kind).is_ok());
            stranded.extend(
                self.network(owner)
                    .filter(|held| !lets_out(held.object.kind))
                    .map(|held| held.object.id),
            );
        }
        self.remove_held(&stranded);
    }

    /// The decay's step (module docs), in the economy phase: each neutral expansion that decays loses a tick of it, in
    /// id order; one whose HP reaches 0 dies, with a `SimEvent::Died`, and leaves the game through the removal path,
    /// the grids following.
    pub(super) fn step_decay(&mut self) {
        let mut dead: Vec<ObjectId> = Vec::new();
        for o in self.objects.values_mut() {
            if o.decay.is_none() {
                continue;
            }
            let (Some(max), Some(hp)) = (o.max_hp(), o.hp) else {
                continue;
            };
            let Some(decay) = o.decay.as_mut() else {
                continue;
            };
            let before = *decay;
            decay.step(DECAY_PERCENT_PER_SECOND.saturating_mul(RATIO_ONE));
            let hp = decayed(hp, max, &before, decay);
            if hp > 0.0 {
                o.hp = Some(hp);
                continue;
            }
            o.hp = Some(0.0);
            dead.push(o.id);
            self.events.push(SimEvent::Died {
                id: o.id,
                kind: o.kind,
                owner: o.owner,
                pos: o.pos,
                hiding: Hiding::of(o),
            });
        }
        if dead.is_empty() {
            return;
        }
        self.remove_objects(&dead);
        paths::sync(&mut self.nav, &self.objects, &mut self.nav_version);
    }

    /// Reclaiming (module docs): Tunnel `tunnel`, which has just finished (its site, or from slice M4-G an upgrade),
    /// takes every neutral expansion wholly inside its Tunnel Area whose Tier Requirement its tier meets: each becomes
    /// its owner's and stops decaying, hidden again, in id order. Nothing for an object that isn't a Tunnel in
    /// operation: a Tunnel under construction reclaims nothing.
    pub(super) fn reclaim(&mut self, tunnel: ObjectId) {
        let Some(t) = self.objects.get(&tunnel).filter(|t| network::is_exit(t)) else {
            return;
        };
        let (Some(owner), Some(area), Some(tier)) = (t.owner, area::of(t), t.tunnel.as_ref().map(|state| state.tier))
        else {
            return;
        };
        let taken: Vec<ObjectId> = (self.objects.values())
            .filter(|o| o.owner.is_none() && is_expansion(o) && !o.is_site())
            .filter(|o| footprint(o).is_some_and(|f| area::encloses(area, f)))
            .filter(|o| tier_requirement(o.kind) <= tier)
            .map(|o| o.id)
            .collect();
        for id in taken {
            if let Some(o) = self.objects.get_mut(&id) {
                o.owner = Some(owner);
                o.decay = None;
            }
            self.vision.forget_own(owner, id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::test_defs::TestDef;

    /// The Tier Requirement a reclaiming Tunnel must meet is the expansion's def's (slice M4-F): the Hideout's 1, a
    /// test kind's 2, and the base tier for a kind no Tunnel builds.
    #[test]
    fn the_tier_requirement_is_the_expansions_def() {
        assert_eq!(tier_requirement(ObjectKind::Hideout), 1);
        assert_eq!(tier_requirement(ObjectKind::Tunnel), BASE_TIER);
        let mut def = *expansion_def(ObjectKind::Hideout).expect("a Tunnel builds the Hideout");
        def.tier = 2;
        let _tier_2 = TestDef::structure(ObjectKind::Hideout, |structure| structure.expansion = Some(def));
        assert_eq!(tier_requirement(ObjectKind::Hideout), 2);
    }

    /// The decay's arithmetic (module docs): from full HP the HP is the share left, 4 HP a second off a Hideout's 400
    /// (a quarter a tick), down to 0 in the 1600th tick and not before; a damaged one falls by as much each tick.
    #[test]
    fn full_hp_lasts_the_whole_decay_and_damage_falls_by_as_much() {
        let max = 400.0;
        let mut decay = Progress::new(DECAY_TICKS);
        let (mut full, mut hurt) = (max, 250.0);
        for tick in 1..=DECAY_TICKS {
            let before = decay;
            decay.step(DECAY_PERCENT_PER_SECOND * RATIO_ONE);
            full = decayed(full, max, &before, &decay);
            hurt = decayed(hurt, max, &before, &decay);
            if tick == 16 {
                assert_eq!((full, hurt), (396.0, 246.0), "4 HP a second");
            }
            if tick < DECAY_TICKS {
                assert!(full > 0.0, "alive after tick {tick}");
            }
            if tick == 600 {
                assert_eq!(hurt, 100.0);
            }
        }
        assert!(decay.done());
        assert!(full <= 0.0, "gone in the 1600th tick: {full}");
        assert!(hurt <= 0.0);
    }
}
