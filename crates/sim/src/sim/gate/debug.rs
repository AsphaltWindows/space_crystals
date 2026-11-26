//! The gate's checks of debug commands (`Command::Debug`, accepted while `SimConfig::debug` is on): a spawn's owner
//! and placement, a teleport onto the map, a unit's box clear of blocked ground and of the units of its layer
//! (`docs/movement.md > 3`), a tier the Tunnel's table has, and augmentations for a unit (slice M5-F2).

use glam::Vec2;

use super::Checked;
use crate::augmentation;
use crate::command::DebugCommand;
use crate::defs::{Domain, ObjectKind};
use crate::ids::ObjectId;
use crate::nav;
use crate::object::{Object, Placement};
use crate::paths::{blocking, blocking_footprint, box_overlaps_rect, boxes_overlap};
use crate::player::Banks;
use crate::scenario::place_spawn;
use crate::sim::{Reject, Sim};
use crate::tunnel;

impl Sim {
    pub(super) fn check_debug(&self, command: &DebugCommand) -> Result<Checked, Reject> {
        // Debug commands act on anyone's objects.
        let anyone = None;
        match *command {
            DebugCommand::Spawn {
                kind,
                owner,
                at,
                orientation,
            } => {
                if self.next_id == u32::MAX {
                    return Err(Reject::OutOfIds);
                }
                let is_player = |p| self.player(p).is_some();
                let (pos, placement) =
                    place_spawn(&self.map, kind, owner, at, orientation, is_player).map_err(Reject::Spawn)?;
                // A defeated player stays defeated and owns nothing (`core/game_rules.md > On Defeat`): no object
                // of theirs, and no eyes for their vision, comes back.
                if let Some(owner) = owner
                    && self.player(owner).is_some_and(|p| p.is_defeated())
                {
                    return Err(Reject::Defeated(owner));
                }
                match &placement {
                    Some(placement) => {
                        self.check_footprint_apart(kind, placement)?;
                        self.check_footprint_clear(kind, placement)?;
                    }
                    None => self.check_unit_fits(kind, pos, None)?,
                }
                Ok(Checked::Spawn {
                    kind,
                    owner,
                    pos,
                    placement,
                })
            }
            DebugCommand::Remove { id } => {
                self.referenced(id, anyone)?;
                Ok(Checked::Remove(id))
            }
            DebugCommand::SetPosition { id, pos } => {
                let object = self.referenced(id, anyone)?;
                if object.placement.is_some() {
                    return Err(Reject::TileObject(id));
                }
                // `contains_point` also rejects NaN and infinite positions.
                if !self.map.contains_point(pos) {
                    return Err(Reject::OffMap);
                }
                self.check_unit_fits(object.kind, pos, Some(id))?;
                Ok(Checked::Teleport { id, pos })
            }
            DebugCommand::SetAutoTargeting { on } => Ok(Checked::AutoTargeting(on)),
            DebugCommand::SetResources {
                player,
                crystals,
                supplies,
                alloys,
                essence,
                flux,
            } => {
                if self.player(player).is_none() {
                    return Err(Reject::UnknownPlayer(player));
                }
                let banks = Banks {
                    crystals,
                    supplies,
                    alloys,
                    essence,
                    flux,
                };
                Ok(Checked::SetResources { player, banks })
            }
            DebugCommand::SetTier { id, tier } => {
                let object = self.referenced(id, anyone)?;
                if tunnel::def_of(object.kind).and_then(|def| def.tier(tier)).is_none() {
                    return Err(Reject::NoSuchTier { id, tier });
                }
                Ok(Checked::SetTier { id, tier })
            }
            DebugCommand::SetAugmentation { id, augmentation } => {
                let object = self.referenced(id, anyone)?;
                if object.def().unit.is_none() {
                    return Err(Reject::TileObject(id));
                }
                if !augmentation.is_none() && !augmentation::is_augmentable(object.kind) {
                    return Err(Reject::NotAugmentable(id));
                }
                Ok(Checked::SetAugmentation { id, augmentation })
            }
        }
    }

    /// `docs/movement.md > 3`: a unit of `kind` placed at `pos` keeps the invariants: its box is clear of blocked
    /// tiles and blocking footprints and inside the map, and overlaps no other unit of its layer (`moving` is the
    /// unit itself, when it is teleported). Other kinds pass.
    pub(in crate::sim) fn check_unit_fits(
        &self,
        kind: ObjectKind,
        pos: Vec2,
        moving: Option<ObjectId>,
    ) -> Result<(), Reject> {
        let def = kind.def();
        let (Some(unit), Some(silhouette)) = (&def.unit, def.silhouette()) else {
            return Ok(());
        };
        let half = silhouette / 2.0;
        let base = unit.base_def();
        if let Some(class) = base.nav_class() {
            let grid = self.nav.nav().grid(class, nav::size_class(silhouette));
            if grid.is_some_and(|grid| !nav::box_clear(grid, half, pos)) {
                return Err(Reject::Blocked);
            }
        }
        // The grids catch up with this tick's spawns and removals only in phase 5b: footprints are checked
        // directly too, so a unit can't be put on a station spawned earlier in the tick.
        if base.domain == Domain::Ground
            && self
                .objects
                .values()
                .any(|o| blocking(o, self.nav.nav().rules()).is_some_and(|rect| box_overlaps_rect(pos, half, rect)))
        {
            return Err(Reject::Blocked);
        }
        if base.domain.hard_collision()
            && let Some(other) = self.objects.values().find(|o| {
                Some(o.id) != moving
                    && in_layer(o, base.domain).is_some_and(|other| boxes_overlap(pos, half, o.pos, other))
            })
        {
            return Err(Reject::Occupied(other.id));
        }
        Ok(())
    }

    /// A blocking footprint must not cover a ground unit (`docs/movement.md > 3`).
    /// `core/entities.md > Placement Validation`: a footprint of `kind` at `placement` overlaps no other footprint,
    /// a structure's or a map resource's (design change 0096: no buildings on resources), except that a plate goes
    /// onto an uncovered patch ([`ObjectKind::may_overlap`]). Else the first object in the way, by id.
    fn check_footprint_apart(&self, kind: ObjectKind, placement: &Placement) -> Result<(), Reject> {
        let in_the_way = self
            .objects
            .values()
            .find(|o| !kind.may_overlap(o.kind) && o.placement.as_ref().is_some_and(|other| other.overlaps(placement)));
        match in_the_way {
            Some(other) => Err(Reject::Overlaps(other.id)),
            None => Ok(()),
        }
    }

    fn check_footprint_clear(&self, kind: ObjectKind, placement: &Placement) -> Result<(), Reject> {
        let Some(rect) = blocking_footprint(kind, placement, self.nav.nav().rules()) else {
            return Ok(());
        };
        match self
            .objects
            .values()
            .find(|o| in_layer(o, Domain::Ground).is_some_and(|half| box_overlaps_rect(o.pos, half, rect)))
        {
            Some(unit) => Err(Reject::Occupied(unit.id)),
            None => Ok(()),
        }
    }
}

/// The half extents of `o`'s box if it is a unit of `domain`'s layer (`docs/movement.md > 6.1`).
fn in_layer(o: &Object, domain: Domain) -> Option<Vec2> {
    let def = o.def();
    let unit = def.unit.as_ref()?;
    (unit.base_def().domain == domain).then(|| def.half_extents()).flatten()
}
