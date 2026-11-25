//! A Tunnel's one operation (slice M4-F; `factions/syndicate/tunnels.md > Construction and Upgrade Rules`: "A Tunnel
//! can only perform one operation at a time"): `TunnelState::operation`, which the gate reads before a Tunnel starts
//! another (`sim/expansions.rs`) and which the Tunnel's X cancels (`factions/syndicate/tunnels.md > Tunnel
//! ObjectInterfaceState`: "**X** Cancel (upgrade, construction or the current operation)").
//!
//! - **Expansions** ([`Operation::Building`]): the site the Tunnel builds in its Tunnel Area, whose builder is the
//!   Tunnel (`construction::site::Builder::Tunnel`). Its progress is the site's, the shared `Progress`
//!   (`Construction::progress`), never a copy.
//! - **Upgrades** join with slice M4-G, and a Laser Turret a Tunnel builds takes the slot through `Building` with slice
//!   S2 (`factions/syndicate/structures/laser_turret.md > Construction`, design change 0267: it "uses the Tunnel's
//!   single operation slot").
//! - **Eject** is none: "Eject still works (it isn't an operation)" (`factions/syndicate/tunnels.md > Expansions under
//!   construction`, design change 0254).
//!
//! The slot ends with what it does: completed, cancelled, or its site gone from the game (`Sim::remove_objects`, the
//! reference cleanup). Hashed only while set, and left out of files while `None`.

use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;

/// What a Tunnel is doing as its one operation (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Building the expansion that is this site, in its Tunnel Area (slice M4-F): its progress is the site's.
    Building(ObjectId),
}

impl Operation {
    /// The site it builds, if it builds one.
    pub fn site(self) -> Option<ObjectId> {
        match self {
            Operation::Building(site) => Some(site),
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
        }
    }
}
