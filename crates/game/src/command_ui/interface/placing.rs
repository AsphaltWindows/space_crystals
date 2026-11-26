//! AwaitingPlacement's building (`factions/gdo/structures/deployment_center.md >
//! ObjectInterfaceState[DeploymentCenter] > AwaitingPlacement`, `factions/gdo/economy.md > ExtractionFacility >
//! ObjectInterfaceState[ExtractionFacility] > AwaitingPlacement`, `factions/colonists/units/drone.md > Drone
//! ObjectInterfaceState > A: Deploy`, `factions/colonists/economy.md > Warp In > Warp In Command`,
//! `factions/syndicate/tunnel_menus.md > AwaitingPlacement (Expansion)`): which one is placed and who builds it
//! ([`Placing`], [`Builder`]; plan decision T23), where its ghost stands ([`Placing::origin`]), and how the placement
//! keys turn it ([`Placing::turned`]). A Deploy given to a group of Drones goes to one of them ([`deploy_recipient`],
//! design change 0125). An Agent's Build Tunnel places the Tunnel its Agent walks to and builds
//! (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState > A: Build Tunnel`, slice M4-J1), a Tunnel's
//! ExpandMenu the expansion it builds in its Tunnel Area (slice M4-F). The ghost and the overlays that show it are
//! `crate::placement`'s.
//!
//! The placement keys are the ones the map editor turns its structures with, one definition for both: the shared
//! placement actions of the input layer (`crate::input::InputAction::is_placement`, which take R, Shift+R, F and
//! Shift+F while the game reports a placement, `crate::input::Placing`), and their turns
//! (`crate::editor::orientation::Turn`: as seen on the map).

use std::collections::BTreeSet;

use bevy::math::Vec2;
use sc_sim::nav::TileRect;
use sc_sim::placement::footprint_size;
use sc_sim::{
    BuildingOrientation, Command, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, SU_PER_GU, Sim, TilePos,
    deploy, transform, tunnel,
};

use super::PanelCommand;
use crate::editor::orientation::Turn;
use crate::selection::Selection;

/// Who builds the building placed (plan decision T23: "a constructor, a worker or a Beacon"; a Tunnel its expansions,
/// slice M4-F), which decides where its ghost stands, what checks it and what a click sends.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Builder {
    /// A GDO constructor holding the building ready: the ActiveGroup's one Deployment Center or Extraction Facility
    /// (`core/entities.md > Placement Validation > Direct Placement`: the ghost under the cursor,
    /// `Sim::placement_check`, `ConstructionCommand::Place`). Placement never moves on to another one
    /// (`systems/controls/selection.md > SelectionGroup > No smart-casting`, design change 0071).
    Constructor(ObjectId),
    /// The Colonists' Drones of the ActiveGroup, one of which each click sends to deploy into a Beacon
    /// (`core/entities.md > Worker-Built Structures`: the ghost under the cursor, `Sim::deploy_check`,
    /// `OrderKind::Deploy`; [`deploy_recipient`]).
    Drones {
        /// A Shift-click has sent a Deploy and a Drone of the group has none yet (`systems/controls/interface.md
        /// > AwaitingTarget > Shift while placing > Exception, Drones`, design change 0125): "Shift-placement stays
        /// active and each Shift-click goes to the next Drone", while Shift is held; letting go of it ends placement
        /// (`systems/controls/interface.md > Shift while placing`: "releasing Shift ends placement").
        queued: bool,
    },
    /// A Beacon warping the building in on itself (`factions/colonists/economy.md > Warp In`): the ghost is the
    /// footprint derived from the Beacon and the orientation (`sc_sim::transform::warp_in_footprint`, design change
    /// 0121: "No cursor snapping"), wherever the cursor is; `Sim::warp_in_check`, `ConstructionCommand::WarpIn`.
    Beacon(ObjectId),
    /// The Syndicate's Agent of the ActiveGroup (Ungroupable: the one Tabbed to), which walks to the site and builds
    /// the building there (`factions/syndicate/units/agent.md > Agent ObjectInterfaceState > A: Build Tunnel`, slice
    /// M4-J1; `core/entities.md > Worker-Built Structures`): the ghost under the cursor, `Sim::build_check`,
    /// `OrderKind::Build`. "Each Agent takes exactly one placement": a click, Shift held or not, sends the Build and
    /// ends placement, which never moves on to another Agent (`systems/controls/interface.md > AwaitingTarget
    /// > Shift while placing`: "to place with the next Agent, the player presses Tab").
    Agent(ObjectId),
    /// The ActiveGroup's Tunnel building an expansion in its Tunnel Area (`factions/syndicate/tunnel_menus.md >
    /// AwaitingPlacement (Expansion)`: "A ghost preview of the expansion follows the cursor within the Tunnel Area,
    /// snapped to the grid"; [`Placing::origin`]): `Sim::expansion_check`, as far as the player knows, then
    /// `TunnelCommand::Expand`, which the sim checks by the truth (design change 0240).
    Tunnel(ObjectId),
}

