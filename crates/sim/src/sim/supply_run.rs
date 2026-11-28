//! The supply loop in the game (`crate::supply_run`): the chopper a Supply Tower comes with
//! ([`Sim::spawn_attached_chopper`]), the attachments a command breaks ([`Sim::detach`]), the schedules
//! ([`Sim::schedule`], [`Sim::step_schedules`]) and the step right after the aircraft's landings that attaches, lands
//! and flies the choppers ([`Sim::step_supply_runs`]). What they do down on a spot is the economy phase's
//! (`economy::supply_run`).

use std::collections::BTreeMap;

use glam::Vec2;

use super::Sim;
use crate::augmentation::Research;
use crate::economy::stations::is_station;
use crate::economy::supply_run::claimed_at;
use crate::ids::{ObjectId, PlayerId};
use crate::landing;
use crate::object::Object;
use crate::orders::OrderKind;
use crate::supply_run::{self, attached_to, chopper_def, due_in, is_tower, spot};
use crate::units::TilePos;

/// The ticks a scheduled chopper leaves before its flight's estimate says it must (`Sim::step_schedules`), and a
/// sixteenth of the flight more: what the estimate leaves out (`movement::flight_ticks`: the air units' separation)
/// delays it by less, so that it is over the station in time, and waits there for the delivery
/// (`Sim::step_supply_runs`). Technical: a margin for an estimate, not a rule of the design's.
const SCHEDULE_MARGIN_TICKS: u32 = 4;

/// The kind of `o`'s current order, if it has one.
fn current_order(o: &Object) -> Option<OrderKind> {
    Some(o.orders.as_ref()?.current.as_ref()?.order.kind)
}

impl Sim {
    /// `factions/gdo/economy.md > SupplyTower > On Placement`: "One Supply Chopper is spawned on the tower at no
    /// additional cost (included in tower's construction cost). This chopper is automatically attached to the tower":
    /// a chopper of the tower's def's kind (`SupplyTowerDef::chopper`) comes into the game down on its spot, attached
    /// to it, for the tower's owner, as the tower is placed (`Sim::apply_construction`). Not for a tower a map or a
    /// debug spawn puts in the game, which places the one object it names: **Placeholder (not in the design: whether a
    /// Supply Tower a map places comes with its chopper)**. It counts toward its owner's Unit Control as any unit, and
    /// may take it past the cap: **Placeholder (not in the design: a free chopper over the Unit Control cap)**.
    pub(super) fn spawn_attached_chopper(&mut self, tower: ObjectId) {
        let Some(o) = self.objects.get(&tower) else {
            return;
        };
        let (Some(def), Some(owner), Some(at)) = (supply_run::tower_def(o.kind), o.owner, spot(o)) else {
            return;
        };
        let (Some(_), Some(lands)) = (chopper_def(def.chopper), landing::def_of(def.chopper)) else {
            return;
        };
        if o.placement.is_none() || self.next_id == u32::MAX {
            return;
        }
        let id = self.insert_object(def.chopper, Some(owner), at, None, None);
        if let Some(o) = self.objects.get_mut(&id)
            && let Some(state) = o.chopper.as_mut()
        {
            state.attached = Some(tower);
            o.landing = Some(landing::down_on(tower, lands.landing_ticks));
        }
        self.cover_kind(def.chopper);
    }

    /// `factions/gdo/economy.md > SupplyTower > Landing and Attachment`: "Any command given to an attached Supply
    /// Chopper breaks its attachment to the tower": each chopper among `units`, which a player's order command goes to
    /// (`Sim::apply_command`), is unattached. The schedule's own orders (`Sim::step_schedules`) break nothing.
    pub(super) fn detach(&mut self, units: &[ObjectId]) {
        for id in units {
            if let Some(chopper) = self.objects.get_mut(id).and_then(|o| o.chopper.as_mut()) {
                chopper.attached = None;
            }
        }
    }

    /// `factions/gdo/economy.md > SupplyTower > AwaitingTarget[ScheduleDeliveries] resolution`: "sets ScheduledSDS to
    /// the clicked station" for each of `towers` (the gate's: the player's, each with a chopper attached). A tower
    /// keeps it when its chopper's attachment breaks, its deliveries waiting for an attached chopper again:
    /// **Placeholder (not in the design: a schedule whose tower has no chopper attached any more)**.
    pub(super) fn schedule(&mut self, towers: &[ObjectId], station: ObjectId) {
        for id in towers {
            if let Some(o) = self.objects.get_mut(id) {
                o.schedule = Some(station);
            }
        }
    }

