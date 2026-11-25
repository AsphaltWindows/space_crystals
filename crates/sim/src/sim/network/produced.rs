//! Units produced into the Tunnel Network (`crate::network`, plan decision T35): an underground producer's release
//! (`ExitRule::Network`), and the surface rally point that sends a unit out of its parent Tunnel.

use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::network::{self, Held, Place};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::production::Rally;
use crate::sim::Sim;

impl Sim {
    /// `factions/syndicate/structures/README.md > Rally Point Behavior` (plan decision T35): `producer`, an underground
    /// expansion (`ExitRule::Network`), finished its unit. The unit comes into the game inside its owner's Tunnel
    /// Network, with no transit delay (design change 0246: "they can exit as soon as they are produced"; 0301,
    /// answering Q0087: "produced units rallied into the network have no delay either"), and:
    /// - "**No rally point, or rally point set on the parent Tunnel**: Unit stays in the Tunnel Network"; "**Rally
    ///   point on any own Tunnel**: the unit stays inside the network" (design change 0252);
    /// - "**Rally point set on the surface**: Unit auto-ejects from the parent Tunnel (Side A) and moves to the rally
    ///   point", which is "the unit's default right-click action at the rally target" (design change 0136,
    ///   [`Sim::rally_order_of`]); "**Rally point on an expansion**: treated as the surface location above it"
    ///   (0252: a Move to its centre, not an object target). It joins "the parent Tunnel's exit queue in order with
    ///   manual ejects" (0253).
    ///
    /// A unit whose producer has no parent Tunnel (plan decision T32: derived, [`Sim::parent_tunnel`]), or whose
    /// parent's tier doesn't let it out, stays in the network. Nothing holds a release up: the network has no
    /// doorway to be occupied.
    pub(in crate::sim) fn release_into_network(&mut self, producer: ObjectId) {
        let Some(building) = self.objects.get(&producer) else {
            return;
        };
        let (Some(owner), Some(production)) = (building.owner, &building.production) else {
            return;
        };
        let (Some(kind), rally, pos) = (production.current(), production.rally, building.pos) else {
            return;
        };
        if kind.def().unit.is_none() || self.next_id == u32::MAX {
            return;
        }
        if let Some(production) = self.objects.get_mut(&producer).and_then(|o| o.production.as_mut()) {
            production.release();
        }
        let id = ObjectId(self.next_id);
        self.next_id += 1;
        let object = self.new_object(id, kind, Some(owner), pos, None, None);
        let order = rally.and_then(|rally| self.surface_rally_order(&object, rally));
        let since = self.tick;
        self.held.insert(
            id,
            Held {
                object,
                place: Place::Network,
                since,
                entered: false,
                ejected: false,
            },
        );
        self.events.push(SimEvent::UnitProduced { producer, unit: id });
        let Some(order) = order else {
            return;
        };
        let parent = (self.parent_tunnel(producer))
            .filter(|parent| (self.objects.get(parent)).is_some_and(|t| network::admits(t, kind).is_ok()));
        if let Some(parent) = parent {
            self.queue_eject(parent, id, Some(order));
        }
    }

    /// The order a unit `unit` produced into the network leaves it with for `rally`, its producer's rally point, or
    /// `None` if the rally keeps it inside (`factions/syndicate/structures/README.md > Rally Point Behavior`,
    /// [`Sim::release_into_network`]): a rally on a Tunnel of its owner's keeps it inside; one on an underground
    /// structure of its owner's is the surface spot above it; any other gives the unit's default command on the target.
    fn surface_rally_order(&self, unit: &Object, rally: Rally) -> Option<OrderKind> {
        if let Rally::Object { id, .. } = rally
            && let Some(target) = self.objects.get(&id)
            && target.owner == unit.owner
        {
            if target.tunnel.is_some() {
                return None;
            }
            if target.def().structure.as_ref().is_some_and(|s| s.underground) {
                return Some(OrderKind::Move(Target::Point(target.pos)));
            }
        }
        self.rally_order_of(unit, rally)
    }
}