/// `systems/controls/hud.md > AwaitingPlacement`: the building that the local player is placing, and who
/// builds it (plan decisions T10, T23).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Placing {
    pub builder: Builder,
    /// The building placed: the ready one, the Beacon a Drone deploys into, the one a Beacon warps in.
    pub kind: ObjectKind,
    /// How the ghost is turned (`core/entities.md > Structure Type`: flipped first, then rotated clockwise), which a
    /// click places it with. Unturned as placement starts (placeholder: the design doesn't say whether the last
    /// orientation is kept).
    pub orientation: BuildingOrientation,
}

impl Placing {
    /// The placement of `constructor`'s ready `kind`, unturned.
    pub fn new(constructor: ObjectId, kind: ObjectKind) -> Self {
        Self {
            builder: Builder::Constructor(constructor),
            kind,
            orientation: BuildingOrientation::default(),
        }
    }

    /// The placement of the Beacon a Drone of the ActiveGroup deploys into (`factions/colonists/units/drone.md > Drone
    /// ObjectInterfaceState > A: Deploy`), no Deploy sent yet.
    pub fn deploy() -> Self {
        Self {
            builder: Builder::Drones { queued: false },
            kind: deploy::BEACON_DEPLOYMENT.kind,
            orientation: BuildingOrientation::default(),
        }
    }

    /// The placement of the `kind` that Beacon `beacon` warps in, unturned: centred on the Beacon, an odd size's
    /// extras at the bottom-right (`factions/colonists/economy.md > Warp In Command`, design change 0121).
    pub fn warp_in(beacon: ObjectId, kind: ObjectKind) -> Self {
        Self {
            builder: Builder::Beacon(beacon),
            kind,
            orientation: BuildingOrientation::default(),
        }
    }

    /// The placement of the `kind` that Agent `agent` builds (`factions/syndicate/units/agent.md > Agent
    /// ObjectInterfaceState > A: Build Tunnel`), unturned.
    pub fn build(agent: ObjectId, kind: ObjectKind) -> Self {
        Self {
            builder: Builder::Agent(agent),
            kind,
            orientation: BuildingOrientation::default(),
        }
    }

    /// The placement of the expansion `kind` that Tunnel `tunnel` builds (`factions/syndicate/tunnel_menus.md >
    /// ExpandMenu`: "Click an expansion type: [...] enters AwaitingPlacement for that expansion"), unturned.
    pub fn expansion(tunnel: ObjectId, kind: ObjectKind) -> Self {
        Self {
            builder: Builder::Tunnel(tunnel),
            kind,
            orientation: BuildingOrientation::default(),
        }
    }

    /// The panel command whose slot the placement belongs to, which the panel marks while it lasts: the ready
    /// building's, the Deploy, the menu entry of the building warped in or of the expansion built, the Agent's Build.
    pub fn command(self) -> PanelCommand {
        match self.builder {
            Builder::Constructor(_) => PanelCommand::PlaceReady(self.kind),
            Builder::Drones { .. } => PanelCommand::Deploy,
            Builder::Beacon(_) => PanelCommand::WarpIn(self.kind),
            Builder::Agent(_) => PanelCommand::Build(self.kind),
            Builder::Tunnel(_) => PanelCommand::Expand(self.kind),
        }
    }

    /// Whether the ghost follows the cursor ("follows cursor, snapped to grid"): all but a Warp In's, which stays on
    /// its Beacon.
    pub fn follows_cursor(self) -> bool {
        !matches!(self.builder, Builder::Beacon(_))
    }

    /// The footprint's size `(w, h)` in tiles, as turned: "Non-square buildings swap footprint dimensions when
    /// rotated to 90 or 270".
    pub fn size(self) -> Option<(u32, u32)> {
        footprint_size(self.kind, self.orientation)
    }

    /// The top-left tile of the ghost's footprint with the cursor on the ground point `ground`: "A ghost preview of
    /// the building follows the cursor, snapped to the grid". The footprint whose centre is nearest the point, so the
    /// cursor stays in the middle of the ghost (on the tile under it, for an odd size). `None` for a point that isn't
    /// finite, or a kind without a footprint.
    pub fn origin_under(self, ground: Vec2) -> Option<TilePos> {
        let (w, h) = self.size()?;
        if !ground.is_finite() {
            return None;
        }
        // Float to int casts saturate: a point far off the map gives a tile far off it, never a panic.
        let snap = |at: f32, size: u32| (at / SU_PER_GU - size as f32 / 2.0).round() as i32;
        Some(TilePos::new(snap(ground.x, w), snap(ground.y, h)))
    }

