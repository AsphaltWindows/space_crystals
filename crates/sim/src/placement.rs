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
//! **The rules**, in the order the problems are listed:
//! - each footprint tile: on the map; Buildable (`core/map.md > TilePreset`); flat (`core/map.md > Elevation >
//!   Structures only on flat terrain`, for the kinds with `ObjectDef::needs_flat_ground`, every structure so far);
//!   Visible to the placing player, in a game only (`> Direct Placement`; Extraction Plates too, design change
//!   0095), and if it isn't, that is the tile's only problem, so the check tells nothing the fog hides; covered by
//!   no other footprint, a structure's (`> Direct Placement`: "no existing structure overlap") or a map resource's
//!   (`> All placement methods: no buildings on resources`, design change 0096), but for a plate on a patch:
//!   [`ObjectKind::may_overlap`], the one rule of shared tiles, which map validation and debug spawns apply too;
//!   overlapped by no ground unit's box, whoever owns it (design change 0094: units are never pushed aside, and air
//!   and underground units don't count);
//! - the footprint: a structure's; for a GDO building, at least one tile in the build area
//!   (`factions/gdo/structures.md > GDOBuildArea`), except for the Deployment Center, which seeds the area
//!   ([`seeds_build_area`]); for an Extraction Plate, exactly onto a Space Crystals Patch
//!   (`factions/gdo/economy.md > ExtractionFacility > AwaitingPlacement`), which must have no plate yet: a covered
//!   patch is a tile under a structure.
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
#[cfg(test)]
mod tests;

pub use build_area::{BuildArea, seeds_build_area};
pub use check::{Mode, Occupant, PlacementCheck, Site, TileProblem, WholeProblem, check, check_layout, footprint_size};

use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::scenario::BuildingOrientation;
use crate::sim::Sim;
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
        };
        check(&site, self.objects().map(Occupant::of), kind, origin, orientation)
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
