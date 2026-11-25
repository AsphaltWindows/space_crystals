//! What a placement is checked against ([`Site`]): the ground, the build area, the [`Mode`] and the [`Method`], and
//! the [`Occupant`]s, everything on the map.

use glam::Vec2;

use super::build_area::BuildArea;
use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::map::Map;
use crate::nav::TileRect;
use crate::object::{Object, Placement};
use crate::vision::Vision;

/// Something on the map that a footprint must keep clear of, or is placed onto: an object of a game, or one of a map
/// or a Starting Layout that hasn't started.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Occupant {
    pub kind: ObjectKind,
    /// Its centre, space units; a unit's box is around it.
    pub pos: Vec2,
    /// A tile-based object's footprint.
    pub placement: Option<Placement>,
    /// A Tunnel's tier as the check knows it: its tier (`Object::tunnel`), or a record's tier as last seen (design
    /// change 0237: "their tier as last seen"), which sets how far its Tunnel Area reaches for the non-overlap rule
    /// ([`WholeProblem::TunnelAreaOverlap`](super::WholeProblem::TunnelAreaOverlap), slice M4-E). `None` for every
    /// other object, and for a Tunnel of a map or of a Starting Layout, which counts at the base tier as it starts
    /// there.
    pub tier: Option<u8>,
}

impl Occupant {
    /// What object `o` is to a placement: its kind, centre, footprint and tier. Only an object with a footprint has a
    /// Tunnel Area, so a unit's tier is never read: the check of a crowded map walks the units as it did before tiers
    /// (`tests/placement_perf.rs`).
    pub fn of(o: &Object) -> Self {
        let tier = match o.placement {
            Some(_) => o.tunnel.as_ref().map(|tunnel| tunnel.tier),
            None => None,
        };
        Self {
            kind: o.kind,
            pos: o.pos,
            placement: o.placement,
            tier,
        }
    }
}

/// How a placement is checked: in a game, or for a Starting Layout.
#[derive(Copy, Clone, Debug)]
pub enum Mode<'a> {
    /// A player's placement in a game (`core/entities.md > Placement Validation > Direct Placement`,
    /// `core/entities.md > Warp In`): every footprint tile must be Visible to `player` in `vision`, as the latest tick
    /// left it; an Extraction Plate's too (design change 0095), and a Warp In's (design change 0119).
    Command { vision: &'a Vision, player: PlayerId },
    /// A worker-built structure's order, its gate and its ghost (plan decision T16; `core/entities.md > Placement
    /// Validation > Worker-Built Structures`, design change 0120: "The ghost judges by what the player knows"; slice
    /// M3-F, `crate::Sim::deploy_check`). The caller gives the occupants `player` knows of: its own objects, the others
    /// it sees now, and its records of the structures and resources out of its sight, as they were when last seen
    /// ("units hidden in fog are ignored"). On a tile `player` has never explored in `vision`, what stands there and
    /// the ground are unknown, and the tile "passes": only the player's own reservations
    /// ([`TileProblem::Reserved`](super::TileProblem::Reserved)) count there. The map's edge is known: a tile off the
    /// map never passes.
    Known { vision: &'a Vision, player: PlayerId },
    /// A worker's check of its site on arrival (`core/entities.md > Worker-Built Structures`: "validates on arrival";
    /// `core/entities.md > Lingering build ghosts`: "No visibility requirement on arrival (the worker is physically
    /// present)"): the truth, with no sight rule.
    Arrival,
    /// An object of a Starting Layout (`core/map_editor.md > Starting Layouts > Layout validation`, design change
    /// 0097): the faction's "normal in-game placement rules, minus visibility", as nothing is "visible" in the editor.
    /// The build area to reach is the layout's own ([`BuildArea::of_layout`]).
    Layout,
}

/// How the structure comes to stand (plan decision T16: one check, whose rules follow the method;
/// `core/entities.md > Placement Validation`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Method {
    /// `core/entities.md > Direct Placement`, the GDO's: placed at once from a menu, onto tiles the player sees (in a
    /// game), within the build area for a GDO building.
    Direct,
    /// `core/entities.md > Warp In` (`factions/colonists/economy.md > Warp In`): the Beacon standing at `beacon`
    /// transforms in place into the structure, on the footprint `crate::transform::warp_in_footprint` derives from it
    /// (design change 0121), which the caller checks. Every footprint tile must be Buildable, flat and unoccupied
    /// "except for the Beacon's own footprint, which will be consumed by the transformation": the tiles under `beacon`
    /// break no rule but sight. A ground unit of any player occupies its tiles, air and underground units never do
    /// (design change 0118), as for a Direct Placement. Every tile must be Visible, as for a Direct Placement (design
    /// change 0119: "a tile that isn't is red on the ghost", whatever stands there), the Beacon's own included (its
    /// owner's Beacon always sees them). There is no build area.
    WarpIn { beacon: Placement },
    /// `core/entities.md > Worker-Built Structures` (slice M3-F, a Drone deploying a Beacon, `crate::deploy`): built by
    /// a worker on a site it walks to. The tile rules of a Direct Placement, sight aside (the [`Mode`] decides what is
    /// seen), and no build area ("no range limit from other Colonists buildings",
    /// `factions/colonists/economy.md > Beacon Deployment > Placement`). "The building worker itself (e.g. the
    /// deploying Drone) doesn't count" (design change 0118): the caller leaves it out of the occupants, as a Warp In's
    /// caller leaves out its Beacon.
    WorkerBuilt,
    /// `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)` (slice M4-F, plan decisions T32 and T53;
    /// `core/entities.md > Direct Placement`, "Tunnel underground expansions"): an underground expansion a Tunnel
    /// builds in its Tunnel Area, `area` (the Tunnel's at its current tier, `crate::tunnel::area::of`; a Starting
    /// Layout's Tunnel's at Tier 1). "The expansion must fit entirely within the Tunnel Area"
    /// ([`WholeProblem::OutsideTunnelArea`](super::WholeProblem::OutsideTunnelArea)); "Every tile under it must be
    /// Drillable" (design change 0239: [`TileProblem::NotDrillable`](super::TileProblem::NotDrillable)), in place of
    /// Buildable; no flatness (`core/map.md > Elevation`) and no sight ("Surface visibility is not relevant"), whatever
    /// the [`Mode`]. Every structure blocks it, whoever owns it and whichever its layer, the Tunnel's own footprint
    /// included (design change 0240: "no expansion can be placed under any surface building, any player's"); so do the
    /// resources (0241), the ground units (Placeholder: Q0080 point 1, `core/entities.md > Direct Placement` read
    /// literally) and the owner's lingering ghosts (Placeholder: Q0080 point 2), as for the other methods.
    Expansion { area: TileRect },
}

/// What a placement is checked against besides the objects on the map: the ground, the build area the footprint
/// must reach, the mode and the method.
#[derive(Copy, Clone, Debug)]
pub struct Site<'a> {
    pub map: &'a Map,
    /// In a game the placing player's build area ([`crate::Sim::build_area`]), for a Starting Layout the layout's.
    pub area: &'a BuildArea,
    pub mode: Mode<'a>,
    pub method: Method,
}