    /// `factions/gdo/economy.md > SupplyTower > Scheduled Deliveries`, before phase 5a, for the towers before
    /// `first_new`: each player's scheduled towers in operation with a chopper attached, station by station.
    /// - "**Only one chopper flies to that SDS at a time**": none leaves while one of them has a Pick Up Supplies there
    ///   (one player's: **Placeholder (not in the design: whether two players' schedules on one station wait for each
    ///   other)**, they don't).
    /// - "**Priority: closest tower with a chopper ready to depart goes first**": of the choppers ready (idle, down on
    ///   their own tower), the one whose tower is nearest the station, centre to centre (the lower id of two as near).
    /// - "The attached chopper automatically departs at a time calculated so it arrives at the SDS simultaneously with
    ///   the next expected supply delivery": it leaves once the station's Supplies are due ([`supply_run::due_in`]) no
    ///   later than its lead: its take-off, its flight by the drag model's estimate along the straight line
    ///   (`movement::flight_ticks`, **from a distance-and-drag estimate**, not by simulating ahead) and its landing,
    ///   with [`SCHEDULE_MARGIN_TICKS`]; it waits over the station for what the margin leaves, and so is down on it
    ///   as the delivery comes (`Sim::step_supply_runs`). "If the travel distance is too long or deliveries at the SDS
    ///   are too frequent, the chopper departs immediately after dropping off supplies": its lead is past the
    ///   delivery as it is ready.
    /// - "After picking up supplies, the chopper returns to the tower, drops off supplies, and waits for the next
    ///   departure": it gets a Pick Up Supplies there and a Drop Off Supplies at its tower behind it, as the schedule's
    ///   own orders, which keep its attachment.
    pub(super) fn step_schedules(&mut self, first_new: ObjectId) {
        let scheduled: Vec<(ObjectId, ObjectId, PlayerId)> = (self.objects.range(..first_new).map(|(_, o)| o))
            .filter(|o| is_tower(o))
            .filter_map(|o| Some((o.id, o.schedule?, o.owner?)))
            .collect();
        if scheduled.is_empty() {
            return;
        }
        let mut attached: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();
        for o in self.objects.range(..first_new).map(|(_, o)| o) {
            if let Some(tower) = attached_to(o) {
                attached.entry(tower).or_insert(o.id);
            }
        }
        let mut runs: BTreeMap<(PlayerId, ObjectId), Vec<(ObjectId, ObjectId)>> = BTreeMap::new();
        for (tower, station, owner) in scheduled {
            if let Some(&chopper) = attached.get(&tower) {
                runs.entry((owner, station)).or_default().push((tower, chopper));
            }
        }
        for ((_, station), towers) in runs {
            if let Some((tower, chopper)) = self.departure(station, &towers) {
                self.give_order(&[chopper], OrderKind::PickUpSupplies(station), false);
                self.give_order(&[chopper], OrderKind::DropOffSupplies(tower), true);
            }
        }
    }

    /// The tower and chopper of `towers` (each a scheduled tower with its attached chopper) that leave for `station`
    /// now, if one does ([`Sim::step_schedules`]).
    fn departure(&self, station: ObjectId, towers: &[(ObjectId, ObjectId)]) -> Option<(ObjectId, ObjectId)> {
        let s = self.objects.get(&station).filter(|s| is_station(s.kind))?;
        let outbound = (towers.iter()).any(|(_, chopper)| {
            self.objects.get(chopper).and_then(current_order) == Some(OrderKind::PickUpSupplies(station))
        });
        if outbound {
            return None;
        }
        let ready = |&&(tower, chopper): &&(ObjectId, ObjectId)| {
            self.objects.get(&chopper).is_some_and(|o| {
                o.orders.as_ref().is_some_and(|orders| orders.is_idle())
                    && landing::down(o)
                    && landing::at(o) == Some(tower)
            })
        };
        let distance = |tower: ObjectId| {
            self.objects
                .get(&tower)
                .map_or(f32::INFINITY, |t| t.pos.distance_squared(s.pos))
        };
        let &(tower, chopper) = (towers.iter())
            .filter(ready)
            .min_by(|a, b| distance(a.0).total_cmp(&distance(b.0)).then(a.0.cmp(&b.0)))?;
        let due = due_in(s, claimed_at(&self.objects, station))?;
        (due <= self.lead_ticks(chopper, station)?).then_some((tower, chopper))
    }

    /// The ticks from a departure of chopper `chopper` now to its being down on `station`, with the schedule's margin
    /// ([`Sim::step_schedules`]): its take-off, which holds it still all but its last frame (`Mover::rise`), the flight
    /// from where it is to the station's spot (`movement::flight_ticks`, its drag doubled over Rugged tiles as the
    /// movement's is), the frame it lands in after it comes to rest, and its landing's frames.
    pub(super) fn lead_ticks(&self, chopper: ObjectId, station: ObjectId) -> Option<u32> {
        let o = self.objects.get(&chopper)?;
        let unit = o.def().unit.as_ref()?;
        chopper_def(o.kind)?;
        let lands = landing::def_of(o.kind)?;
        let to = spot(self.objects.get(&station)?)?;
        let rugged = |p: Vec2| (self.map.tile_preset(TilePos::containing(p))).is_some_and(|preset| preset.rugged);
        let drag_only = unit.base_def().brakes_by_drag;
        let movement = o.movement(Research::new(&self.players))?;
        let flight = crate::movement::flight_ticks(&movement, drag_only, (o.pos, to), o.facing, rugged)?;
        Some(
            (lands.takeoff_ticks + lands.landing_ticks).saturating_sub(1)
                + flight
                + SCHEDULE_MARGIN_TICKS
                + flight / 16,
        )
    }

