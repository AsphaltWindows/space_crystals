# Mechanic: Unit Production

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit or faction; gathers rules previously written in the GDO overview's Unit Control section, the Barracks', the Armory's and the Cults Evacuation on Destruction; no rule changed in the move.)

A **producer** is a building (or expansion) that makes units from a queue. A producer's doc says which archetype it uses, its parameters, and its exceptions.

## Common rules (every producer)
- **Serial queue**: orders are processed one at a time, in the order queued; only the front is in production.
- **Paid when queued** (user, 2026-11-24): queueing deducts the unit's whole cost at once (every resource it costs, e.g. Space Crystals and Supplies; the order needs enough of each). **Cancel** (X) removes the **last** entry and **refunds its full cost**.
- **Queue length**: at most **20** orders, the one in production included (user, 2026-11-24), unless the archetype bounds it lower.
- **Capacity is reserved when production starts** (user, 2026-11-24; replaces "reserved when queued"; same model for every faction's capacity: Unit Control, Tunnel Space, Beacon Capacity):
  - Queueing does **not** check or reserve capacity.
  - When an order reaches the front and production is about to start, it needs free capacity at least equal to its cost; that capacity then counts as used from that moment, through production and for as long as the unit lives.
  - Not enough: the producer **waits** with the order at the front (0% progress) and starts by itself once there is room; the player can still cancel it for a full refund. A blocked queue raises a notification (notification system not yet designed).
  - A unit already in production always finishes.
  - **Several producers waiting** (user, 2026-11-24): when room opens for fewer than are waiting, they are served in a **fixed internal order** (e.g. lowest id, oldest first); the player has no control over it.
  - **No progress, no start**: a producer that can't make progress (e.g. at zero power) doesn't take the room; another may.
  - **Zero-cost units always pass**: a unit whose capacity cost is 0 never waits, even when used is over available.
  - The HUD's "used" count includes units in production, not merely queued ones.
- **Finished but can't get out**: a finished unit **waits at 100%** while its exit is blocked (core/entities.md > Exit Doorway; runways: systems/mechanics/aircraft_landing.md > Runway).
- **Rally point**: a new unit gets its own default right-click command against the producer's rally point (systems/controls/interface.md > CommandPanel > What a rally point makes a new unit do); with none, it gets no command.
- **Producer destroyed**: every order (the one in production included, and a finished unit still fully inside) is **refunded as if cancelled**, and reserved capacity is released (user, 2026-11-24). (Archetypes may add what happens to units inside.)

## Archetype: Queue producer
Makes units from nothing but their cost (e.g. GDO Barracks, Factory, Airfield). Parameters: the units it makes (cost, time, TechPrerequisites), its exit, its power use.

## Archetype: Pool producer
**Turns units held inside into other units** (e.g. Cults Recruits into Militants, vehicles, aircraft; user, 2026-11-26).
- **Pool**: eligible units **enter** the producer and wait in an **internal pool** (parameter: capacity, counting reserved units).
- **An order reserves** the number of pool units the product needs (parameter per product), **first in, first used** (Q0105), removing them from the pool while it waits; it can't be queued without them. The queue is therefore bounded by the pool's capacity.
- **On completion** the reserved units are **consumed** and the product comes out. What carries over (e.g. infantry keep the consumed unit's damage and augmentations; vehicles and aircraft come out at full HP, Q0121) is a parameter.
- **Cancel** returns the reserved units to the pool, with the full refund.
- **Release / Eject All** let pool units back out unchanged (reserved ones stay committed).
- **Destroyed**: everyone inside comes out alive (factions/cults/economy.md > Evacuation on Destruction), and orders are refunded.

## Used by (index only)
Queue producer: GDO Barracks, Factory, Airfield; Syndicate production expansions (output into the Tunnel Network or ejected, factions/syndicate/structures/README.md > Rally Point Behavior); Colonists producers (see their docs). Pool producer: Cults Armory, Chop Shop, Airstrip, Cathedral.
