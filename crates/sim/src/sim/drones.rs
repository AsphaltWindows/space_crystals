//! (M6-GD) The Drone Operator's drones in a game (`crate::stock`, `crate::tether`; plan decisions T89 and T91): a
//! Buy's purchase, paid at the press ([`Sim::buy`]), its frames in the economy phase ([`Sim::step_stocks`]), the launch
//! right after phase 5a ([`Sim::step_launches`]), and the tether's step right after phase 5d ([`Sim::step_tethers`]).

use super::Sim;
use crate::combat::Projectile;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::object::Object;
use crate::orders::launch;
use crate::tether::{self, Fall, Tether};
use crate::vision::Hiding;
use crate::{paths, stock};

/// The death of `o`, a drone that ended (`crate::tether`): what it was and where, as a destroyed object's
/// (`SimEvent::Died`), since its `Removed` event follows in the same tick.
pub(super) fn died(o: &Object) -> SimEvent {
    SimEvent::Died {
        id: o.id,
        kind: o.kind,
        owner: o.owner,
        pos: o.pos,
        hiding: Hiding::of(o),
    }
}

impl Sim {
    /// A Buy for unit `id` (`Sim::use_ability`; the gate checked the first unit of the command): if it may start a
    /// purchase now (`stock::refusal`), its type's price is paid from its owner's bank at the press, through the bank's
    /// checked spend (plan decision T89, T7), and its purchase starts (`stock::start_buying`). Returns whether the
    /// Buy goes on to the units after it: not once the bank can't pay for one (design change 0083's rule for queuing).
    pub(super) fn buy(&mut self, id: ObjectId) -> bool {
        let Some(o) = self.objects.get(&id) else {
            return true;
        };
        let (Some(stock), Some(def)) = (o.stock, stock::def_of(o.kind)) else {
            return true;
        };
        if stock::refusal(o, &stock).is_some() {
            return true;
        }
        let owner = o.owner;
        let paid = (self.players.iter_mut())
            .find(|p| Some(p.id) == owner)
            .is_some_and(|p| p.spend_crystals(def.price));
        if !paid {
            return false;
        }
        if let Some(o) = self.objects.get_mut(&id) {
            stock::start_buying(o);
        }
        true
    }

    /// The stocks' step, in the economy phase (`stock::step`): each purchase under way takes a frame, at full speed,
    /// and each that is done adds its item to its unit's stock, with a `SimEvent::StockBought`.
    pub(super) fn step_stocks(&mut self, first_new: ObjectId) {
        for unit in stock::step(&mut self.objects, first_new) {
            self.events.push(SimEvent::StockBought { unit });
        }
    }

    /// The launches, right after phase 5a (`orders::launch`): each unit before `first_new` whose current order is a
    /// Launch, in id order, ends it, and launches its drone if it may ([`Sim::launch`]).
    pub(super) fn step_launches(&mut self, first_new: ObjectId) {
        let launching: Vec<ObjectId> = (self.objects.range(..first_new))
            .filter(|(_, o)| launch::launching(o))
            .map(|(&id, _)| id)
            .collect();
        for operator in launching {
            if let Some(orders) = self.objects.get_mut(&operator).and_then(|o| o.orders.as_mut()) {
                orders.current = None;
            }
            self.launch(operator);
        }
    }

    /// `operator` launches a drone (`crate::tether`): if its stock holds one and none is out, a unit of its stock's
    /// kind comes into the game over it ("The drone appears over the operator", plan decision T91), with a tether
    /// naming it whose battery runs out its type's battery from now (`Tether::battery_until`), and its stock holds one
    /// less: a `SimEvent::DroneLaunched` after the drone's `SimEvent::Spawned`. The drone is the tick's new object: it
    /// moves and attacks from the next tick on.
    fn launch(&mut self, operator: ObjectId) {
        let Some(o) = self.objects.get(&operator) else {
            return;
        };
        let (Some(stock), Some(def)) = (o.stock, stock::def_of(o.kind)) else {
            return;
        };
        let Some(leash) = tether::def_of(def.item) else {
            return;
        };
        if tether::holds(o) || stock.count == 0 || self.next_id == u32::MAX {
            return;
        }
        let (owner, pos) = (o.owner, o.pos);
        let drone = self.insert_object(def.item, owner, pos, None, None);
        let battery_until = self.tick.saturating_add(leash.battery);
        if let Some(d) = self.objects.get_mut(&drone) {
            d.tether = Some(Tether {
                operator,
                battery_until,
            });
        }
        if let Some(o) = self.objects.get_mut(&operator) {
            o.drone = Some(drone);
            if let Some(stock) = &mut o.stock {
                stock.count -= 1;
            }
        }
        self.events.push(SimEvent::DroneLaunched { operator, drone });
    }

    /// (M6-GD) The removal path (`Sim::remove_objects`), for the objects `ids` about to leave the game: the drones their
    /// operators take along, those not among `ids`, in the order of their operators in `ids`. Each falls without
    /// exploding (`crate::tether`: "Its **operator dies**: Drone falls, no explosion"), with a `SimEvent::DroneFell`
    /// and a `SimEvent::Died`.
    pub(super) fn drones_of(&mut self, ids: &[ObjectId]) -> Vec<ObjectId> {
        let taken: Vec<(ObjectId, ObjectId)> = (ids.iter())
            .filter_map(|&operator| Some((self.objects.get(&operator)?.drone?, operator)))
            .filter(|(drone, _)| !ids.contains(drone))
            .collect();
        let mut drones = Vec::with_capacity(taken.len());
        for (drone, operator) in taken {
            let Some(o) = self.objects.get(&drone) else {
                continue;
            };
            self.events.push(SimEvent::DroneFell {
                drone,
                operator,
                why: Fall::Operator,
            });
            self.events.push(died(o));
            drones.push(drone);
        }
        drones
    }

    /// The tether's step, right after phase 5d (`crate::tether`): the drones whose end comes in this tick
    /// (`tether::ends`) fall, in id order, each with a `SimEvent::DroneFell` and a `SimEvent::Died`, and leave the game
    /// through the removal path, which frees their operators. One past its leash drops its attack's blast where it
    /// falls (`Projectile::dropped`), which lands in this tick's phase 5e, after the drone has gone.
    pub(super) fn step_tethers(&mut self, first_new: ObjectId) {
        let ends = tether::ends(&self.objects, first_new, self.tick);
        if ends.is_empty() {
            return;
        }
        let mut fallen = Vec::with_capacity(ends.len());
        for (drone, operator, why) in ends {
            let Some(o) = self.objects.get(&drone) else {
                continue;
            };
            let attack = o.def().unit.as_ref().and_then(|unit| unit.attack.as_ref());
            if let Some(attack) = attack.filter(|_| why.explodes()) {
                self.projectiles
                    .push(Projectile::dropped(drone, o.owner, attack, o.pos, self.tick));
            }
            self.events.push(SimEvent::DroneFell { drone, operator, why });
            self.events.push(died(o));
            fallen.push(drone);
        }
        self.remove_objects(&fallen);
        paths::sync(&mut self.nav, &self.objects, &mut self.nav_version);
    }
}
