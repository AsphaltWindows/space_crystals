//! The release of a producer's finished unit (slice M2-D2; `core/entities.md > Structure Type > Exit Doorway`): into
//! its exit doorway, a portal's energy doorway (slice M3-D2) or, from underground, its owner's Tunnel Network; and the
//! spot in a side doorway where a unit appears ([`Sim::doorway_spot`]), which the buildings that hold units let theirs
//! out at too (`sim/holding.rs`).

use glam::Vec2;

use crate::defs::{ExitRule, ObjectKind, ProductionDef};
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::math;
use crate::object::Object;
use crate::paths::{self, boxes_overlap};
use crate::production::{Door, ExitDoor, QueueEntry};
use crate::shape::Shape;
use crate::sim::Sim;

/// When something stands right in front of the doorway, the released unit stands this far from it, space units,
/// so that no rounding makes the two boxes overlap (a technical margin: a sixty-fourth of a space unit).
const SPOT_CLEARANCE: f32 = 1.0 / 64.0;

/// What `o`'s type produces, if anything.
fn producer_def(o: &Object) -> Option<&'static ProductionDef> {
    o.def().structure.as_ref()?.production.as_ref()
}

/// (M5-N6) The half extents of the box of a unit of `kind` as it comes out of `door`, facing out of it
/// (`Sim::put_on_map`): its Silhouette's bounding box turned to the door's outward direction (slice M5-V1,
/// `Shape::bound_half`), the box the unit has where it appears. A Seeker out of a side facing north or south is 40 su
/// wide across the door and 56 su long out of it; a square Silhouette's is its own.
pub(in crate::sim) fn exit_half(kind: ObjectKind, door: &Door) -> Vec2 {
    Shape::silhouette(kind, Vec2::ZERO, math::vector_angle(door.outward())).bound_half()
}

impl Sim {
    /// `core/entities.md > Structure Type > Exit Doorway` (design change 0080): `producer`'s finished unit comes into
    /// the game in its exit doorway with the order its rally point gives (`factions/gdo/structures/barracks.md >
    /// RallyPoint behavior`); or it stays inside, finished, while the doorway is occupied: "the next unit can only
    /// appear once the doorway is free. While a unit still stands in it, the building pauses". The doorway is occupied
    /// while a unit this building released (`Mover::exit`) is still in it. Out of a side doorway the unit appears half
    /// inside the middle of the exit side, facing out ([`Sim::doorway_spot`]), and (M6-GF) a wheeled or tracked vehicle
    /// whose rally point is taken parks at the free spot nearest it (design change 0451, `Sim::park`); out of a
    /// portal's energy doorway, at the
    /// centre, facing the heading it leaves along (design change 0145: `production::EnergyDoorway::release`, which
    /// draws from the sim's Rng only with no rally point and two or more ways out to choose from). A building with
    /// several exit sides releases it through the first door in its order whose doorway is free and has a spot for it
    /// (`production::Doorway::order`), and holds it while none has. A Recruitment
    /// Hall's Recruit is of its lineage (`Object::lineage`, plan decision T56: "Each Recruit tracks which Recruitment
    /// Hall produced it", `factions/cults/structures/recruitment_hall.md > Unit Control Tracking`).
    pub(super) fn release_unit(&mut self, producer: ObjectId) {
        let Some(building) = self.objects.get(&producer) else {
            return;
        };
        // A producer underground releases into its owner's Tunnel Network (`sim/network.rs`).
        if producer_def(building).is_some_and(|def| def.exits == ExitRule::Network) {
            return self.release_into_network(producer);
        }
        // (M6-GH1) An Airfield's aircraft appears (M6-GH2) landed on a free slot (`ExitRule::Pad`, `sim/landing.rs`).
        if producer_def(building).is_some_and(|def| def.exits == ExitRule::Pad) {
            return self.release_onto_slot(producer);
        }
        let (Some(doorway), Some(owner)) = (ExitDoor::of(building), building.owner) else {
            return;
        };
        let Some(production) = &building.production else {
            return;
        };
        let (Some(kind), rally) = (production.current().and_then(QueueEntry::unit), production.rally) else {
            return;
        };
        let def = kind.def();
        let (Some(unit), Some(half)) = (&def.unit, def.half_extents()) else {
            return;
        };
        if self.next_id == u32::MAX {
            return;
        }
        // The units this building released that are still in its doorway.
        let released: Vec<Vec2> = (self.objects.values())
            .filter(|o| o.mover.as_ref().is_some_and(|mover| mover.exit == Some(producer)))
            .map(|o| o.pos)
            .collect();
        let recruits = producer_def(building).is_some_and(|def| def.recruitment.is_some());
        // Where it appears, which way it faces, the heading of a unit leaving an energy doorway, its box's half
        // extents there (out of a side, turned to face out of it: [`exit_half`]), and the side's door.
        let (pos, facing, heading, half, door) = match doorway {
            ExitDoor::Side(doorway) => {
                let toward = rally.map(|rally| rally.point() - building.pos);
                let free = |door: &Door| released.iter().all(|&pos| doorway.door_at(pos) != *door);
                let mut doors = doorway.order(toward).filter(free);
                let Some((door, pos, half)) = doors.find_map(|door| {
                    let half = exit_half(kind, &door);
                    Some((door, self.doorway_spot(&door, kind, half)?, half))
                }) else {
                    return;
                };
                (pos, door.outward(), None, half, Some(door))
            }
            ExitDoor::Centre(_) if !released.is_empty() => return,
            ExitDoor::Centre(portal) => {
                let grid = paths::unit_grid(self.nav.nav(), unit, half * 2.0);
                let rng = &mut self.rng;
                let rally = rally.map(|rally| rally.point());
                let Some(release) = portal.release(unit, half, grid, &self.objects, rally, |n| rng.below(n)) else {
                    return;
                };
                (release.pos, release.heading, Some(release.heading), half, None)
            }
        };
        if let Some(production) = self.objects.get_mut(&producer).and_then(|o| o.production.as_mut()) {
            production.release();
        }
        let id = self.insert_object(kind, Some(owner), pos, None, None);
        if let Some(o) = self.objects.get_mut(&id) {
            if recruits {
                o.lineage = vec![producer];
            }
            o.facing = math::vector_angle(facing);
            o.prev_facing = o.facing;
            if let Some(mover) = &mut o.mover {
                mover.exit = doorway.holds(pos, half).then_some(producer);
                mover.exit_heading = heading.filter(|_| mover.exit.is_some());
            }
        }
        self.events.push(SimEvent::UnitProduced { producer, unit: id });
        if let Some(kind) = rally.and_then(|rally| self.rally_order(id, rally)) {
            self.give_order(&[id], kind, false);
            // (M6-GF) Design change 0451: a vehicle out of an exit side parks round its rally point if that is taken
            // (`producers/park.rs`).
            if let Some(door) = door {
                self.park(id, &door);
            }
        }
    }

