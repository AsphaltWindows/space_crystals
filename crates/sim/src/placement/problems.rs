//! What a placement check finds ([`PlacementCheck`]): the rules each footprint tile breaks ([`TileProblem`]) and those
//! the footprint breaks as a whole ([`WholeProblem`]), with their words.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::units::TilePos;

/// A rule one footprint tile breaks (`core/entities.md > Placement Validation > Direct Placement`), in the order they
/// are checked. The placement ghost shows them tile by tile (design change 0093).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TileProblem {
    /// The tile lies off the map; nothing else is checked on it.
    OffMap,
    /// The tile's preset isn't Buildable (`core/map.md > TilePreset`; of the defaults only Plane is,
    /// `core/map.md > DefaultTilePresets`).
    NotBuildable,
    /// An underground expansion's tile isn't Drillable ([`super::Method::Expansion`], slice M4-F;
    /// `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`, design change 0239: "Every tile under it must
    /// be Drillable (... Plane, Rugged and Cliff yes; Mountain and Water no ...)"), which it needs in place of
    /// Buildable. The preset's flag, a custom preset's included (`core/map.md > Tile`).
    NotDrillable,
    /// The tile is sloped (`core/map.md > Elevation > Structures only on flat terrain`).
    NotFlat,
    /// The tile isn't Visible to the placing player (in a game: a Direct Placement or a Warp In, design change
    /// 0119). It is then the tile's only problem: what stands on a tile the player doesn't see, and the terrain of one
    /// they never saw, is hidden from them (`core/vision.md > VisibilityStateEnum`), and the tile is invalid anyway.
    NotVisible,
    /// Another structure's footprint covers the tile ("no existing structure overlap"), whoever owns it: a plate on
    /// the patch, for another plate.
    Structure,
    /// A map resource covers the tile, and the footprint isn't a plate going onto a patch (`core/entities.md >
    /// Placement Validation > All placement methods: no buildings on resources`, design change 0096).
    Resource,
    /// A ground unit's box overlaps the tile, whoever owns it: units are never pushed aside, and air and underground
    /// units don't count (design change 0094).
    Unit,
    /// One of the placing player's pending worker builds covers the tile (`core/entities.md > Placement Validation >
    /// Lingering build ghosts`: it "reserves the site against the owner's further placements: a new ghost overlapping
    /// it is invalid", design change 0125): a Deploy (slice M3-F) or a Build (slice M4-E), current or queued
    /// (`crate::Sim::pending_builds`). The tile's only problem on ground the player never explored
    /// ([`super::Mode::Known`]).
    Reserved,
}

impl TileProblem {
    /// Every problem, in the order they are listed.
    pub const ALL: [TileProblem; 9] = [
        TileProblem::OffMap,
        TileProblem::NotBuildable,
        TileProblem::NotDrillable,
        TileProblem::NotFlat,
        TileProblem::NotVisible,
        TileProblem::Structure,
        TileProblem::Resource,
        TileProblem::Unit,
        TileProblem::Reserved,
    ];

    /// Its bit in a tile's set of problems: a u16, room for 16 (plan decision T53, widened from a u8 that the first
    /// eight filled).
    pub(super) const fn bit(self) -> u16 {
        1 << self as u16
    }
}

// Every problem has a bit of its own in the set.
const _: () = assert!(TileProblem::ALL.len() <= u16::BITS as usize);

/// A rule the footprint breaks as a whole, in the order they are checked.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WholeProblem {
    /// Only structures are placed this way, not units or map resources; nothing else is checked.
    NotAStructure,
    /// No tile of the GDO building is in the build area (`factions/gdo/structures/README.md > GDOBuildArea`: "at least
    /// 1 grid cell of the new building must be within the current build area"). A Deployment Center seeds the area and
    /// needs none ([`super::seeds_build_area`]).
    OutsideBuildArea,
    /// The Extraction Plate isn't exactly on a Space Crystals Patch (`factions/gdo/economy.md > ExtractionFacility >
    /// AwaitingPlacement`: "Left-click Space Crystal Patch within GDO build area that does not already have a plate").
    /// A patch that has a plate is the [`TileProblem::Structure`] of that plate.
    NotOnPatch,
    /// A cell of the Tunnel Area that the Tunnel would have at Tier 1 lies past the map's edge (slice M4-B;
    /// `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule > Drillable area`, design change 0242: "a Tunnel
    /// can only be placed if every cell of its Tunnel Area is on the map and Drillable ... So no Tunnel near the map
    /// edge (closer than its area reaches)"). The map's edge is known: this counts on ground never explored too
    /// ([`super::Mode::Known`]). The rule is in `placement/tunnel_area.rs`.
    TunnelAreaOffMap,
    /// A cell of that area lies on ground that isn't Drillable (0242: "no Water or Mountain"): a preset without the
    /// flag, a custom one included (`core/map.md > Tile`: "Any combination of properties is valid"). In
    /// [`super::Mode::Known`] only on ground the player has explored ("never-explored tiles pass on the ghost").
    TunnelAreaNotDrillable,
    /// The Tunnel Area that the Tunnel would have at Tier 1 overlaps the Tunnel Area of another Tunnel, any player's, a
    /// site included, at that Tunnel's tier (slice M4-E; `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap
    /// Rule`, design change 0237: "Every player's Tunnels count"; question Q0079 point 3), or the Tier 1 area of one of
    /// the placing player's lingering Tunnel ghosts (Placeholder: Q0079 point 2). In [`super::Mode::Known`] another
    /// player's Tunnel counts as the player knows it, "their remembered position and their tier as last seen". The
    /// rule is in `placement/tunnel_overlap.rs`.
    TunnelAreaOverlap,
    /// An underground expansion doesn't lie wholly inside the Tunnel Area it is placed in
    /// ([`super::Method::Expansion`], slice M4-F; `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`: "The
    /// expansion must fit entirely within the Tunnel Area"): that of the Tunnel building it, at its current tier; for a
    /// Starting Layout's, the Tier 1 Tunnel Area of a Tunnel of the same layout (slice M4-B;
    /// `factions/syndicate/tunnels.md > Tunnel > Starting Conditions`: "One pre-built **Tier 1 Tunnel** containing one
    /// pre-built **Hideout**"; `factions/syndicate/structures/README.md > Tunnel Expansions`: "Underground buildings
    /// constructed within a Tunnel's Tunnel Area"), as a layout's Extraction Plates must lie in the build area of its
    /// Deployment Center ([`super::check_layout`]).
    OutsideTunnelArea,
}

