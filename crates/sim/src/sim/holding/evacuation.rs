//! `factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction` (slice M5-F1): "Applies to every
//! Cults building that can hold units inside [...]. When the building is destroyed: **Everyone inside is evacuated
//! alive**"; "**Units come out unchanged**"; "**Placement**: evacuated units appear around the building's footprint
//! wherever there is open space, unharmed".
//!
//! It is part of the one way objects leave the game (`Sim::remove_objects`): the units inside a building that goes are
//! taken down before anything else of the removal ([`Sim::evacuations`]), and put on the map once the building has gone
//! ([`Sim::evacuate`]), in the same tick, so before the defeat check judges their owner (`sim/phase.rs`). Each comes
//! out at the first spot around the footprint where it fits (plan decision T62, `Sim::spot_around`; Placeholder
//! (Q0106 point 1): "Rings around the footprint, nearest first, each ring clockwise from the middle of the exit side,
//! as far out as needed"), the units placed before it included, facing away from the footprint, and idles
//! (Placeholder (Q0106 point 4): "They idle").

use glam::Vec2;

use crate::events::SimEvent;
use crate::holding;
use crate::ids::ObjectId;
use crate::object::Placement;
use crate::placement::Round;
use crate::production::Doorway;
use crate::sim::Sim;

/// The units of one building that is about to leave the game ([`Sim::evacuations`]), as it is before it goes.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sim) struct Evacuation {
    building: ObjectId,
    placement: Placement,
    /// The direction out of its exit side, where each ring's search starts (Placeholder (Q0106 point 1)).
    towards: Vec2,
    /// Its units, in the order they come out.
    units: Vec<ObjectId>,
}

impl Sim {
    /// The units inside the buildings among `gone` that hold units, as the buildings are before they go, in id order
    /// of the buildings. Each building's come out in a fixed order (Placeholder (Q0106 point 2): "The unit in training,
    /// then the reserved ones in queue order, then the pool, longest inside first"): its pool's, as nothing trains yet.
    ///
    /// **For slice M5-F2** (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`: "units
    /// reserved by queued orders, the unit currently being trained"): the unit in training and the reserved ones go
    /// first here, in queue order, read before `Sim::remove_objects` refunds the queue (`Sim::refund_production`, which
    /// runs after this): "Orders are cancelled with a full refund", and "a Recruit mid-training comes out as a
    /// Recruit".
    pub(in crate::sim) fn evacuations(&self, gone: &[ObjectId]) -> Vec<Evacuation> {
        if self.held.is_empty() {
            return Vec::new();
        }
        (gone.iter())
            .filter_map(|&id| {
                let building = self.objects.get(&id)?;
                holding::holds(building)?;
                let units: Vec<ObjectId> = self.held_in(id).iter().map(|held| held.object.id).collect();
                if units.is_empty() {
                    return None;
                }
                Some(Evacuation {
                    building: id,
                    placement: building.placement?,
                    towards: Doorway::of(building).map_or(Vec2::X, |doorway| doorway.door().outward()),
                    units,
                })
            })
            .collect()
    }

    /// The units of `evacuations` come out around their buildings' footprints, which have left the game, building by
    /// building, each in its order (module docs), with a `SimEvent::Evacuated`. Placeholder (not in the design: the
    /// design has "wherever there is open space", and no case with none): a unit for which no spot on the whole map
    /// is clear leaves the game (`Sim::remove_held`).
    pub(in crate::sim) fn evacuate(&mut self, evacuations: &[Evacuation]) {
        for evacuation in evacuations {
            let building = evacuation.building;
            for &unit in &evacuation.units {
                let Some(kind) = self.held.get(&unit).map(|held| held.object.kind) else {
                    continue;
                };
                let rings = self.rings_on_map(kind, &evacuation.placement);
                let spot = self.spot_around(kind, &evacuation.placement, rings, evacuation.towards, Round::Clockwise);
                match spot {
                    Some((pos, outward)) => {
                        if self.put_on_map(unit, pos, outward) {
                            self.events.push(SimEvent::Evacuated { unit, building });
                        }
                    }
                    None => self.remove_held(&[unit]),
                }
            }
        }
    }
}
