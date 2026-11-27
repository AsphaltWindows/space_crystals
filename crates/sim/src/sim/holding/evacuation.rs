//! `factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction` (slice M5-F1; design change 0349,
//! slice M5-F3): "Applies to every Cults building that can hold units inside [...]. When the building
//! is destroyed: **Everyone inside is evacuated alive**"; "**Units come out unchanged**"; "the building is simply gone
//! [...], and evacuated units **all appear at the same time inside its former footprint**, unharmed, spread over it.
//! Units that overlap ignore each other until separated, and idle ones push apart"; "**A unit that had finished
//! training** and was waiting for the doorway comes out **as the trained unit**, with no refund"; "**Afterwards**
//! evacuated units **idle**".
//!
//! It is part of the one way objects leave the game (`Sim::remove_objects`): the units inside a building that goes are
//! read before anything else of the removal, and its finished entry taken off its queue ([`Sim::evacuations`]), then
//! put on the map once the building has gone ([`Sim::evacuate`]), in the same tick, so before the defeat check judges
//! their owner (`sim/phase.rs`). The grids catch up with the footprint that went only after the removal
//! (`paths::sync`), so a unit's spot is checked against the terrain alone, which is all that lies under a footprint.
//!
//! - **Where** ([`Sim::footprint_spot`]): the footprint's tiles in a fixed order, row by row from its north-west tile,
//!   west to east (`Placement::tiles`), each unit on the centre of the next tile, round again when there are more
//!   units than tiles: of ten Recruits in an Armory (3 × 2 tiles), two on each of the first four tiles and one on each
//!   of the last two. A tile whose terrain the unit can't stand on (`nav::box_on_terrain`) is passed over, so no unit
//!   lands on terrain it can't stand on, and none starts outside the footprint. **Placeholder (not in the design: how
//!   the units spread over the footprint)**: design change 0349 has "spread over it", and "No placement order".
//! - **Overlaps** are the movement's (`systems/units.md > Ground Collision > Units that already overlap`): units that
//!   overlap as a unit's turn begins don't block each other (`movement::space`), and each idle one steps off another
//!   until they are apart (`movement::separate`), from the first movement phase after the removal on.
//! - **As what**: as they are, the units an entry reserved too, as Recruits (the refunds of `Sim::remove_objects`
//!   come between: "Orders are cancelled with a full refund"; "a Recruit mid-training comes out as a Recruit"; "a unit
//!   mid-augmentation does not receive the Augmentation"); the unit a finished training was letting out as its new
//!   type ([`Sim::evacuate`], `Sim::train`), and the one a finished augmentation order was letting out augmented
//!   (slice M5-N1b, `Sim::augment`): **Placeholder (not in the design: an augmentation finished and waiting for the
//!   doorway as its building goes)**, read as a training is ("its training was complete"), unrefunded. Each stands
//!   where it is put, facing as it did, idle, with a `SimEvent::Evacuated`.

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::events::SimEvent;
use crate::holding;
use crate::ids::ObjectId;
use crate::math;
use crate::nav;
use crate::object::Placement;
use crate::paths;
use crate::production::QueueEntry;
use crate::shape::Shape;
use crate::sim::Sim;
use crate::units::{SU_PER_GU, TilePos};

/// The units of one building that is about to leave the game ([`Sim::evacuations`]), as it is before it goes.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sim) struct Evacuation {
    building: ObjectId,
    placement: Placement,
    /// Its units, longest inside first (`Sim::held_in`), in the order they take the footprint's tiles, but for those
    /// its finished entry consumes ([`Finished::consumed`]).
    units: Vec<ObjectId>,
    /// Its front entry, if that was finished.
    finished: Option<Finished>,
}

/// The entry a building's front had finished as the building went (design change 0349 point 3: "comes out as the
/// trained unit, with no refund"): its first reserved unit, waiting for the doorway (`Sim::ready_to_leave`), comes out
/// as the entry makes it (`Sim::complete`): a training's as the unit it trains, the others it reserved going into it
/// (`Sim::train`), an augmentation order's augmented (slice M5-N1b), a Release's as it is.
#[derive(Clone, Debug, PartialEq)]
struct Finished {
    unit: ObjectId,
    entry: QueueEntry,
    consumed: Vec<ObjectId>,
}

impl Sim {
    /// The units inside the buildings among `gone` that hold units, as the buildings are before they go, in id order
    /// of the buildings, each building's longest inside first: the units its queue's entries reserved and the ones
    /// ejected and not out yet among them (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on
    /// Destruction`: "units in the internal pool, units reserved by queued orders, the unit currently being trained").
    /// A pool producer's front entry, if finished, leaves its queue here, unrefunded ([`Sim::finished_entry`]);
    /// `Sim::remove_objects` refunds the others after this, in full, and their units come out as they went in.
    pub(in crate::sim) fn evacuations(&mut self, gone: &[ObjectId]) -> Vec<Evacuation> {
        if self.held.is_empty() {
            return Vec::new();
        }
        let mut evacuations = Vec::new();
        for &id in gone {
            let Some(placement) = (self.objects.get(&id))
                .filter(|building| holding::holds(building).is_some())
                .and_then(|building| building.placement)
            else {
                continue;
            };
            let finished = self.finished_entry(id);
            let consumed = finished.as_ref().map_or(&[][..], |finished| &finished.consumed[..]);
            let units: Vec<ObjectId> = (self.held_in(id).into_iter())
                .map(|held| held.object.id)
                .filter(|unit| !consumed.contains(unit))
                .collect();
            if !units.is_empty() {
                evacuations.push(Evacuation {
                    building: id,
                    placement,
                    units,
                    finished,
                });
            }
        }
        evacuations
    }