/// Everything that keeps a placement from being valid ([`super::check`]): nothing, if it is.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementCheck {
    /// Each rule each footprint tile breaks: the tiles row by row from the north-west, each one's problems in the
    /// order of [`TileProblem`]. A tile without a problem isn't listed.
    pub tiles: Vec<(TilePos, TileProblem)>,
    /// The rules the footprint breaks as a whole, in the order of [`WholeProblem`].
    pub whole: Vec<WholeProblem>,
}

impl PlacementCheck {
    /// Whether the placement breaks no rule: the ghost is green, and the building may be placed.
    pub fn is_valid(&self) -> bool {
        self.tiles.is_empty() && self.whole.is_empty()
    }

    /// The rules `tile` breaks, in order.
    pub fn problems_at(&self, tile: TilePos) -> impl Iterator<Item = TileProblem> + '_ {
        self.tiles.iter().filter(move |(t, _)| *t == tile).map(|&(_, p)| p)
    }

    /// Whether `tile` breaks no rule of its own (the footprint as a whole still may).
    pub fn tile_is_valid(&self, tile: TilePos) -> bool {
        self.problems_at(tile).next().is_none()
    }

    /// The first rule the placement breaks, for a short answer such as the command gate's
    /// (`crate::Reject::Unplaceable`): the footprint's first as a whole, else its first tile's, in the order they are
    /// listed. `None` if it is valid.
    pub fn first_problem(&self) -> Option<PlacementProblem> {
        let whole = self.whole.first().map(|&problem| PlacementProblem::Whole(problem));
        whole.or_else(|| {
            self.tiles
                .first()
                .map(|&(tile, problem)| PlacementProblem::Tile(tile, problem))
        })
    }
}

/// One rule a placement breaks ([`PlacementCheck::first_problem`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlacementProblem {
    /// A rule the footprint breaks as a whole.
    Whole(WholeProblem),
    /// A rule this footprint tile breaks.
    Tile(TilePos, TileProblem),
}

impl fmt::Display for TileProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TileProblem::OffMap => "off the map",
            TileProblem::NotBuildable => "not buildable ground",
            TileProblem::NotDrillable => "not drillable ground",
            TileProblem::NotFlat => "sloped",
            TileProblem::NotVisible => "not in sight",
            TileProblem::Structure => "under a structure",
            TileProblem::Resource => "under a resource",
            TileProblem::Unit => "under a ground unit",
            TileProblem::Reserved => "under a building already ordered",
        })
    }
}

impl fmt::Display for WholeProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            WholeProblem::NotAStructure => "only structures are placed",
            WholeProblem::OutsideBuildArea => "no tile is in the build area",
            WholeProblem::NotOnPatch => "an Extraction Plate goes onto a Space Crystals patch",
            WholeProblem::TunnelAreaOffMap => {
                "the Tunnel Area reaches off the map: a Tunnel's whole Tunnel Area must be on the map"
            }
            WholeProblem::TunnelAreaNotDrillable => {
                "the Tunnel Area takes in undrillable ground: a Tunnel's whole Tunnel Area must be Drillable (no Water \
                 or Mountain)"
            }
            WholeProblem::TunnelAreaOverlap => {
                "the Tunnel Area overlaps another Tunnel's: Tunnel Areas never overlap, whoever owns them"
            }
            WholeProblem::OutsideTunnelArea => "an expansion lies wholly inside a Tunnel's Tunnel Area",
        })
    }
}

impl fmt::Display for PlacementProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlacementProblem::Whole(problem) => write!(f, "{problem}"),
            PlacementProblem::Tile(tile, problem) => write!(f, "tile ({}, {}) is {problem}", tile.x, tile.y),
        }
    }
}
