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
//! asks [`Sim::warp_in_check`]. A worker-built structure ([`Method::WorkerBuilt`]: a Drone deploying a Beacon, slice
//! M3-F, `crate::deploy`; an Agent building a Tunnel, slice M4-E, `crate::construction::site`; `core/entities.md >
//! Worker-Built Structures`) has the same tile rules and no build area; its order and its ghost are checked on what
//! the player knows ([`Mode::Known`], design change 0120: never-explored tiles pass, units hidden in the fog don't
//! count, and the structures out of sight are as last seen), a game asking [`Sim::deploy_check`] or
//! [`Sim::build_check`], and the worker checks its site again on arrival, on the truth ([`Mode::Arrival`]). An
//! underground expansion ([`Method::Expansion`], slice M4-F; `factions/syndicate/tunnels.md > AwaitingPlacement
//! (Expansion)`) is placed by its Tunnel inside the Tunnel's current area, on Drillable ground (design change 0239)
//! with no flatness and no sight; any structure blocks it, the Tunnel's own and any player's surface building
//! included (0240), as do resources; ground units don't, and no lingering ghost can be in its way (design change
//! 0290). Its ghost judges by what the owner knows ([`Sim::expansion_check`], [`Mode::Known`]), and the command by the
//! truth (0240: "judged by the truth; the placement is refused"). A direct placement or a Warp In over a hidden
//! expansion is judged by the truth too, red at once ([`Mode::Command`]; 0240: "this is accepted"; a Warp In "is
//! judged the same way: red at once", 0290); only a worker-built structure's ghost judges by what the player knows.
//!
//! **Reservations** (`core/entities.md > Lingering build ghosts`, design change 0125): a player's pending worker builds
//! ([`Sim::pending_builds`]) reserve their sites against that player's further placements, a Deploy's, a Build's and
//! a Warp In's ([`check_reserving`], [`TileProblem::Reserved`]); a Tunnel's whole Tier 1 area too, against a further
//! Tunnel's (design change 0289 point 2, [`WholeProblem::TunnelAreaOverlap`]). Another player's never matter, and
//! units walk over them.
//!
//! **The rules**, in the order the problems are listed:
//! - each footprint tile: on the map; Buildable (`core/map.md > TilePreset`); flat (`core/map.md > Elevation >
//!   Structures only on flat terrain`, for the kinds with `ObjectDef::needs_flat_ground`, every structure so far);
//!   Visible to the placing player, in a game only (`core/entities.md > Direct Placement`; Extraction Plates too,
//!   design change 0095; a Warp In's too, 0119), and if it isn't, that is the tile's only problem, so the check tells
//!   nothing the fog hides; covered by
//!   no other footprint, a structure's (`core/entities.md > Direct Placement`: "no existing structure overlap") or a
//!   map resource's (`core/entities.md > All placement methods: no buildings on resources`, design change 0096), but
//!   for a plate on a patch: [`ObjectKind::may_overlap`], the one rule of shared tiles, which map validation and debug
//!   spawns apply too; overlapped by no ground unit's box, whoever owns it (design change 0094: units are never pushed
//!   aside, and air and underground units don't count), but for an expansion's (design change 0290: "ground units
//!   **don't** block a Syndicate expansion's placement");
//! - the footprint: a structure's; for a GDO building, at least one tile in the build area
//!   (`factions/gdo/structures/README.md > GDOBuildArea`), except for the Deployment Center, which seeds the area
//!   ([`seeds_build_area`]); for an Extraction Plate, exactly onto a Space Crystals Patch
//!   (`factions/gdo/economy.md > ExtractionFacility > AwaitingPlacement`), which must have no plate yet: a covered
//!   patch is a tile under a structure. That rule is [`Onto`], which map validation also applies to the map's
//!   pre-placed objects, the one placement rule they follow besides the shared tiles; for a Tunnel (any kind with a
//!   tier table), its whole Tier 1 Tunnel Area on the map and Drillable (slice M4-B; `factions/syndicate/tunnels.md >
//!   Tunnel Area Non-Overlap Rule > Drillable area`, design change 0242: [`WholeProblem::TunnelAreaOffMap`],
//!   [`WholeProblem::TunnelAreaNotDrillable`]), in every mode, [`Mode::Known`] counting only the undrillable ground
//!   the player explored (`placement/tunnel_area.rs`); and that area overlapping no other Tunnel's area at its tier,
//!   whoever owns it, a site included (slice M4-E; `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`,
//!   design change 0237: [`WholeProblem::TunnelAreaOverlap`], `placement/tunnel_overlap.rs`), [`Mode::Known`] judging
//!   other players' Tunnels by what the player knows of them. Map and debug spawns don't take the check, so they are
//!   exempt.
//!
//! **The files**: [`Site`], [`Mode`], [`Method`] and [`Occupant`] in `site.rs`; the problems in `problems.rs`; the
//! check in `check.rs`, its footprint's tiles in `footprint.rs`; the Tunnel Area's rules in `tunnel_area.rs` and
//! `tunnel_overlap.rs`, on the geometry of `crate::tunnel::area`; the build area in `build_area.rs` and [`Onto`] in
//! `onto.rs`.
//!
//! **The build area** ([`BuildArea`], [`Sim::build_area`]): the union of the parts each of the player's GDO
//! buildings adds, its footprint grown by its BuildRadiusExtension, square (design change 0091). It is derived from
//! the buildings standing now whenever it is asked for, never stored or hashed, so it follows every placement and
//! every loss at once (design change 0092) and can't go stale. It is the area only, apart from the per-tile validity
//! (design change 0093). A Starting Layout's area is its Deployment Center's ([`BuildArea::of_layout`]).
//!
//! **Starting Layouts** (`core/map_editor.md > Starting Layouts > Layout validation`, design change 0097) follow the
//! same rules in [`Mode::Layout`]: no visibility, and the area of the layout's own Deployment Center. A layout object
//! is checked against the map's objects and the rest of its layout. A layout's underground structure, the Syndicate's
//! Hideout, follows the expansion method, wholly inside the Tier 1 Tunnel Area of a Tunnel of its layout (slices M4-B
//! and M4-F, [`WholeProblem::OutsideTunnelArea`], [`check_layout`]). The rule for starting units ("can't stand on
//! structures or impassable tiles") is not a placement rule: it is the unit-start rule of `paths::misplaced`, which
//! scenario validation applies to the units a game starts with; the mirror rule, no footprint over a ground unit, is
//! the check's [`TileProblem::Unit`] in both modes.

