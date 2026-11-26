//! On-site construction (plan decision T31, slice M4-E): a structure that a worker builds where it stands, from its
//! first frame the object it will be — the Tunnel an Agent builds (`factions/syndicate/units/agent.md > Building >
//! Tunnel Construction Flow`: "the partially-built Tunnel appears at the location with the Agent inside it"). A Tunnel
//! builds its expansions so too (slice M4-F, `sim/expansions.rs`): the Hideout it places in its Tunnel Area.
//!
//! - **The state** ([`Construction`], `Object::construction`): its progress, the shared [`Progress`] over its def's
//!   frames (`SiteDef::frames`, a Tunnel's expansion `ExpansionDef::frames`), and its [`Builder`]: the unit held
//!   inside it (`network::Place::Site`), the Tunnel whose operation it is (`tunnel::Operation::Building`), or the
//!   group that builds it from outside, of which nothing is stored (`Builder::Recruits`, plan decision T59, slice
//!   M5-E: the Cults' buildings, `factions/cults/economy.md > Cults Building Mechanics`; a frame for each builder
//!   within reach in each tick, `sim/sites.rs`). Hashed only while set, and left out of files while `None`, so that
//!   the states of games without a site are as they were.
//! - **HP** (`core/entities.md > ConstructionHP Rule`, design changes 0234 and 0254): the rule gives the **maximum**,
//!   [`Construction::max_hp`]: "MaxHP x (10% + 90% x construction_progress)", the share it starts at being its
//!   builder's def's (`SiteDef::start_hp`, `ExpansionDef::start_hp`: [`Construction::max_hp_of`]). The current HP
//!   follows the maximum as it rises, so the damage taken stays (`transform::follow_max`, the Warp In's rule).
//! - **Seen** by the other players as the structure it will be (an owned underground expansion not at all,
//!   `vision::hidden`; `factions/syndicate/units/agent.md > While the Tunnel is under construction`: "What enemies see:
//!   the site, like any surface structure, but not the Agent inside it"), and remembered in the fog as last seen
//!   (`core/vision.md > Explored`): a site at the progress it had then, without who builds it
//!   (`vision::LastKnown::site`, [`max_hp_seen`]).
//! - **Operational** ([`Object::is_operational`]): a site does nothing but cancel (design change 0234: "no Tunnel
//!   Space, no drop-offs, no Enter, no Expand, no Eject"), and neither does a building still transforming. One
//!   predicate, for every function that waits for completion: the exits and the Enter target (`network::is_exit`),
//!   the drop-offs (`orders::world`, `Sim::nearest_drop_off`, the gate) and Tunnel Space (`economy::army`).
//! - **The order, the price and the steps** are the game's: `sim/builds.rs` and `sim/sites.rs`. A price is in its
//!   def's resource ([`BuildPrice`]): Supplies for a Tunnel, Space Crystals for a Cults building (slice M5-E).

use serde::{Deserialize, Serialize};

use crate::defs::{ExpansionDef, ObjectKind, Resource, SiteDef, SitePrice};
use crate::economy::Refund;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::player::Player;
use crate::progress::Progress;

/// What a worker's Build of a type costs now (`SiteDef::price`, `Sim::build_price`): an amount of one resource,
/// Supplies for a Tunnel (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`), Space Crystals for a
/// Cults building (`factions/cults/economy.md > Build Command`: "the building's Space Crystals cost is deducted as soon
/// as the build order is given"). Paid as the order is given, and given back in the same resource.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BuildPrice {
    pub resource: Resource,
    pub amount: u32,
}

impl BuildPrice {
    /// `amount` Supplies.
    pub const fn supplies(amount: u32) -> Self {
        Self {
            resource: Resource::Supplies,
            amount,
        }
    }

    /// `amount` Space Crystals.
    pub const fn crystals(amount: u32) -> Self {
        Self {
            resource: Resource::Crystals,
            amount,
        }
    }

    /// The price by the rule `price`, `owned` being how many structures of the type the player has for it
    /// (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`: "Cost = current Tunnels owned").
    pub fn of(price: SitePrice, owned: u32) -> Self {
        match price {
            SitePrice::SuppliesPerOwned => Self::supplies(owned),
            SitePrice::Crystals(amount) => Self::crystals(amount),
        }
    }

    /// What `player`'s bank holds of its resource.
    pub fn banked(self, player: &Player) -> u32 {
        match self.resource {
            Resource::Crystals => player.crystals,
            Resource::Supplies => player.supplies,
        }
    }

