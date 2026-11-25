# Q0105 — Armory training: which Recruit, its HP, ejected Recruits, the doorway

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — proceeding on assumption: (a) for each point.
- **Docs**: `factions/cults/structures/armory.md > Training`, `> Eject All`, `> ObjectInterfaceState[Armory]`; `factions/cults/structures/README.md > Cults Infantry` ("no natural health regeneration"); `factions/syndicate/structures/README.md > Rally Point Behavior` (0252)

## Question
1. **Which Recruit.** Queuing "reserves one Recruit from StoredRecruits". Which one? Lineage decides which Hall the trained unit counts against.
2. **HP.** A damaged Recruit is trained into a Militant of the same MaxHP. Does the Militant keep the damage?
3. **Ejected Recruits.** Do they walk to the rally point, or straight out?
4. **One doorway.** Trained units and ejected Recruits share the exit doorway. In what order do they come out?
5. **Several Armories.** One press of Q goes to several Armories. Which ones train?

## Context
"The trained unit inherits the Recruit's AugmentationState unchanged." "Cults infantry have no natural health regeneration. Their only source of healing is the Shrine's health regeneration ability." Ejected Syndicate units go to the rally (0252). Group production: "those with the least work queued pay first, as many as the player can afford" (0083).

## Options
1. (a) The one inside the longest (as the Tunnel ejects, 0250). (b) The healthiest. (c) The one that entered last.
2. (a) It keeps the damage. (b) It comes out at full HP, which makes training a heal.
3. (a) To the rally point, as trained units do. (b) Straight out of side C.
4. (a) In the order they are ready to leave. (b) Trained units first.
5. (a) 0083's order, among the Armories that have an unreserved Recruit. (b) Only the first selected.

Proposed: (a) for each.
