# 0462 — The Pilgrim: Cults paratrooper glider (Airstrip, needs a Scrap Hangar)

- **Date**: 2026-11-27T15:30:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/pilgrim.md` (new)
  - `factions/cults/structures/airstrip.md > Production`, `> Placeholder stats` (Pilgrim cost, W: Build Pilgrim)
  - `factions/cults/structures/scrap_hangar.md > Unlocks`
  - `factions/cults/units/recruit.md > Repair` (a landed Pilgrim)
  - `README.md`, `factions/cults/units/README.md` (index)

## What changed and why
The user designed the Scrap Hangar's first aircraft, the **Pilgrim**: a big, unarmed **glider** that carries **16 infantry** and drops them by parachute anywhere on the map.

**On the runway:**
- A new Pilgrim **appears landed** on the runway and stays until it is given an order. Any order makes it take off along the runway.
- The player sends it to land by **right-clicking a specific own Airstrip**. There is no generic Land command, because with several Airstrips it matters where it is loaded and repaired.
- It lands at once. While it is on the runway, production carries on but a finished aircraft **waits at 100%**, as with a blocked exit.
- Recruits can repair it while it is landed.
- After a drop it **doesn't come back by itself**: the player may not want it landing while that Airstrip is busy producing.

**Loading:** only while landed. Any Cults infantry, Disciples' illusions included, are right-clicked onto it and walk over and board. **Unload All** puts everyone out along the side of the runway.

**Drop:** the player targets a spot. On reaching it, the Pilgrim rapidly unloads everyone. While parachuting down, passengers are **air targets** for anti-air.

**Destroyed:** everyone aboard dies.

**Cost:** **2 Recruits** (user), so Unit Control 2.

## Implementation impact
- **New unit.**
  - A glider that is the only aircraft which lands. It can land only on an own Airstrip, one Pilgrim per runway; one sent to an occupied runway circles over it (designer's reading).
  - A transport with Enter while landed, Unload All while landed, and a targeted Drop.
  - A new **falling** state for dropped infantry: an air target for 3 s (placeholder) that can't act. Each lands below where it jumped, or at the nearest walkable spot (designer's reading).
- **Airstrip changes.**
  - Runway occupancy by a landed Pilgrim blocks the release of finished aircraft at 100%.
  - New button: W Build Pilgrim, which needs a Scrap Hangar.
- **Placeholders** (designer):
  - Body: 96x96 su, 600 HP, armour 2/2, sight 8 GU, SeparationRadius 104.
  - GliderMovement: MaxSpeed 7, IdleSpeed 3, Acceleration and Deceleration 0.3, MaxCentripetalAcceleration 0.25, idle circle 2 GU.
  - Cost: 300 SC, 960 frames.
  - Drop: one passenger per 2 frames, fall 48 frames. Landing takes 32 frames.
  - Panel: Q Move, X Stop, E HoldPosition, R Drop, F Unload All.
  - Unload side: the long side to the left of the takeoff direction.

## Verification
1. With a Scrap Hangar, build a Pilgrim at an Airstrip. It appears landed and stays put, and a queued Sparrow finishing meanwhile waits at 100%.
2. Right-click the Pilgrim with 20 Militants. 16 walk over and board; 4 stay outside.
3. Press R and click a far spot. It takes off along the runway and the waiting Sparrow comes out. Over the spot, 16 parachutes come out in about 2 s and land 3 s later in a short line. Enemy anti-air can shoot them while they fall.
4. Afterwards the Pilgrim circles there. Right-click an Airstrip: it flies over, lines up, and lands.
5. Damage it, then have Recruits repair it while landed. Press F: anyone aboard comes out along the runway's side.
6. Shoot it down with passengers aboard: all of them die.