    /// Pool producer `building`'s front entry, if it is finished, its unit waiting for the doorway (the producers'
    /// step, `sim/producers/pool.rs`), leaves its queue without a refund: "its training was complete" (design change
    /// 0349 point 3), and so was an augmentation order (module docs); a Release's unit comes out as it is, as it was
    /// leaving. `None` for an entry not finished (`Sim::refund_production` refunds it), and for any other building.
    fn finished_entry(&mut self, building: ObjectId) -> Option<Finished> {
        let o = self.objects.get_mut(&building)?;
        let def = o.def().structure.as_ref()?.production.as_ref()?;
        let production = o.production.as_mut()?;
        if !production.status(def, None)?.waiting {
            return None;
        }
        let (entry, reserved) = production.release()?;
        let (&unit, consumed) = reserved.split_first()?;
        Some(Finished {
            unit,
            entry,
            consumed: consumed.to_vec(),
        })
    }

    /// The units of `evacuations` come out inside their buildings' former footprints, which have left the game,
    /// building by building, all in this call (module docs): each at the first tile from the next one in the
    /// footprint's order where it stands as what it comes out as ([`Sim::footprint_spot`]), facing as it did, idle, a
    /// finished training's unit trained into its new type there (`Sim::train`, a `SimEvent::KindChanged`), a finished
    /// augmentation order's augmented (`Sim::augment`), each with a `SimEvent::Evacuated`. **Placeholder (not in the design: a footprint none of whose tiles a unit can stand on,
    /// whose terrain was buildable but isn't passable for it)**: the unit leaves the game (`Sim::remove_held`), with
    /// the units it would have consumed.
    pub(in crate::sim) fn evacuate(&mut self, evacuations: &[Evacuation]) {
        for evacuation in evacuations {
            let building = evacuation.building;
            let tiles: Vec<TilePos> = evacuation.placement.tiles().collect();
            let mut next = 0;
            for &unit in &evacuation.units {
                let Some(held) = self.held.get(&unit) else {
                    continue;
                };
                let finished = evacuation.finished.as_ref().filter(|finished| finished.unit == unit);
                let kind = finished
                    .and_then(|finished| finished.entry.unit())
                    .unwrap_or(held.object.kind);
                let facing = math::facing_vector(held.object.facing);
                let spot = (0..tiles.len()).map(|i| (next + i) % tiles.len()).find_map(|i| {
                    let spot = self.footprint_spot(kind, facing, &evacuation.placement, tiles[i])?;
                    Some((i, spot))
                });
                let Some((tile, pos)) = spot else {
                    let consumed = finished.map_or(&[][..], |finished| &finished.consumed[..]);
                    self.remove_held(&[&[unit][..], consumed].concat());
                    continue;
                };
                next = tile + 1;
                if !self.put_on_map(unit, pos, facing) {
                    continue;
                }
                if let Some(finished) = finished {
                    self.complete(unit, finished.entry, &finished.consumed);
                }
                self.events.push(SimEvent::Evacuated { unit, building });
            }
        }
    }

    /// Where a unit of `kind` facing `facing` stands on `tile` of the footprint `placement` (module docs): the tile's
    /// centre, moved in as far as it takes for the unit's box to lie inside the footprint (along an axis on which the
    /// box is longer than the footprint, at the footprint's middle; no type the design's buildings hold is), if the
    /// terrain under the box is all ground the unit's grid lets it stand on (`nav::box_on_terrain`). The box is
    /// (M5-N6) the bounding box of its Silhouette as it faces (`Shape::bound_half`), a Seeker's turned with it. `None`
    /// if it isn't, or `kind` is no unit.
    fn footprint_spot(&self, kind: ObjectKind, facing: Vec2, placement: &Placement, tile: TilePos) -> Option<Vec2> {
        let def = kind.def();
        let (unit, silhouette) = (def.unit.as_ref()?, def.silhouette()?);
        let half = Shape::silhouette(kind, Vec2::ZERO, math::vector_angle(facing)).bound_half();
        let size = Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU;
        let (lo, hi) = (
            placement.origin.corner() + half,
            placement.origin.corner() + size - half,
        );
        let (centre, middle) = (tile.center(), placement.center());
        let inside = |axis: usize| match lo[axis] <= hi[axis] {
            true => centre[axis].max(lo[axis]).min(hi[axis]),
            false => middle[axis],
        };
        let spot = Vec2::new(inside(0), inside(1));
        // Its grid by its size class, which its Silhouette gives, however it is turned.
        let grid = paths::unit_grid(self.nav.nav(), unit.base_def(), silhouette);
        grid.is_none_or(|grid| nav::box_on_terrain(grid, half, spot))
            .then_some(spot)
    }
}
