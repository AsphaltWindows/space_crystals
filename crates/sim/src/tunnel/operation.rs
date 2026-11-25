//! A Tunnel's one operation (slice M4-F; `factions/syndicate/tunnels.md > Construction and Upgrade Rules`: "A Tunnel
//! can only perform one operation at a time"): `TunnelState::operation`, which the gate reads before a Tunnel starts
//! another (`sim/expansions.rs`, `sim/upgrades.rs`) and which the Tunnel's X cancels (`factions/syndicate/tunnels.md >
//! Tunnel ObjectInterfaceState`: "**X** Cancel (upgrade, construction or the current operation)").
//!
//! - **Expansions** ([`Operation::Building`]): the site the Tunnel builds in its Tunnel Area, whose builder is the
//!   Tunnel (`construction::site::Builder::Tunnel`). Its progress is the site's, the shared `Progress`
//!   (`Construction::progress`), never a copy. A Laser Turret a Tunnel builds will take the slot through `Building`
//!   with slice S2 (`factions/syndicate/structures/laser_turret.md > Construction`, design change 0267: it "uses the
//!   Tunnel's single operation slot").
//! - **Upgrades** ([`Operation::Upgrading`], slice M4-G, `sim/upgrades.rs`): the Tunnel upgrades to the tier above
//!   its own, and the operation keeps the upgrade's progress, a `Progress` of the tier's upgrade frames
//!   (`defs::UpgradeDef::frames`). "Upgrading temporarily locks the Tunnel out of expansion construction for the
//!   upgrade duration" (`factions/syndicate/tunnels.md > Construction and Upgrade Rules`): the slot is taken.
//! - **Eject** is none: "Eject still works (it isn't an operation)" (`factions/syndicate/tunnels.md > Expansions under
//!   construction`, design change 0254).
//!
//! The slot ends with what it does: completed, cancelled, or its site gone from the game (`Sim::remove_objects`, the
//! reference cleanup); an upgrade goes with its Tunnel. Hashed only while set, and left out of files while `None`.

use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::progress::Progress;

/// What a Tunnel is doing as its one operation (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Building the expansion that is this site, in its Tunnel Area (slice M4-F): its progress is the site's.
    Building(ObjectId),
    /// Upgrading to tier `to`, the one above the Tunnel's own (slice M4-G): `progress` counts the frames of that
    /// tier's upgrade. Until it completes the Tunnel is at its tier, but for its Tunnel Area, of which it reserves the
    /// one of `to` ([`super::TunnelState::reserved_tier`]).
    Upgrading { progress: Progress, to: u8 },
}

impl Operation {
    /// The site it builds, if it builds one.
    pub fn site(self) -> Option<ObjectId> {
        match self {
            Operation::Building(site) => Some(site),
            Operation::Upgrading { .. } => None,
        }
    }

    /// The tier it upgrades to, if it is an upgrade.
    pub fn upgrade_to(self) -> Option<u8> {
        match self {
            Operation::Building(_) => None,
            Operation::Upgrading { to, .. } => Some(to),
        }
    }
}

impl StateHash for Operation {
    fn hash(&self, h: &mut StateHasher) {
        match self {
            Operation::Building(site) => {
                h.tag(0);
                site.hash(h);
            }
            // (M4-G) Its own tag, then its progress and its tier.
            Operation::Upgrading { progress, to } => {
                h.tag(1);
                progress.hash(h);
                to.hash(h);
            }
        }
    }
}
