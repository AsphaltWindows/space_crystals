//! Placement validation (slice M2-C; `core/entities.md > Placement Validation`): whether a structure may go
//! somewhere, and the GDO build area it must reach.
//!
//! **One pure check.** [`check`] holds every placement rule, so the command gate (the `Place` command), the client's
//! placement ghost and Starting Layout validation answer alike by construction: a game asks
//! [`Sim::placement_check`] ([`Mode::Command`]), a layout calls [`check`] in [`Mode::Layout`]. It looks at a
//! footprint, not an object: the [`ObjectKind`], the top-left tile of its footprint and its [`BuildingOrientation`],
//! against a [`Site`] (the map, the build area to reach and the mode) and the [`Occupant`]s, everything on the map.
//! It lists every problem ([`PlacementCheck`]): for each footprint tile the rules it breaks ([`TileProblem`]), which
//! the ghost shows tile by tile (design change 0093), and the rules the footprint breaks as a whole
//! ([`WholeProblem`]). Its output depends on its inputs only, in a fixed order.
//!
//! **Methods** (plan decision T16): the [`Site`] also says how the structure comes to stand ([`Method`]), and the
//! one check applies that method's rules. [`Method::Direct`] is the GDO's Direct Placement, as below. A Warp In
//! ([`Method::WarpIn`], slice M3-E; `core/entities.md > Placement Validation > Warp In`) has the same tile rules,
//! sight included (design change 0119), none but sight on the tiles of the Beacon it consumes, and no build area;
//! its footprint is derived from the Beacon (`crate::transform::warp_in_footprint`, design change 0121), and a game
//! asks [`Sim::warp_in_check`]. A worker-built structure ([`Method::WorkerBuilt`], slice M3-F: a Drone deploying a
//! Beacon, `crate::deploy`; `> Worker-Built Structures`) has the same tile rules and no build area; its order and its
//! ghost are checked on what the player knows ([`Mode::Known`], design change 0120: never-explored tiles pass, units
//! hidden in the fog don't count), a game asking [`Sim::deploy_check`], and the worker checks its site again on
//! arrival, on the truth ([`Mode::Arrival`]).
//!
//! **Reservations** (`> Lingering build ghosts`, design change 0125): a player's pending worker builds reserve their
//! sites against that player's further placements, a Deploy's and a Warp In's ([`check_reserving`],
//! [`TileProblem::Reserved`]). Another player's never matter, and units walk over them.
//!
//! **The rules**, in the order the problems are listed:
//! - each footprint tile: on the map; Buildable (`core/map.md > TilePreset`); flat (`core/map.md > Elevation >
//!   Structures only on flat terrain`, for the kinds with `ObjectDef::needs_flat_ground`, every structure so far);
//!   Visible to the placing player, in a game only (`> Direct Placement`; Extraction Plates too, design change
//!   0095; a Warp In's too, 0119), and if it isn't, that is the tile's only problem, so the check tells nothing the
//!   fog hides; covered by
//!   no other footprint, a structure's (`> Direct Placement`: "no existing structure overlap") or a map resource's
//!   (`> All placement methods: no buildings on resources`, design change 0096), but for a plate on a patch:
//!   [`ObjectKind::may_overlap`], the one rule of shared tiles, which map validation and debug spawns apply too;
//!   overlapped by no ground unit's box, whoever owns it (design change 0094: units are never pushed aside, and air
//!   and underground units don't count);
//! - the footprint: a structure's; for a GDO building, at least one tile in the build area
//!   (`factions/gdo/structures.md > GDOBuildArea`), except for the Deployment Center, which seeds the area
//!   ([`seeds_build_area`]); for an Extraction Plate, exactly onto a Space Crystals Patch
//!   (`factions/gdo/economy.md > ExtractionFacility > AwaitingPlacement`), which must have no plate yet: a covered
//!   patch is a tile under a structure. That rule is [`Onto`], which map validation also applies to the map's
//!   pre-placed objects, the one placement rule they follow besides the shared tiles.
//!
//! **The build area** ([`BuildArea`], [`Sim::build_area`]): the union of the parts each of the player's GDO
//! buildings adds, its footprint grown by its BuildRadiusExtension, square (design change 0091). It is derived from
//! the buildings standing now whenever it is asked for, never stored or hashed, so it follows every placement and
//! every loss at once (design change 0092) and can't go stale. It is the area only, apart from the per-tile validity
//! (design change 0093). A Starting Layout's area is its Deployment Center's ([`BuildArea::of_layout`]).
//!
//! **Starting Layouts** (`core/map_editor.md > Starting Layouts > Layout validation`, design change 0097) follow the
//! same rules in [`Mode::Layout`]: no visibility, and the area of the layout's own Deployment Center. A layout object
//! is checked against the map's objects and the rest of its layout. The rule for starting units ("can't stand on
//! structures or impassable tiles") is not a placement rule: it is the unit-start rule of `paths::misplaced`, which
//! scenario validation applies to the units a game starts with; the mirror rule, no footprint over a ground unit, is
//! the check's [`TileProblem::Unit`] in both modes.