    /// Where a unit of `kind` with half extents `half` (its box as it faces out of the door: [`exit_half`]) appears in
    /// `door`: "half inside the building, at the middle of the exit side", or as far out as there is room when another
    /// unit or a blocked tile is right in front of the doorway (fully inside, against the exit side, at most: see
    /// `production::exit` for this reading of the design). Its box overlaps no unit of its layer and no blocked tile
    /// but the building's own, nor ground under those that the unit can't stand on (`production::ExitDoor::box_clear`,
    /// slice M5-Z1). `None` if there is no such spot: the unit waits inside, as behind an occupied doorway. The tiles
    /// are those of the grid its type walks on, by its Silhouette as it is (`paths::key_of`): (M6-GF) the turned box
    /// may be a hair longer, a 64 su one 64.000008 su out of a side facing west, which would name the next size class's
    /// grid, one no unit in the game walks on (plan decision T100's 64 × 48 Rig, `producers/tests.rs`).
    pub(in crate::sim) fn doorway_spot(&self, door: &Door, kind: ObjectKind, half: Vec2) -> Option<Vec2> {
        let def = kind.def();
        let (unit, silhouette) = (def.unit.as_ref()?, def.silhouette()?);
        let grid = paths::unit_grid(self.nav.nav(), unit, silhouette);
        // (M6-ST1) Its type's Domain, its own over its base's (`UnitDef::domain`).
        let domain = unit.domain();
        let clear = |p: Vec2| grid.is_none_or(|grid| door.box_clear(grid, half, p));
        let depth = door.outward().abs().dot(half);
        // The other units of its layer, with their half extents: (M5-V1) of their shapes' bounding boxes, a turned
        // Silhouette's for one that turns with its facing (`Shape::bound_half`), which holds all of it.
        let others = || {
            self.objects.values().filter_map(|o| {
                let other = o.def();
                let same_layer = domain.hard_collision() && other.unit.as_ref().is_some_and(|u| u.domain() == domain);
                other.silhouette().filter(|_| same_layer)?;
                Some((o.pos, Shape::of(o).bound_half()))
            })
        };
        let mut out: f32 = if clear(door.spot(0.0)) { 0.0 } else { -depth };
        for (pos, other_half) in others() {
            if let Some(room) = door.room_before(half, pos, other_half) {
                out = out.min(room - SPOT_CLEARANCE);
            }
        }
        // Not deeper than fully inside, the clearance apart.
        if out < -depth - 2.0 * SPOT_CLEARANCE {
            return None;
        }
        let spot = door.spot(out);
        let free = clear(spot) && others().all(|(pos, other_half)| !boxes_overlap(spot, half, pos, other_half));
        free.then_some(spot)
    }
}
