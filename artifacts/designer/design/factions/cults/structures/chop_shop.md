# Chop Shop

Tier 2 major building of the Tech/Weaponry branch. A scavenger's garage where Recruits are turned into vehicles.

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Armory

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Production
Produces Cults vehicles: the **Jackal** ( light vehicle, factions/cults/units/jackal.md). Each vehicle consumes multiple Recruits (number set per vehicle type), and its Unit Control cost equals the number of Recruits consumed. **Production works like the Armory** (user, 2026-11-26; factions/cults/structures/armory.md > Training): Recruits enter and join an internal pool; orders are queueable and serial; queuing an order deducts its Space Crystals cost and **reserves the vehicle's number of Recruits** from the pool (an order can only be queued if enough unreserved Recruits are inside); when it finishes, they are consumed and the vehicle exits to the RallyPoint. Eject All and Cancel work as at the Armory (cancel returns the reserved Recruits to the pool). Internal capacity, queue size, sides: TBD.
