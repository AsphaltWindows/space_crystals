# Mechanic: Garrisoning

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit; gathers rules previously written in the Cults Garrison's and GDO Bunker's docs; no rule changed in the move.)

A **garrison** is an object that infantry enter and **fire out of**. Unlike a carrier (systems/mechanics/carrying.md), its occupants fight from inside. A garrison's doc says which archetype it uses, gives its parameters, and lists only its exceptions.

## Parameters
- **Capacity** and **who can enter**.
- **Weapons inside**: each occupant type fires its own weapon, a **substitute weapon** (e.g. an unarmed unit fires a given unit's attack), or **doesn't fire** (sheltered only).
- **Range bonus** for some occupant types (e.g. +1 grid unit), or none.
- **SightRange** of the garrison: occupants see with it.
- **Door**: the side units enter and leave by.

## Rules (every garrison)
- **Entering**: the **Enter** command (systems/controls/commands.md > Enter): right-click the own garrison with eligible units selected; they go in through the door, up to capacity.
- **Each occupant picks its own target** by the normal targeting rules (user, 2026-11-26; Q0115), and fires at enemies its player sees within its range. **Range is measured from the garrison's edge** (its footprint or Silhouette), MinRange included.
- **A substitute weapon is used unchanged**: the occupant's own augmentations or upgrades don't change it (Q0115).
- **No abilities or spells from inside** (Q0115): occupants only fire their weapons. (A garrison's doc may add entry rules, e.g. the Bunker: a shielded Peacekeeper lowers its shield on entering, Q0164.)
- **Protection**: occupants **can't be targeted and take no damage** while the garrison stands.
- **Seen as a threat** (user, 2026-11-27; Q0139): enemies' target scans treat the garrison as a threatening target while any occupant could attack the scanner's domain (systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority).
- **Capacity counts**: occupants are not consumed and still count toward their faction's capacity (Unit Control etc.).
- **Leaving** (user, 2026-11-26; Q0115/Q0116): through the door; **Eject All** lets everyone out **at once**, ignoring collision with each other until they are clear.
- **Garrison destroyed**: **everyone inside comes out alive**, unharmed and idle. Where they appear is a parameter of the archetype (below).
- A garrison with **no occupants able to fire** doesn't attack.

## Archetypes

### Garrison structure
A building. **Destroyed**: occupants appear all at once **inside its former footprint**, unharmed, spread over it; overlapping units ignore each other until separated (user, 2026-11-28, for every garrison structure; as the Cults Evacuation on Destruction, factions/cults/economy.md). Panel: **A: Eject All**.

### Garrison vehicle
A vehicle whose occupants fire out (to be defined with the first one: the Cults heavy vehicle, in design). Open: firing on the move, range from its Silhouette's edge, where occupants appear when it is destroyed.

## Used by (index only)
Garrison structure: Cults Garrison, GDO Bunker.
