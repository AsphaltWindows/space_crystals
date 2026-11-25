//! On-site construction (plan decision T31, slice M4-E): a structure that a worker builds where it stands, from its
//! first frame the object it will be — the Tunnel an Agent builds (`factions/syndicate/units/agent.md > Building >
//! Tunnel Construction Flow`: "the partially-built Tunnel appears at the location with the Agent inside it").
//!
//! - **The state** ([`Construction`], `Object::construction`): its progress, the shared [`Progress`] over its def's
//!   frames (`SiteDef::frames`), and its [`Builder`], the unit held inside it (`network::Place::Site`). Hashed only
//!   while set, and left out of files while `None`, so that the states of games without a site are as they were.
//! - **HP** (`core/entities.md > ConstructionHP Rule`, design change 0234): the rule gives the **maximum**,
//!   [`Construction::max_hp`]: "MaxHP x (10% + 90% x construction_progress)", the share it starts at being its def's
//!   (`SiteDef::start_hp`). The current HP follows the maximum as it rises, so the damage taken stays
//!   (`transform::follow_max`, the Warp In's rule).
//! - **Operational** ([`Object::is_operational`]): a site does nothing but cancel (design change 0234: "no Tunnel
//!   Space, no drop-offs, no Enter, no Expand, no Eject"), and neither does a building still transforming. One
//!   predicate, for every function that waits for completion: the exits and the Enter target (`network::is_exit`),
//!   the drop-offs (`orders::world`, `Sim::nearest_drop_off`, the gate) and Tunnel Space (`economy::army`).
//! - **The order, the price and the steps** are the game's: `sim/builds.rs` and `sim/sites.rs`.

use serde::{Deserialize, Serialize};

use crate::defs::{ObjectKind, SiteDef};
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::object::Object;
use crate::progress::Progress;

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
}

impl Builder {
    /// The unit that builds.
    pub fn unit(self) -> ObjectId {
        match self {
            Builder::Held(unit) => unit,
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

    /// `core/entities.md > ConstructionHP Rule`: the maximum HP of a site of `def`'s whose structure has `full` MaxHP
    /// once built: "MaxHP x (10% + 90% x construction_progress)", the progress in whole frames done
    /// ([`Progress::ratio`]).
    pub fn max_hp(&self, def: &SiteDef, full: f32) -> f32 {
        full * (def.start_hp + (1.0 - def.start_hp) * self.progress.ratio())
    }
}

/// How a worker builds a structure of `kind` (`StructureDef::site`): `None` for a kind no worker builds.
pub fn site_def(kind: ObjectKind) -> Option<&'static SiteDef> {
    kind.def().structure.as_ref()?.site.as_ref()
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

impl Object {
    /// Whether it is a site, a structure under construction (module docs).
    pub fn is_site(&self) -> bool {
        self.construction.is_some()
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

    /// The Agent builds the Tunnel; nothing else builds anything (the Drone deploys).
    #[test]
    fn only_the_agent_builds_and_only_tunnels() {
        for &builder in ObjectKind::ALL {
            for &kind in ObjectKind::ALL {
                let expected = builder == ObjectKind::Agent && kind == ObjectKind::Tunnel;
                assert_eq!(builds(builder, kind), expected, "{builder:?} {kind:?}");
            }
            assert_eq!(builds_any(builder), builder == ObjectKind::Agent, "{builder:?}");
        }
    }
}