mod build_area;
mod check;
mod footprint;
mod onto;
mod problems;
mod site;
mod spots;
#[cfg(test)]
mod tests;
mod tunnel_area;
mod tunnel_overlap;
mod wall;

pub use build_area::{BuildArea, seeds_build_area};
pub use check::{check, check_layout, check_reserving, footprint_size};
pub use onto::Onto;
pub use problems::{PlacementCheck, PlacementProblem, TileProblem, WholeProblem};
pub use site::{Method, Mode, Occupant, Site};
pub use spots::{Ring, Round, Spots};
pub use wall::{Wall, crosses};

use crate::construction::site::builds;
use crate::defs::ObjectKind;
use crate::deploy;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
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
    /// [`Mode::Command`] (slice M3-E): every object as it is now but the Beacon, whose tiles the building takes over, a
    /// hidden enemy expansion included, as for a direct placement (design change 0290: "a Colonists **Warp In** is
    /// judged the same way: red at once"),
    /// and the sight of the Beacon's owner as the latest tick left it (design change 0119: "every footprint tile must
    /// be Visible to the player"), with no build area. The sites of the owner's pending Deploys are reserved
    /// ([`check_reserving`], design change 0125), as are those of its pending Builds. The command gate asks it when a
    /// `ConstructionCommand::WarpIn` applies, and the Warp In ghost can ask it every frame (slice M3-J1). Whether the
    /// Beacon may warp in a `kind` at all (its Constructs List, the bank) is the gate's. `None` if `beacon` is no object
    /// of a player's with a footprint.
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
        let reserved = self.reserved_sites(player, &[]);
        Some(check_reserving(&site, others, &reserved, kind, origin, orientation))
    }

    /// `core/entities.md > Placement Validation > Worker-Built Structures` for a Deploy (slice M3-F, `crate::deploy`):
    /// everything that keeps the Drone `drone` from being ordered to deploy into a Beacon whose footprint's top-left
    /// tile is `origin`, as far as its owner knows ([`Sim::worker_check`]). The command gate asks it when the order
    /// applies, and the Deploy ghost can ask it every frame (slice M3-J1). `None` if `drone` is no unit of a player's
    /// that deploys.
    pub fn deploy_check(&self, drone: ObjectId, origin: TilePos, queued: bool) -> Option<PlacementCheck> {
        let o = self.object(drone).filter(|o| deploy::deploys(o.kind))?;
        let beacon = deploy::BEACON_DEPLOYMENT.kind;
        self.worker_check(o, beacon, origin, BuildingOrientation::default(), queued)
    }

    /// `core/entities.md > Placement Validation > Worker-Built Structures` for a Build (slice M4-E, plan decision T51):
    /// everything that keeps `worker` from being ordered to build a structure of `kind` with the top-left tile of its
    /// footprint at `origin`, turned by `orientation`, as far as its owner knows ([`Sim::worker_check`]): for a
    /// Tunnel, its Tier 1 area on the map and Drillable where the owner explored (design change 0242), clear of every
    /// Tunnel's area as the owner knows it, at its tier as last seen (0237: "The ghost judges enemy Tunnels by what the
    /// player knows"), and clear of the areas of the owner's other lingering Tunnel ghosts (design change 0289 point
    /// 2). The command gate asks it when the order applies, and the ghost can ask it every frame. `None` if `worker` is
    /// no unit of a player's that builds `kind` ([`builds`]).
    pub fn build_check(
        &self,
        worker: ObjectId,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
        queued: bool,
    ) -> Option<PlacementCheck> {
        let o = self.object(worker).filter(|o| builds(o.kind, kind))?;
        self.worker_check(o, kind, origin, orientation, queued)
    }

    /// (M5-E) [`Sim::build_check`] for a group's Build (plan decision T59, `sim/builds/shared.rs`): everything that
    /// keeps `workers`, workers of one player's that build `kind` together, from being ordered to build it with the
    /// top-left tile of its footprint at `origin`, turned by `orientation`, as far as their owner knows. Every one of
    /// them is left out of the occupants, as one worker is (design change 0118: "the building worker itself ...
    /// doesn't count"): they build from outside, and walk off the footprint before the site starts. Without Shift
    /// (`queued`), the ghosts their orders alone hold reserve nothing: the new order replaces them. `None` if `workers`
    /// is empty, or holds an object that is no unit of that player's that builds `kind`.
    pub fn group_build_check(
        &self,
        workers: &[ObjectId],
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
        queued: bool,
    ) -> Option<PlacementCheck> {
        let player = self.object(*workers.first()?)?.owner?;
        let builder = |id: &ObjectId| {
            self.object(*id)
                .is_some_and(|o| o.owner == Some(player) && builds(o.kind, kind))
        };
        if !workers.iter().all(builder) {
            return None;
        }
        self.workers_check(player, workers, (kind, origin, orientation), queued)
    }

    /// The check of a worker build's order, worker `o` building a `kind` at `origin` turned by `orientation`, as far as
    /// its owner knows ([`check_reserving`] with [`Method::WorkerBuilt`] in [`Mode::Known`]; design change 0120: "it
    /// is green if, as far as the player knows, nothing blocks the footprint"). The occupants are the owner's
    /// objects, the others it sees, and its records of the structures and resources out of its sight
    /// (`Sim::known_occupants`), the worker left out (design change 0118), on the owner's sight as the latest tick
    /// left it. The sites of the owner's pending worker builds are reserved (design change 0125), those of `o`'s own
    /// orders but for an order `queued` with Shift: without, the new order replaces them. `None` for an object
    /// without an owner.
    fn worker_check(
        &self,
        o: &Object,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
        queued: bool,
    ) -> Option<PlacementCheck> {
        self.workers_check(o.owner?, &[o.id], (kind, origin, orientation), queued)
    }

    /// [`Sim::worker_check`] for the workers `workers` of `player`'s, one or a group (slice M5-E), all left out of the
    /// occupants; the ghosts their orders alone hold reserve nothing unless the order is `queued`.
    fn workers_check(
        &self,
        player: PlayerId,
        workers: &[ObjectId],
        (kind, origin, orientation): (ObjectKind, TilePos, BuildingOrientation),
        queued: bool,
    ) -> Option<PlacementCheck> {
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
        let others = (self.known_occupants(player)).filter(|&(id, _)| !id.is_some_and(|id| workers.contains(&id)));
        let reserved = self.reserved_sites(player, if queued { &[] } else { workers });
        Some(check_reserving(
            &site,
            others.map(|(_, occupant)| occupant),
            &reserved,
            kind,
            origin,
            orientation,
        ))
    }

    /// `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)` (slice M4-F): everything that keeps Tunnel
    /// `tunnel` from placing an expansion of `kind` with the top-left tile of its footprint at `origin`, turned by
    /// `orientation`, as far as its owner knows: the ghost's check ([`check`] by [`Method::Expansion`] in the Tunnel's
    /// current area, in [`Mode::Known`]: "The ghost judges by what the player knows", design change 0120), on the
    /// objects its owner knows of (`Sim::known_occupants`), none of its owner's pending worker builds reserved: none
    /// can be in the way (design change 0290: "Lingering Tunnel ghosts can't be in the way", "No ghost check is
    /// needed"). The command places it by the truth ([`Sim::expansion_truth`]): a building the
    /// owner doesn't know of leaves the ghost green and refuses the placement (design change 0240). Whether the Tunnel
    /// may build a `kind` now at all (its operation, the tier, the bank) is the gate's (`Sim::can_expand`). `None` if
    /// `tunnel` is no placed Tunnel of a player's.
    pub fn expansion_check(
        &self,
        tunnel: ObjectId,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> Option<PlacementCheck> {
        self.expansion_site_check(self.object(tunnel)?, kind, origin, orientation, true)
    }

    /// [`Sim::expansion_check`] by the truth, the command's (design change 0240: "judged by the truth; the placement is
    /// refused"): every object as it is now, in [`Mode::Command`], whose sight rule the expansion method doesn't have.
    pub(crate) fn expansion_truth(
        &self,
        tunnel: &Object,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
    ) -> Option<PlacementCheck> {
        self.expansion_site_check(tunnel, kind, origin, orientation, false)
    }

    /// The expansion's check, as far as the Tunnel's owner knows (`known`) or by the truth.
    fn expansion_site_check(
        &self,
        tunnel: &Object,
        kind: ObjectKind,
        origin: TilePos,
        orientation: BuildingOrientation,
        known: bool,
    ) -> Option<PlacementCheck> {
        let (player, area) = (tunnel.owner?, crate::tunnel::area::of(tunnel)?);
        let build_area = BuildArea::default();
        let vision = self.vision();
        let site = Site {
            map: self.map(),
            area: &build_area,
            mode: match known {
                true => Mode::Known { vision, player },
                false => Mode::Command { vision, player },
            },
            method: Method::Expansion { area },
        };
        // (M4-S2) The owner's pending worker builds reserve their sites (design change 0125). Design change 0290 found
        // "No ghost check is needed", since no Tunnel ghost lies inside another Tunnel's area; an Agent's Laser Turret
        // ghost may (slice M4-S1), so the reservation applies here as to every other placement of the player's.
        let reserved = self.reserved_sites(player, &[]);
        Some(match known {
            true => check_reserving(
                &site,
                self.known_occupants(player).map(|(_, o)| o),
                &reserved,
                kind,
                origin,
                orientation,
            ),
            false => check_reserving(
                &site,
                self.objects().map(Occupant::of),
                &reserved,
                kind,
                origin,
                orientation,
            ),
        })
    }

    /// What `player` knows to stand on the map ([`Mode::Known`], design change 0120), each with the id of its object:
    /// its own objects and the others it sees now (a tile they cover is Visible), as they are, its own Tunnels' upgrades
    /// with the areas they reserve, the others' Tunnels at their tier (`Occupant::as_seen`, slice M4-G); and its records
    /// of the structures and resources out of its sight, as it last saw them (`core/vision.md > Explored`). The units of
    /// others out of its sight are left out ("units hidden in fog are ignored").
    fn known_occupants(&self, player: PlayerId) -> impl Iterator<Item = (Option<ObjectId>, Occupant)> + '_ {
        let vision = self.vision();
        let seen = self
            .objects()
            .filter(move |o| o.owner == Some(player) || vision.sees(player, o))
            .map(move |o| match o.owner == Some(player) {
                true => (Some(o.id), Occupant::of(o)),
                false => (Some(o.id), Occupant::as_seen(o)),
            });
        let remembered = vision
            .player(player)
            .into_iter()
            .flat_map(|sight| sight.remembered())
            .map(|record| {
                let occupant = Occupant {
                    kind: record.kind,
                    pos: record.pos(),
                    placement: Some(record.placement),
                    tier: record.tunnel.as_ref().map(|tunnel| tunnel.tier),
                };
                (None, occupant)
            });
        seen.chain(remembered)
    }

    /// The sites `player`'s pending worker builds reserve ([`Sim::pending_builds`], design change 0125), but those
    /// that only the orders of the units `except` hold, which are about to go (`Sim::pending_without`): each the
    /// structure it will be, on its footprint, a Tunnel at the tier it starts at ([`check_reserving`]).
    fn reserved_sites(&self, player: PlayerId, except: &[ObjectId]) -> Vec<Occupant> {
        self.pending_without(player, except)
            .into_iter()
            .map(|pending| Occupant {
                kind: pending.kind,
                pos: pending.site.center(),
                placement: Some(pending.site),
                tier: None,
            })
            .collect()
    }

    /// `factions/gdo/structures/README.md > GDOBuildArea`: `player`'s build area now, from the GDO buildings they own
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
