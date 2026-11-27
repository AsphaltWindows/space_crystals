# Q0175 — A vehicle building with no rally point stalls after its first vehicle

- **Date**: 2026-11-26T23:40:02-05:00
- **Blocking**: no. M5-N6 (the Chop Shop and the Seeker) is built as below, and the stall is in the playtest notes with "set a rally point". It stays so until the answer.
- **Docs**:
  - `core/entities.md > Entrance and exit > Exit Doorway`, points 2–4;
  - `systems/units.md > UnitCollision > Ground Collision` ("Vehicles"), as changed by 0440;
  - `factions/cults/structures/chop_shop.md`;
  - `factions/gdo/structures/factory.md` (M6, the same case).

## Question
The Exit Doorway rules promise that own idle units don't stall an exit: "Own idle units outside step aside for the unit walking out (friendly yielding), so they don't cause stalls" (point 4). Since 0440, idle wheeled and tracked vehicles don't yield. So with no rally point, a Chop Shop stalls after its first vehicle, and so will the GDO's Factory.

Is that stall intended? Or which rule should give way?

## Context
Measured in M5-N6:
- **With no rally point,** a unit "walks out toward its RallyPoint (or straight out of the exit side if there is none)" (point 2). As built, it stops as soon as its box is clear of the building.
- **The first Seeker** drives out and stops just clear of the Chop Shop's side C, idle, in front of the doorway.
- **The second Seeker** appears in the doorway and drives straight out. The first is in its way, and as an idle wheeled vehicle it doesn't step aside (0440). So the second stops in the doorway ("If the way is blocked, it stops in the doorway").
- **The building pauses** (point 3), and the rest of the queue waits until the player moves the first Seeker.
- **Infantry don't hit this:** idle infantry step aside under point 4. The Syndicate's Tunnels say for their own case "to keep the exit clear, set a rally point" (`factions/syndicate/tunnel_menus.md`).

## Options
1. **Keep it** (as built). A vehicle building needs a rally point, or the player clears the doorway. The playtest notes say so.
2. **An idle own vehicle in front of a doorway clears it** for the unit leaving: it drives on straight ahead (or reverses, if that's shorter) until the way out is free. This is an exception to 0440 for doorways only. Elsewhere vehicles still stand.
3. **A vehicle with no rally point drives further out:** for example to the first spot along the exit's line where it leaves the next vehicle room to come out (its own length, plus a margin). Vehicles then line up away from the door, and the building only stalls once the line meets something.
4. **Vehicle buildings get a default rally point** a short way out of the exit (for example 3 GU), which the player can move or clear. Cleared, option 1 applies.
