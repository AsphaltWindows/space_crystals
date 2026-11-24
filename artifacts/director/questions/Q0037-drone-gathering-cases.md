# Q0037: Drone gathering: the cases the loop doesn't define

- **Date**: 2026-11-24T05:02:31-05:00
- **Blocking**: no. Slice H builds the proposals.
- **Docs**: `factions/colonists/units.md > Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`, `> Assigned Drop-off`, `> Unit Commands`; `factions/colonists/structures.md > Robo Portal > RallyPoint behavior`; `core/map.md > SpaceCrystalsPatch`.

## Question
1. **After a Drop Off order** (right-click an own Egress Node while carrying), what does the Drone do? The docs are silent. **Proposal:** it goes back to the patch it was mining, if any; otherwise it idles.
2. **Gather while carrying a load.** **Proposal:** it delivers the load to its AssignedDropOff first, then goes to the new patch.
3. **A load when the Drone gets another order** (Move, Stop). **Proposal:** the Drone keeps it and delivers it at its next drop-off.
4. **Mining interrupted** (ordered away during the 32 frames). **Proposal:** nothing is collected, and the patch frees at once.
5. **"The nearest unoccupied patch to itself"**: among which patches? With no limit, a Drone would leave for any free patch on the map, even in an enemy base. **Proposal:** patches the owner knows (seen or remembered), not under a structure, within 6 grid units of the patch it arrived at (placeholder). Otherwise it waits.
6. **Waiting at an occupied patch: where, and in what order?** **Proposal:** where it stands, within MiningReach. The first Drone to wait gets the patch next; ties go by id.
7. **A patch that disappears** (depleted) while Drones go to it or wait. **Proposal:** treated as occupied, so the redirect rule applies. With no patch left, the Drone delivers any load and idles.
8. **No Egress Node**: Gather when the player has none, or the AssignedDropOff is destroyed. **Proposal:** when the Drone starts its return, it assigns the nearest own Egress Node. With none, it idles holding its load.
9. **Robo Portal rally on a patch.** `> Robo Portal > RallyPoint behavior` first says the unit gets "the default right-click command resolved against the RallyPoint target", which for a Drone on a patch is Gather. It then lists "rally on friendly/neutral object -> Move to object". These disagree for a patch. **Proposal:** Gather.
10. **A Drone rallied on an enemy**: "Move to that location". **Proposal:** a Move to the point where the enemy stands when the Drone appears, not a Follow. Please confirm.
11. **A patch under a GDO Extraction Plate.** **Proposal:** Drones can't gather it, because a right-click hits the plate, and it is never a redirect target. The GDO side of the same patch is Q0034.
12. **Orders Shift-queued behind a Gather.** Gather never ends, so a queued order would never start. **Proposal:** a queued order starts after the Drone's next drop-off.
13. **The last load of a nearly empty patch.** **Proposal:** the Drone mines the full 32 frames and takes what is left, fewer than 8.

## Context
Gathering is new: `systems/controls/commands.md` and `behaviors.md` define no Gather or Drop Off, so `units.md` is the whole spec. The loop's numbers and the occupancy rule are clear. These points are the transitions around them.

## Options
- **Point 1:** A, as proposed: back to its patch. B, idle.
- **Point 5:**
  - A, a radius around the patch, as proposed;
  - B, the same patch field (patches connected through neighbours);
  - C, any known patch.
- **Point 9:** A, Gather: the default right-click. B, Move to the patch, as listed.
- **Point 12:** A, after the next drop-off, as proposed. B, at once: the Gather is dropped and the load is kept.
- **Other points:** the proposal, or a rule you give.