mod build_area;
mod check;
mod footprint;
mod onto;
mod problems;
mod site;
#[cfg(test)]
mod tests;
mod tunnel_area;

pub use build_area::{BuildArea, seeds_build_area};
pub use check::{check, check_layout, check_reserving, footprint_size};
pub use onto::Onto;
pub use problems::{PlacementCheck, PlacementProblem, TileProblem, WholeProblem};
pub use site::{Method, Mode, Occupant, Site};

use crate::defs::ObjectKind;
use crate::deploy;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Placement;
use crate::scenario::BuildingOrientation;
use crate::sim::Sim;
use crate::transform::warp_in_footprint;
use crate::units::TilePos;

impl Sim {
    /// `core/entities.md > Placement Validation > Direct Placement`: everything that keeps `player` from placing a
    /// structure of `kind` now, with the top-left tile of its footprint at `origin`, turned by `orientation`
    /// ([`check`] in [`Mode::Command`]). It looks at the map, at every object as it is now (those placed or removed
    /// earlier in the tick included), at the player's sight as the latest tick left it ([`Sim::visibility`]) and at
    /// their build area ([`Sim::build_area`]). A player who isn't in the game sees nothing and owns nothing. It
    /// changes nothing, so the client's ghost can ask it every frame, and the command gate asks it again when the
    /// command applies.
    pub fn placement_check(
        &self,
        player: PlayerId,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> PlacementCheck {
        let area = self.build_area(player);
        let site = Site {
            map: self.map(),
            area: &area,
            mode: Mode::Command {
                vision: self.vision(),
                player,
            },
            method: Method::Direct,
        };
        check(&site, self.objects().map(Occupant::of), kind, origin, orientation)
    }

    /// `core/entities.md > Placement Validation > Warp In`: everything that keeps Beacon `beacon` from warping in a
    /// structure of `kind` now, turned by `orientation`, on the footprint that follows from them
    /// ([`crate::transform::warp_in_footprint`], design change 0121), by [`check`] with [`Method::WarpIn`] in
    /// [`Mode::Command`] (slice M3-E): every object as it is now but the Beacon, whose tiles the building takes over,
    /// and the sight of the Beacon's owner as the latest tick left it (design change 0119: "every footprint tile must
    /// be Visible to the player"), with no build area. The sites of the owner's pending Deploys are reserved
    /// ([`check_reserving`], design change 0125). The command gate asks it when a `ConstructionCommand::WarpIn`
    /// applies, and the Warp In ghost can ask it every frame (slice M3-J1). Whether the Beacon may warp in a `kind` at
    /// all (its Constructs List, the bank) is the gate's. `None` if `beacon` is no object of a player's with a
    /// footprint.
    pub fn warp_in_check(
        &self,
        beacon: ObjectId,
        kind: ObjectKind,
        orientation: BuildingOrientation,
    ) -> Option<PlacementCheck> {
        let o = self.object(beacon)?;
        let (player, placement) = (o.owner?, o.placement?);
        // A kind without a footprint is no structure, wherever the check looks.
        let origin = warp_in_footprint(&placement, kind, orientation).map_or(placement.origin, |p| p.origin);
        let area = BuildArea::default();
        let site = Site {
            map: self.map(),
            area: &area,
            mode: Mode::Command {
                vision: self.vision(),
                player,
            },
            method: Method::WarpIn { beacon: placement },
        };
        let others = self.objects().filter(|o| o.id != beacon).map(Occupant::of);
        let reserved = self.reserved_sites(player, None);
        Some(check_reserving(&site, others, &reserved, kind, origin, orientation))
    }

    /// `core/entities.md > Placement Validation > Worker-Built Structures` for a Deploy (slice M3-F, `crate::deploy`):
    /// everything that keeps the Drone `drone` from being ordered to deploy into a Beacon whose footprint's top-left
    /// tile is `origin`, as far as its owner knows ([`check_reserving`] with [`Method::WorkerBuilt`] in
    /// [`Mode::Known`]; design change 0120: "it is green if, as far as the player knows, nothing blocks the
    /// footprint"). The occupants are the owner's objects, the others it sees, and its records of the structures and
    /// resources out of its sight (`Sim::known_occupants`), the Drone left out (design change 0118), on the owner's
    /// sight as the latest tick left it. The sites of the owner's pending Deploys are reserved (design change 0125),
    /// those of `drone`'s own orders but for a Deploy `queued` with Shift: without, the new order replaces them. The
    /// command gate asks it when the order applies, and the Deploy ghost can ask it every frame (slice M3-J1). `None` if
    /// `drone` is no unit of a player's that deploys.
    pub fn deploy_check(&self, drone: ObjectId, origin: TilePos, queued: bool) -> Option<PlacementCheck> {
        let o = self.object(drone).filter(|o| deploy::deploys(o.kind))?;
        let player = o.owner?;
        let area = BuildArea::default();
        let site = Site {
            map: self.map(),
            area: &area,
            mode: Mode::Known {
                vision: self.vision(),
                player,
            },
            method: Method::WorkerBuilt,
        };
        let others = self.known_occupants(player).filter(|&(id, _)| id != Some(drone));
        let reserved = self.reserved_sites(player, (!queued).then_some(drone));
        let beacon = deploy::BEACON_DEPLOYMENT.kind;
        Some(check_reserving(
            &site,
            others.map(|(_, occupant)| occupant),
            &reserved,
            beacon,
            origin,
            BuildingOrientation::default(),
        ))
    }

    /// What `player` knows to stand on the map ([`Mode::Known`], design change 0120), each with the id of its object:
    /// its own objects and the others it sees now (a tile they cover is Visible), as they are; and its records of the
    /// structures and resources out of its sight, as it last saw them (`core/vision.md > Explored`). The units of
    /// others out of its sight are left out ("units hidden in fog are ignored").
    fn known_occupants(&self, player: PlayerId) -> impl Iterator<Item = (Option<ObjectId>, Occupant)> + '_ {
        let vision = self.vision();
        let seen = self
            .objects()
            .filter(move |o| o.owner == Some(player) || vision.sees(player, o))
            .map(|o| (Some(o.id), Occupant::of(o)));
        let remembered = vision
            .player(player)
            .into_iter()
            .flat_map(|sight| sight.remembered())
            .map(|record| {
                let occupant = Occupant {
                    kind: record.kind,
                    pos: record.pos(),
                    placement: Some(record.placement),
                };
                (None, occupant)
            });
        seen.chain(remembered)
    }

    /// The footprints `player`'s pending Deploys reserve ([`Sim::pending_deploys`], design change 0125), but those of
    /// `except`'s orders.
    fn reserved_sites(&self, player: PlayerId, except: Option<ObjectId>) -> Vec<Placement> {
        self.pending_deploys(player)
            .into_iter()
            .filter(|&(drone, _)| Some(drone) != except)
            .map(|(_, site)| site)
            .collect()
    }

    /// `factions/gdo/structures.md > GDOBuildArea`: `player`'s build area now, from the GDO buildings they own
    /// ([`BuildArea`]). Worked out from the objects at each call (each building is one rectangle), so it is always
    /// that of the buildings standing now. Empty for a player who owns none.
    pub fn build_area(&self, player: PlayerId) -> BuildArea {
        let own = self
            .objects()
            .filter(|o| o.owner == Some(player))
            .filter_map(|o| Some((o.kind, o.placement.as_ref()?)));
        BuildArea::of(self.map(), own)
    }
}