    /// `player` pays it, if the bank holds it, and says whether it did (`Player::spend_crystals`: a spend is refused,
    /// never taken below 0).
    #[must_use]
    pub fn pay(self, player: &mut Player) -> bool {
        match self.resource {
            Resource::Crystals => player.spend_crystals(self.amount),
            Resource::Supplies => player.spend_supplies(self.amount),
        }
    }

    /// `player` gets it back (`Player::credit_crystals`).
    pub fn refund(self, player: &mut Player) {
        match self.resource {
            Resource::Crystals => player.credit_crystals(self.amount),
            Resource::Supplies => player.credit_supplies(self.amount),
        }
    }

    /// What cancelling a site gives back, `self` being the price of the next one once the site has gone
    /// (`Sim::build_price`) and `rule` its type's (`SiteDef::price`): a Tunnel's all of it, "the current top price"
    /// (design changes 0233 and 0284); a Cults building's 75%, rounded down (`factions/cults/economy.md > Cancel
    /// Construction`: "refunds 75% of the building's Space Crystals cost (rounded down)"; `Refund::Expansion`, an
    /// expansion's share).
    pub fn cancelled(self, rule: SitePrice) -> Self {
        let amount = match rule {
            SitePrice::SuppliesPerOwned => self.amount,
            SitePrice::Crystals(_) => Refund::Expansion.of(self.amount),
        };
        Self { amount, ..self }
    }
}

/// A structure under construction on its site (module docs): how far it is, and who builds it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Construction {
    /// Its frames done, out of its def's [`SiteDef::frames`]: a frame a tick from the tick after the site appears, a new
    /// object (`ARCHITECTURE.md > Tick pipeline > Spawn tick`), so that a site started in tick `t` completes at the end
    /// of tick `t + frames`.
    pub progress: Progress,
    pub builder: Builder,
}

/// Who builds a site (plan decision T31).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Builder {
    /// A unit held inside the site, off the map (`network::Place::Site`, plan decision T30): the Agent of a Tunnel
    /// (`factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`: "The Agent is untargetable for the
    /// duration of construction").
    Held(ObjectId),
    /// The Tunnel building the site, an expansion in its Tunnel Area, as its one operation (slice M4-F,
    /// `tunnel::Operation::Building`; `factions/syndicate/tunnel_menus.md > Expansions under construction`). It stays
    /// on the map, and its Eject still works ("it isn't an operation").
    Tunnel(ObjectId),
    /// The workers that build it from outside, a group sharing its build (plan decision T59, slice M5-E;
    /// `factions/cults/economy.md > Cults Building Mechanics`: "Cults buildings are constructed on-site by Recruits
    /// working from outside the structure"; `BuilderGoes::Outside`). Nothing of them is stored: they are the units on
    /// an Assist of it within reach of its footprint's edge, their walk over, derived each tick (`crate::spot_work`),
    /// and none of them leaves the map.
    Recruits,
}

impl Builder {
    /// The unit that builds, held inside the site; `None` for a Tunnel's expansion and a site built from outside.
    pub fn unit(self) -> Option<ObjectId> {
        match self {
            Builder::Held(unit) => Some(unit),
            Builder::Tunnel(_) | Builder::Recruits => None,
        }
    }

    /// The Tunnel that builds, for its expansion; `None` for a worker's site.
    pub fn tunnel(self) -> Option<ObjectId> {
        match self {
            Builder::Held(_) | Builder::Recruits => None,
            Builder::Tunnel(tunnel) => Some(tunnel),
        }
    }
}

impl Construction {
    /// A site of `def`'s, built by `builder`, with no frame done.
    pub fn new(def: &SiteDef, builder: Builder) -> Self {
        Self {
            progress: Progress::new(def.frames),
            builder,
        }
    }

    /// An expansion of `def`'s that Tunnel `tunnel` builds (slice M4-F), with no frame done.
    pub fn expansion(def: &ExpansionDef, tunnel: ObjectId) -> Self {
        Self {
            progress: Progress::new(def.frames),
            builder: Builder::Tunnel(tunnel),
        }
    }

    /// `core/entities.md > ConstructionHP Rule`: the maximum HP of a site of `def`'s whose structure has `full` MaxHP
    /// once built: "MaxHP x (10% + 90% x construction_progress)", the progress in whole frames done
    /// ([`Progress::ratio`]).
    pub fn max_hp(&self, def: &SiteDef, full: f32) -> f32 {
        self.max_hp_from(def.start_hp, full)
    }