    /// The top-left tile of the ghost's footprint in `sim`, the cursor on the ground point `ground` if it is over the
    /// map: under the cursor ([`Placing::origin_under`]), an expansion's held within its Tunnel's area as it is now
    /// ([`within`]: "follows the cursor within the Tunnel Area", read as never leaving it: the cursor outside the area
    /// leaves the ghost at the nearest spot inside), or for a Warp In the footprint derived from the Beacon as it
    /// stands (`sc_sim::transform::warp_in_footprint`, design change 0121: "moving the mouse never changes the
    /// ghost"), wherever the cursor is. `None` without a footprint: no cursor over the map, a Beacon or a Tunnel gone.
    pub fn origin(self, sim: &Sim, ground: Option<Vec2>) -> Option<TilePos> {
        match self.builder {
            Builder::Beacon(beacon) => {
                let placement = sim.object(beacon)?.placement?;
                transform::warp_in_footprint(&placement, self.kind, self.orientation).map(|p| p.origin)
            }
            Builder::Tunnel(id) => {
                let area = tunnel::area::of(sim.object(id)?)?;
                Some(within(self.origin_under(ground?)?, self.size()?, area))
            }
            Builder::Constructor(_) | Builder::Drones { .. } | Builder::Agent(_) => self.origin_under(ground?),
        }
    }

    /// The same, with the ghost turned by `turn`: "R rotates the ghost 90 degrees clockwise, Shift+R rotates 90
    /// degrees counter-clockwise. [...] F flips the ghost horizontally, Shift+F flips vertically", as seen on the
    /// map, whichever way it is turned already; a Warp In's about the Beacon's centre (design change 0121,
    /// [`Placing::origin`]). A Deploy's Beacon isn't turned: the order has no orientation (`sc_sim::deploy::site`),
    /// and its sides are all alike (AAAA).
    pub fn turned(self, turn: Turn) -> Self {
        if matches!(self.builder, Builder::Drones { .. }) {
            return self;
        }
        Self {
            orientation: turn.apply(self.orientation),
            ..self
        }
    }
}

/// The top-left tile nearest `origin` of a footprint `size` tiles across that lies within `area`: the ghost of an
/// expansion held in its Tunnel Area ([`Placing::origin`]). A footprint larger than the area, which no expansion is,
/// stays at the area's north-west corner, where its check finds it outside.
pub fn within(origin: TilePos, (w, h): (u32, u32), area: TileRect) -> TilePos {
    let clamp = |at: i32, start: i32, len: u32, size: u32| {
        let last = start.saturating_add_unsigned(len.saturating_sub(size));
        at.clamp(start, last)
    };
    TilePos::new(
        clamp(origin.x, area.origin.x, area.w, w),
        clamp(origin.y, area.origin.y, area.h, h),
    )
}

/// Design change 0125 (`systems/controls/selection.md > SelectionGroup > Groupable workers used up by a
/// build`): the Drone of the ActiveGroup that a Deploy goes to, "the first in selection order (the SelectionGroup's
/// portrait order in the SelectionPanel) that can still take it": for a plain click the first one, whose orders the
/// Deploy replaces; for a Shift-click (`queue`) "the first Drone that has no Deploy in its queue yet", one not among
/// `deploying` ([`deploying`]). None when the ActiveGroup is no Drones, or no Drone of it can take a Shift-click's.
pub fn deploy_recipient(selection: &Selection, queue: bool, deploying: &BTreeSet<ObjectId>) -> Option<ObjectId> {
    let group = selection.active_group().filter(|g| deploy::deploys(g.kind()))?;
    group.instances.into_iter().find(|id| !queue || !deploying.contains(id))
}

/// The Drones of `player`'s that have a Deploy, current or queued: those the sim has (`Sim::pending_builds`), and
/// those named by a Deploy among the commands `unsent` that the sim hasn't applied yet (the client's
/// `PendingCommands`), so that two Shift-clicks before the next tick go to two Drones.
pub fn deploying(sim: &Sim, player: PlayerId, unsent: &[PlayerCommand]) -> BTreeSet<ObjectId> {
    let sent = unsent
        .iter()
        .filter(|c| c.player == player)
        .flat_map(|c| match &c.command {
            Command::Order {
                units,
                kind: OrderKind::Deploy { .. },
                ..
            } => units.clone(),
            _ => Vec::new(),
        });
    sim.pending_builds(player)
        .into_iter()
        .filter(|pending| pending.kind == deploy::BEACON_DEPLOYMENT.kind)
        .map(|pending| pending.worker)
        .chain(sent)
        .collect()
}
