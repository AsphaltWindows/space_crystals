# Mechanic: Restoring HP (Repair, Healing, Regeneration)

(Mechanics doc, 2026-11-28; Q0193. Defined independently of any unit; gathers rules previously written in the Recruit's, Field Engineer's, Agent's, First Aid Tent's, Shrine's, Tunnels' and Supply Tower's docs; no rule changed in the move.)

Three archetypes restore HP. (Colonists Shields regenerate by their own rule: systems/combat.md > Shields.) An object's doc says which it uses, with its parameters and exceptions.

## Archetype: Repair (by a repairer)
(user, 2026-11-26) A worker unit **repairs** a damaged own target by working on it.
- **Parameters (per repairer type)**: which targets it can repair (e.g. buildings and vehicles; only certain structures); which are free.
- **Several repairers on one target repair faster, scaling linearly** (2 repairers = twice the rate), as construction does.
- **Rate and cost** (designer placeholder, 2026-11-26): from 0 to full HP takes **half the target's build time** with one repairer, and costs **half its price** in the resource it was bought with, **paid gradually** as HP is restored. A vehicle's price for this counts its **Space Crystals only**, not units it consumed (Q0124; e.g. a Cults Seeker: 37.5 SC from 0 to full). A free target's rate still uses its build time.
- **When it ends** (user, 2026-11-26; Q0124): repairers **stop once the target is at full HP**; if the player **runs out** of the resource mid-repair, the order **ends** (a free repair never runs out).
- **Gliders can't be repaired** (they can't stop; user, 2026-11-26), except where a doc says so (e.g. a Pilgrim landed on an Airstrip).
- **Input**: **D: Repair** (targeted), or a right-click on a damaged own target the selection can repair, **unless the selection can enter it: then Enter wins** (systems/controls/hud.md; each repairer's right-click resolution).
- **Completed targets only** (a building under construction is built, not repaired).

## Archetype: Healing area
A structure (or other source) that **heals eligible units near it**, continuously, with no need to enter.
- **Parameters**: who is healed (e.g. own and allied infantry); radius (measured to the unit's **centre**); HP per second.
- **Several sources don't stack**: the highest applies.
- **Only units on the map**: units inside a building or carrier aren't healed.
- **Only once complete**: a structure under construction heals nobody (as every building works only once complete).
- **Docked service** (variant): an aircraft landed on a given structure is restored at the repairers' rate, free (e.g. the GDO Supply Chopper on its Supply Tower).

## Archetype: Regeneration
An object **regenerates a percentage of its MaxHP per second** (or a flat amount), by itself, while a condition holds.
- **Parameters**: rate; the condition (e.g. inside the Syndicate Tunnel Network; a completed expansion while its Tunnel is alive; for the duration of a buff); whether it runs during combat (yes unless stated).
- As a **timed buff** (e.g. the Cults Shrine's health regeneration), it follows the buff rules of the source.

## Used by (index only)
Repair: Cults Recruits (own buildings and vehicles, a landed Pilgrim), GDO Field Engineers (buildings and vehicles), Syndicate Agents (Tunnels, free; Laser Turrets). Healing area: GDO First Aid Tent; docked: Supply Chopper on its Supply Tower. Regeneration: units inside the Syndicate Tunnel Network (1%/s), Syndicate expansions (1%/s), the Cults Shrine's health regeneration (+2 HP/s buff).
