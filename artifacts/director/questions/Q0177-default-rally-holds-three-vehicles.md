# Q0177 — The default rally point holds three Seekers; a fourth stalls the doorway

- **Date**: 2026-11-27T01:32:01-05:00
- **Blocking**: no. M5-S8 built 0446 as written (3 GU, a placeholder). Its Verification passes with three Seekers. It stays so until the answer, and the playtest notes say so.
- **Docs**:
  - `core/entities.md > Entrance and exit > Exit Doorway`, point 7;
  - `factions/cults/structures/chop_shop.md > Placeholder stats` ("at most 5 Seekers queued");
  - `systems/controls/command_behaviors.md` (a Move's arrival);
  - design changes 0446 and 0447.

## Question
With the default rally point 3 GU out, a Chop Shop left alone releases three Seekers cleanly. The fourth stops in the doorway, and the building pauses. Yet the Chop Shop can queue five Seekers.

Is three enough? Or should more vehicles fit before the door stalls?

## Context
Measured in M5-S8:
- **Where they park:** each new Seeker gets a Move to the rally point, like every rallied unit. The first parks on the point. Each later one stops behind the last, on its way from the door, because the point is taken and parked vehicles don't yield (0440). They end 0, 62 and 126 su behind the point.
- **The stall:** the third's tail ends at the doorway. A fourth stops in the doorway, and a fifth waits inside.
- **Five queue:** the Chop Shop's capacity of 10 lets five Seekers queue (2 Recruits each). A Hand of Heaven is 64 su long, so even fewer of those fit.
- **A rally point the player sets farther away** gives more room, as before.

## Options
1. **Keep 3 GU** (as built). Three vehicles fit, and a player who queues more moves them or sets a rally point.
2. **A longer default distance:** for example 6 GU, which fits about five Seekers in a line, the queue's maximum. Fewer Hands of Heaven fit, since they are longer.
3. **Vehicles arriving at a taken rally point park around it,** at the nearest free spot beside the point, rather than in a line back toward the door. More fit at any distance. This changes how a vehicle's Move ends when its goal is taken, at least for rally moves.
