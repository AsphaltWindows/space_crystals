# 0465 — Airfield landings: hold fire, queued orders wait for the rearm, "Rearm" command, slot layout

- **Date**: 2026-11-28T09:00:00-05:00
- **Answers**: Q0189
- **Docs**:
  - `factions/gdo/structures/airfield.md > Landing slots`, `> Returning and rearming`
  - `factions/gdo/units/hornet.md` (panel; corrected example shares)
  - `factions/gdo/units/dragonfly.md` (panel)

## What changed and why
1. **Landed aircraft hold fire** (user, option a, as built). They fire only in the air.
2. **A Shift-queued order waits for the full rearm** (user, option a, as built). Only an order *given* during rearming takes off early (0428).
   - **Renamed:** the command "Return to Base" is now **"Rearm"** (user). The name says it rearms, which is why a queued order waits for it. Same hotkey F; right-clicking the own Airfield does the same.
3. **Landed aircraft are out of the air separation** (designer, as built).
4. **Slot layout:** a 2x2 grid, one slot near each corner (designer placeholder, as built).
5. **A new aircraft with no free slot waits inside**, as behind a blocked exit (designer, as built).
6. **Hornet damage stays 45** (as built). The doc's example shares were my arithmetic error. Corrected to about 33 for a 64x40 vehicle square-on and 8 for 24x24 infantry. The Rocketman's examples, for 40 damage, are right.

## Implementation impact
- Rename the command label to "Rearm" on the Hornet's and Dragonfly's panels.
- Everything else is as built; remove the placeholder marks.

## Verification
1. A landed Hornet with enemies in range doesn't fire.
2. Press F (Rearm), then Shift+Attack Move. The aircraft lands, rearms fully, then leaves.
3. A plain order given during rearming makes it leave at once with partial ammo.
4. A Hornet flying over a landed one doesn't displace it.