    /// The same for a site of `kind`'s, by its builder's def: a worker's [`SiteDef`], a Tunnel's [`ExpansionDef`]
    /// (design change 0254: "the ConstructionHP Rule applies to every expansion (starts at 10%, grows with
    /// progress)"). `None` for a kind its builder doesn't build.
    pub fn max_hp_of(&self, kind: ObjectKind, full: f32) -> Option<f32> {
        let start_hp = match self.builder {
            Builder::Held(_) | Builder::Recruits => site_def(kind)?.start_hp,
            Builder::Tunnel(_) => expansion_def(kind)?.start_hp,
        };
        Some(self.max_hp_from(start_hp, full))
    }

    /// The rule, from the share `start_hp` of `full`.
    fn max_hp_from(&self, start_hp: f32, full: f32) -> f32 {
        rule_max_hp(start_hp, &self.progress, full)
    }
}

/// The ConstructionHP Rule's maximum for a site of `kind`'s seen at `progress`, whose structure has `full` MaxHP once
/// built: what a player who saw it remembers (`vision::LastKnown::max_hp`), not knowing who builds it. Its kind tells
/// the share it starts at: a worker's site's ([`site_def`]) or a Tunnel's expansion's ([`expansion_def`]), no kind
/// being built both ways (a test holds every kind to it). `None` for a kind no one builds on a site.
pub fn max_hp_seen(kind: ObjectKind, progress: &Progress, full: f32) -> Option<f32> {
    let start_hp = match (site_def(kind), expansion_def(kind)) {
        (Some(def), _) => def.start_hp,
        (None, Some(def)) => def.start_hp,
        (None, None) => return None,
    };
    Some(rule_max_hp(start_hp, progress, full))
}

/// "MaxHP x (10% + 90% x construction_progress)", from the share `start_hp` of `full` at `progress`.
fn rule_max_hp(start_hp: f32, progress: &Progress, full: f32) -> f32 {
    full * (start_hp + (1.0 - start_hp) * progress.ratio())
}

/// How a worker builds a structure of `kind` (`StructureDef::site`): `None` for a kind no worker builds.
pub fn site_def(kind: ObjectKind) -> Option<&'static SiteDef> {
    kind.def().structure.as_ref()?.site.as_ref()
}

/// How a Tunnel builds an expansion of `kind` (`StructureDef::expansion`, slice M4-F): its Tier Requirement, cost and
/// build time. `None` for a kind no Tunnel builds.
pub fn expansion_def(kind: ObjectKind) -> Option<&'static ExpansionDef> {
    kind.def().structure.as_ref()?.expansion.as_ref()
}

/// Whether a unit of `builder` builds a structure of `kind` on its site: its worker lists it (`WorkerDef::builds`),
/// and it is a kind that a worker builds ([`site_def`]). The Agent builds the Tunnel.
pub fn builds(builder: ObjectKind, kind: ObjectKind) -> bool {
    let worker = builder.def().unit.as_ref().and_then(|unit| unit.worker);
    worker.is_some_and(|worker| worker.builds.contains(&kind)) && site_def(kind).is_some()
}

/// Whether a unit of `builder` builds anything on a site.
pub fn builds_any(builder: ObjectKind) -> bool {
    let worker = builder.def().unit.as_ref().and_then(|unit| unit.worker);
    worker.is_some_and(|worker| worker.builds.iter().any(|&kind| site_def(kind).is_some()))
}

/// Whether workers of `kind` share a build (`WorkerDef::shares_builds`, plan decision T59): the Recruit.
pub fn shares_builds(kind: ObjectKind) -> bool {
    let worker = kind.def().unit.as_ref().and_then(|unit| unit.worker);
    worker.is_some_and(|worker| worker.shares_builds)
}

/// Whether a unit of `builder` assists the construction of a site of `kind` (`OrderKind::Assist`;
/// `factions/cults/economy.md > Assist Construction Command`): it builds the kind ([`builds`]) with the others, sharing
/// the build ([`shares_builds`]). The Recruit assists every building it constructs.
pub fn assists(builder: ObjectKind, kind: ObjectKind) -> bool {
    shares_builds(builder) && builds(builder, kind)
}