    /// The supply loop's step, right before the landing step (`Sim::step_landings`), which flies and lands the choppers
    /// as every aircraft that lands, for the choppers before `first_new`, as phase 5a's orders leave them. The rules
    /// below that are the landing's are the landing step's, but for the supply loop's own on its spots
    /// ([`Sim::spot_rules`]):
    /// - **Attachments**: a chopper whose Attach to Tower has started is attached to its tower, and every other chopper
    ///   on the tower, landing or down, is evicted: it takes off (`factions/gdo/economy.md > SupplyTower > Landing and
    ///   Attachment`). One whose tower has another chopper attached by then (two
    ///   ordered to it in one tick, the lower id first) ends its order: **Placeholder (not in the design: an Attach to
    ///   a tower that another chopper attached to meanwhile)**.
    /// - **Each landing under way** takes a frame (`SupplyChopperDef::landing_ticks`); one whose frames are all done is
    ///   down, a ground target from here on (`crate::landing`).
    /// - **Each chopper flying to a spot** (`orders::supply_run`), spot by spot in id order, flies on to it, and lands
    ///   on it once it has come to rest over it (`Air::landing_reach_su`) and holds it: a spot is held by the
    ///   chopper on it, landing, down or rising off it, else by the chopper flying to it that is nearest it (the lower
    ///   id of two as near), and a tower's by its attached chopper alone (`factions/gdo/economy.md > SupplyTower >
    ///   Landing and Attachment`; a Drop Off or an Attach to a tower that another chopper is
    ///   attached to ends: **Placeholder (not in the design: a drop-off under way when another chopper attaches to the
    ///   tower)**). The others wait over the spot. A chopper with a Pick Up Supplies holds over its station, landing
    ///   only once the Supplies are due by the time it is down ([`supply_run::due_in`]): it is down as they come.
    pub(super) fn step_supply_runs(&mut self, first_new: ObjectId) {
        let choppers: Vec<ObjectId> = (self.objects.range(..first_new))
            .filter(|(_, o)| o.chopper.is_some())
            .map(|(&id, _)| id)
            .collect();
        if choppers.is_empty() {
            return;
        }
        for &id in &choppers {
            self.attach(id);
        }
    }

    /// Chopper `id`'s Attach to Tower, as it starts ([`Sim::step_supply_runs`]).
    fn attach(&mut self, id: ObjectId) {
        let Some(o) = self.objects.get(&id) else {
            return;
        };
        let Some(OrderKind::AttachToTower(tower)) = current_order(o) else {
            return;
        };
        if attached_to(o) == Some(tower) {
            return;
        }
        let owner = o.owner;
        let valid = self
            .objects
            .get(&tower)
            .is_some_and(|t| is_tower(t) && t.owner == owner);
        let taken = supply_run::attached_chopper(self.objects.values(), tower).is_some_and(|other| other != id);
        if !valid || taken {
            self.end_flight(id);
            return;
        }
        if let Some(state) = self.objects.get_mut(&id).and_then(|o| o.chopper.as_mut()) {
            state.attached = Some(tower);
        }
        let evicted: Vec<ObjectId> = (self.objects.values())
            .filter(|o| o.id != id && o.chopper.is_some() && landing::at(o) == Some(tower))
            .map(|o| o.id)
            .collect();
        for other in evicted {
            if let Some(o) = self.objects.get_mut(&other)
                && let (true, Some(mover)) = (o.chopper.is_some(), o.mover.as_mut())
            {
                landing::lift_off(o.kind, &mut o.landing, &mut mover.rise);
            }
        }
    }

    /// Chopper `id`'s order ends where it is.
    pub(super) fn end_flight(&mut self, id: ObjectId) {
        if let Some(o) = self.objects.get_mut(&id) {
            if let Some(orders) = &mut o.orders {
                orders.current = None;
            }
            if let Some(mover) = &mut o.mover {
                mover.stop();
            }
        }
    }

    /// The supply loop's rules on the spot of `target` for the landing step's flights to it (`Sim::step_landings`;
    /// [`Sim::step_supply_runs`]), as the choppers before `first_new` have them: on a Supply Tower in operation, the
    /// chopper attached to it, the only one that may take its spot (the others' flights there end); on a station, the
    /// ticks until its Supplies are due that no pick-up has claimed ([`supply_run::due_in`]), a chopper flying there
    /// landing only once they are due by the time it is down. Neither on any other structure.
    pub(super) fn spot_rules(&self, target: &Object, first_new: ObjectId) -> (Option<ObjectId>, Option<u32>) {
        let at = target.id;
        let only = is_tower(target)
            .then(|| (self.objects.range(..first_new)).find(|(_, o)| attached_to(o) == Some(at)))
            .flatten()
            .map(|(&id, _)| id);
        let due = (is_station(target.kind))
            .then(|| due_in(target, claimed_at(&self.objects, at)))
            .flatten();
        (only, due)
    }
}
