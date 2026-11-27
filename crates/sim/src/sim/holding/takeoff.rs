//! (M5-N7) The takeoff of a trained aircraft from its building's runway (`ExitRule::Runway`, plan decision T71;
//! `production::Runway`): the Airstrip's Sparrow (`factions/cults/structures/airstrip.md > Production`: "A finished
//! aircraft **takes off from the Airstrip** to its RallyPoint; Cults aircraft (so far) never land back").
//!
//! - **When** ([`Sim::takes_off`]): the moment its training is finished, in the step that lets a pool producer's units
//!   out (`Sim::step_holding`), whatever waits at the building's doorway: "Aircraft and units released underground ...
//!   are not covered by this rule" (`core/entities.md > Structure Type > Exit Doorway`), so nothing on the runway holds
//!   the next one back and the building never pauses. The units it lets out on the ground, ejected or released, keep to
//!   their doorway and their order (design change 0348).
//! - **How** ([`Sim::take_off`]): the first Recruit the entry reserved, the one inside the longest (design change 0332:
//!   "first in, first trained"), comes out as the aircraft it trains (`Sim::train`: at full HP whatever the Recruit's
//!   damage, its Augmentations lost, design change 0330), over the runway's `from` end, facing along it, flying at its
//!   IdleSpeed (`factions/cults/units/sparrow.md > Flight rules`: "**Takeoff**: it appears over the runway's B end
//!   facing C at IdleSpeed and speeds up along the runway"), on its takeoff run (`Mover::takeoff`). Then it goes to the
//!   building's RallyPoint with the order the rally gives, as a producer's unit does (`Sim::rally_order`), from the
//!   runway's end on (`movement::glider`); with no rally point it circles 2 grid units past the runway's `toward` end
//!   ("**No rally point**: it circles 2 grid units past the runway's C end", `Mover::orbit`).

use crate::augmentation::Research;
use crate::defs::MovementAttrs;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::production::{Production, QueueEntry, Runway};
use crate::sim::Sim;

impl Sim {
    /// Whether unit `unit`, ready to leave `building`, takes off from its runway ([`Sim::take_off`]): it is the first
    /// unit the front entry reserved, an aircraft's training, at a building whose production takes off from a runway.
    pub(in crate::sim) fn takes_off(&self, building: ObjectId, unit: ObjectId) -> bool {
        let Some(o) = self.objects.get(&building) else {
            return false;
        };
        let production = o.production.as_ref();
        Runway::of(o).is_some()
            && production.is_some_and(|production| {
                matches!(production.current(), Some(QueueEntry::Unit(_)))
                    && production.reserved_by_current().first() == Some(&unit)
            })
    }

    /// Unit `unit`, the first the finished front entry of `building` reserved, takes off from the building's runway
    /// as the aircraft its entry trains (module docs), with a `SimEvent::UnitProduced`; the entry leaves the queue.
    pub(in crate::sim) fn take_off(&mut self, building: ObjectId, unit: ObjectId) {
        let Some(o) = self.objects.get(&building) else {
            return;
        };
        let (Some(runway), Some(production)) = (Runway::of(o), o.production.as_ref()) else {
            return;
        };
        let rally = production.rally;
        let Some(QueueEntry::Unit(kind)) = production.current() else {
            return;
        };
        let Some(half) = kind.def().half_extents() else {
            return;
        };
        if !self.put_on_map(unit, runway.start(half), runway.along()) {
            return;
        }
        let released = (self.objects.get_mut(&building))
            .and_then(|o| o.production.as_mut())
            .and_then(Production::release);
        if let Some((entry, reserved)) = released {
            self.complete(unit, entry, reserved.get(1..).unwrap_or_default());
        }
        let orbit = rally.is_none().then(|| self.map.clamp_point(runway.circle_point()));
        let research = Research::new(&self.players);
        let Some(o) = self.objects.get_mut(&unit) else {
            return;
        };
        // A Glider's flight: the only aircraft so far (`systems/units.md > GliderMovement`).
        if let (Some(MovementAttrs::Glider { idle_speed, .. }), Some(mover)) = (o.movement(research), &mut o.mover) {
            mover.momentum = runway.along() * idle_speed;
            mover.takeoff = Some(runway.takeoff());
            mover.orbit = orbit;
        }
        self.events.push(SimEvent::UnitProduced {
            producer: building,
            unit,
        });
        if let Some(kind) = rally.and_then(|rally| self.rally_order(unit, rally)) {
            self.give_order(&[unit], kind, false);
        }
    }
}