impl Object {
    /// Whether it is a site, a structure under construction (module docs).
    pub fn is_site(&self) -> bool {
        self.construction.is_some()
    }

    /// Whether it is a site its builders build from outside (`Builder::Recruits`, slice M5-E): the one target of an
    /// Assist (`OrderKind::Assist`).
    pub fn is_built_from_outside(&self) -> bool {
        self.construction.is_some_and(|site| site.builder == Builder::Recruits)
    }

    /// Whether it does what its type does (module docs): it is neither a site (design change 0234: "It does nothing
    /// but Cancel") nor transforming into its kind (`crate::transform`: a building warped in, a Beacon being deployed,
    /// whose functions wait for completion). Every function that waits for completion asks this.
    pub fn is_operational(&self) -> bool {
        !self.is_site() && !self.is_transforming()
    }
}

impl StateHash for Builder {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Builder::Held(unit) => {
                h.tag(0);
                unit.hash(h);
            }
            Builder::Tunnel(tunnel) => {
                h.tag(1);
                tunnel.hash(h);
            }
            Builder::Recruits => h.tag(2),
        }
    }
}

impl StateHash for Construction {
    fn hash(&self, h: &mut StateHasher) {
        let Construction { progress, builder } = self;
        progress.hash(h);
        builder.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::RATIO_ONE;

    /// `core/entities.md > ConstructionHP Rule`: a Tunnel site's maximum is 10% of the Tunnel's 600 as it starts, and
    /// grows linearly to the whole over its 480 frames.
    #[test]
    fn the_maximum_grows_from_a_tenth_to_the_whole() {
        let def = site_def(ObjectKind::Tunnel).unwrap();
        let mut site = Construction::new(def, Builder::Held(ObjectId(7)));
        assert_eq!(site.max_hp(def, 600.0), 60.0);
        assert!(!site.progress.step(240 * RATIO_ONE));
        assert!((site.max_hp(def, 600.0) - 330.0).abs() < 1e-3);
        assert!(site.progress.step(240 * RATIO_ONE));
        assert_eq!(site.max_hp(def, 600.0), 600.0);
    }

    /// `factions/syndicate/structures/hideout.md` ("HP - 400", "Build Time - 400 frames") and design change 0254: a
    /// Hideout a Tunnel builds has a maximum of 40 as it starts, 220 halfway and 400 once its 400 frames are done.
    /// The builder picks the def: as a worker's site the Hideout has none.
    #[test]
    fn an_expansion_grows_from_a_tenth_too() {
        let def = expansion_def(ObjectKind::Hideout).unwrap();
        let mut site = Construction::expansion(def, ObjectId(3));
        assert_eq!(site.builder, Builder::Tunnel(ObjectId(3)));
        assert_eq!(site.max_hp_of(ObjectKind::Hideout, 400.0), Some(40.0));
        assert!(!site.progress.step(200 * RATIO_ONE));
        assert!((site.max_hp_of(ObjectKind::Hideout, 400.0).unwrap() - 220.0).abs() < 1e-3);
        assert!(site.progress.step(200 * RATIO_ONE));
        assert_eq!(site.max_hp_of(ObjectKind::Hideout, 400.0), Some(400.0));
        let held = Construction {
            builder: Builder::Held(ObjectId(4)),
            ..site
        };
        assert_eq!(held.max_hp_of(ObjectKind::Hideout, 400.0), None);
        assert_eq!(expansion_def(ObjectKind::Tunnel), None);
        assert_eq!((held.builder.unit(), held.builder.tunnel()), (Some(ObjectId(4)), None));
        assert_eq!((site.builder.unit(), site.builder.tunnel()), (None, Some(ObjectId(3))));
    }

    /// The three builders hash apart, by their tags. A site built from outside starts at its def's share, as a held
    /// builder's does (slice M5-E: a Storage's 25 of 250).
    #[test]
    fn the_builders_hash_apart() {
        let hash = |builder: Builder| {
            let mut h = StateHasher::new();
            builder.hash(&mut h);
            h.finish()
        };
        let builders = [
            Builder::Held(ObjectId(5)),
            Builder::Tunnel(ObjectId(5)),
            Builder::Recruits,
        ];
        for (i, a) in builders.iter().enumerate() {
            for b in &builders[i + 1..] {
                assert_ne!(hash(*a), hash(*b), "{a:?} {b:?}");
            }
        }
        let storage = Construction::new(site_def(ObjectKind::Storage).unwrap(), Builder::Recruits);
        assert_eq!(storage.max_hp_of(ObjectKind::Storage, 250.0), Some(25.0));
        assert_eq!((Builder::Recruits.unit(), Builder::Recruits.tunnel()), (None, None));
    }

    /// A record of a site keeps its progress, not who builds it (slice M4-I2): its maximum takes the share it started
    /// at by its kind, as its builder's def gave it, a Tunnel site's halfway 330 of 600 and a Hideout's 220 of 400.
    /// No kind is built both ways (since design change 0359, which took the Laser Turret from the Agents), so its kind
    /// tells its builder; none for a kind never built so.
    #[test]
    fn a_record_takes_the_rule_by_its_kind() {
        let halfway = |frames: u32| Progress {
            elapsed: frames / 2 * RATIO_ONE,
            total: frames,
        };
        let tunnel = Construction::new(site_def(ObjectKind::Tunnel).unwrap(), Builder::Held(ObjectId(4)));
        let tunnel = Construction {
            progress: halfway(tunnel.progress.total),
            ..tunnel
        };
        let hideout = Construction::expansion(expansion_def(ObjectKind::Hideout).unwrap(), ObjectId(3));
        let hideout = Construction {
            progress: halfway(hideout.progress.total),
            ..hideout
        };
        for (kind, site, full, max) in [
            (ObjectKind::Tunnel, tunnel, 600.0, 330.0),
            (ObjectKind::Hideout, hideout, 400.0, 220.0),
        ] {
            let seen = max_hp_seen(kind, &site.progress, full).unwrap();
            assert!((seen - max).abs() < 1e-3, "{kind:?}: {seen}");
            assert_eq!(Some(seen), site.max_hp_of(kind, full), "{kind:?}");
        }
        for &kind in ObjectKind::ALL {
            let built = site_def(kind).is_some() || expansion_def(kind).is_some();
            assert!(
                site_def(kind).is_none() || expansion_def(kind).is_none(),
                "{kind:?}: built both ways"
            );
            assert_eq!(max_hp_seen(kind, &halfway(100), 100.0).is_some(), built, "{kind:?}");
        }
    }

    /// The Agent builds the Tunnel, alone (the Laser Turret no more: design change 0359); the Recruit the Recruitment
    /// Hall, the Storage and the Armory (slice M5-E), a group sharing the build, and assists them; nothing else builds
    /// anything (the Drone deploys).
    #[test]
    fn the_agent_and_the_recruit_build_their_own() {
        let cults = [ObjectKind::RecruitmentHall, ObjectKind::Storage, ObjectKind::Armory];
        for &builder in ObjectKind::ALL {
            for &kind in ObjectKind::ALL {
                let expected = match builder {
                    ObjectKind::Agent => kind == ObjectKind::Tunnel,
                    ObjectKind::Recruit => cults.contains(&kind),
                    _ => false,
                };
                assert_eq!(builds(builder, kind), expected, "{builder:?} {kind:?}");
                let assisted = builder == ObjectKind::Recruit && expected;
                assert_eq!(assists(builder, kind), assisted, "{builder:?} {kind:?}");
            }
            let builders = [ObjectKind::Agent, ObjectKind::Recruit];
            assert_eq!(builds_any(builder), builders.contains(&builder), "{builder:?}");
            assert_eq!(shares_builds(builder), builder == ObjectKind::Recruit, "{builder:?}");
        }
    }

    /// A price in its def's resource (`SiteDef::price`): a Tunnel's one Supply for each owned, a Storage's 100 Space
    /// Crystals however many stand. A cancelled site gives back a Tunnel's whole "current top price" (design changes
    /// 0233 and 0284), a Cults building's 75%, rounded down (`factions/cults/economy.md > Cancel Construction`).
    #[test]
    fn a_cancelled_crystals_site_gives_back_three_quarters() {
        let (supplies, crystals) = (BuildPrice::supplies, BuildPrice::crystals);
        let (tunnel, storage) = (SitePrice::SuppliesPerOwned, SitePrice::Crystals(100));
        assert_eq!(
            (BuildPrice::of(tunnel, 3), BuildPrice::of(storage, 3)),
            (supplies(3), crystals(100))
        );
        assert_eq!(supplies(3).cancelled(tunnel), supplies(3));
        assert_eq!(crystals(100).cancelled(storage), crystals(75));
        assert_eq!(crystals(1).cancelled(storage), crystals(0));
    }
}
