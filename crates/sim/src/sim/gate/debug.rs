//! The gate's checks of debug commands (`Command::Debug`, accepted while `SimConfig::debug` is on): a spawn's owner
//! and placement, a teleport onto the map, a unit's box clear of blocked ground and of the units of its layer
//! (`docs/movement.md > 3`), a tier the Tunnel's table has, augmentations for a unit (slice M5-F2), and (M5-X1b) rules
//! of the game's structure, their numbers finite, with a generation for them (`sim/rules.rs`).

use glam::Vec2;

use super::Checked;
use crate::augmentation;
use crate::command::DebugCommand;
use crate::defs::{Domain, ObjectKind};
use crate::ids::ObjectId;
use crate::nav;
use crate::object::{Object, Placement};
use crate::paths::{blocking, blocking_footprint, box_overlaps_rect};
use crate::player::Banks;
use crate::ruleset;
use crate::scenario::place_spawn;
use crate::shape::Shape;
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
            DebugCommand::SetRules(ref rules) => {
                if let Some(change) = self.rules().structural_diff(rules) {
                    return Err(Reject::RulesStructural(change));
                }
                if let Some(field) = rules.non_finite() {
                    return Err(Reject::RulesNotFinite(field));
                }
                if let Some(invalid) = rules.check() {
                    return Err(Reject::RulesInvalid {
                        field: invalid.field,
                        why: invalid.why,
                    });
                }
                if !ruleset::installable(rules) {
                    return Err(Reject::RulesFull);
                }
                Ok(Checked::SetRules)
            }
        }
    }

    /// `docs/movement.md > 3`: a unit of `kind` placed at `pos` keeps the invariants: its box is clear of blocked
    /// tiles and blocking footprints and inside the map, and overlaps no other unit of its layer (`moving` is the
    /// unit itself, when it is teleported). Other kinds pass. (M5-V1) Its shape and theirs are their Silhouettes as
    /// they lie (`Shape::silhouette`: a teleported unit keeps its facing, a spawned one faces east), tested against
    /// the tiles as the movement step tests them (`Shape::clear_on`), against this tick's footprints by its bounding
    /// box.
    ///
    /// The tiles are its grid's (slice M5-Z2, `paths`, *Which grids*); for a spawn of the first unit of its class,
    /// whose grid the game builds only once it is in (`Sim::apply_command`), the grid it will build, made for the
    /// answer: of the footprints of the true grids, which the gate checks on.
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
        let facing = moving.and_then(|id| self.objects.get(&id)).map_or(0.0, |o| o.facing);
        let shape = Shape::silhouette(kind, pos, facing);
        let half = shape.bound_half();
        let base = unit.base_def();
        if let Some(class) = base.nav_class() {
            let size = nav::size_class(silhouette);
            let built;
            let grid = match self.nav.nav().grid(class, size) {
                Some(grid) => grid,
                None => {
                    debug_assert!(moving.is_none(), "a unit in the game without its grid");
                    built = self.nav.fresh_grid(&self.map, class, size);
                    &built
                }
            };
            if !shape.clear_on(grid) {
                return Err(Reject::Blocked);
            }
        } else if !unit.glides() && self.map.clamp_box(pos, half) != pos {
            // (M6-GK) No grid holds a HoverCraft's box on the map, as a ground unit's holds its own: the map does
            // (`systems/units.md > Air Collision > Map edge`: "HoverCraft stay inside the map"). A Glider may be past it.
            return Err(Reject::Blocked);
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
            && let Some(other) = self
                .objects
                .values()
                .find(|o| Some(o.id) != moving && in_layer(o, base.domain).is_some_and(|other| shape.overlaps(&other)))
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
        match self.objects.values().find(|o| {
            in_layer(o, Domain::Ground).is_some_and(|shape| box_overlaps_rect(shape.centre(), shape.bound_half(), rect))
        }) {
            Some(unit) => Err(Reject::Occupied(unit.id)),
            None => Ok(()),
        }
    }
}

/// `o`'s shape if it is a unit of `domain`'s layer (`docs/movement.md > 6.1`): its Silhouette, (M5-V1) as it lies.
fn in_layer(o: &Object, domain: Domain) -> Option<Shape> {
    let def = o.def();
    let unit = def.unit.as_ref()?;
    (unit.base_def().domain == domain && def.silhouette().is_some()).then(|| Shape::of(o))
}
