# Q0153 — Tracked vehicles: the turn rate, turning in place beside infantry, and the Transport

- **Date**: 2026-11-26T18:26:57-05:00
- **Blocking**: no — tracked movement and crushing will be built in M5 for the Hand of Heaven (slices V3 and V4, after wheeled movement), and the Subterranean Transport in M6. Each proceeds on the placeholder marked in Options.
- **Docs**: `systems/units.md > SpeedTurnRadiusMovement`, `> UnitCollision > Crushing`, `> DrillUnit`; `factions/cults/units/hand_of_heaven.md`; `factions/syndicate/units/{subterranean_transport,laser_battery}.md > Placeholder stats`

## Question
1. Is a tracked unit's turning rate meant to be the same at every speed?
2. Can a crusher turning in place crush an adjacent crushable enemy, or is it blocked by it? (The doc's TBD.)
3. What are the Subterranean Transport's above-ground Acceleration, Deceleration and SpeedToTurnRadiusRatio?
4. Does the surfaced Transport crush?

## Context
- **SpeedTurnRadiusMovement:** "Turn radius increases with speed ... Small directional adjustments at speed cost little momentum, but sharp turns require significant slowing or a complete stop."
  - Its pathfinding note: "tracked units can reach the same place along many different paths by trading speed for turning rate (slow down for a sharp turn vs keep speed on a wide arc)".
- **The law as I read it:** turn radius = speed × SpeedToTurnRadiusRatio, with speed in su/frame and the radius in su, so the ratio counts frames.
  - Then the turning rate is speed ÷ radius = 1 ÷ ratio, whatever the speed.
  - Slowing makes a turn tighter, but not quicker. Speed is traded for room, not for turning rate.
- **At the placeholder numbers** (ratio 8 for every tracked unit; the game runs 16 frames a second; a GU is 64 su):
  - Every tracked unit turns about 7.2° a frame, so 90° takes about 13 frames (0.8 s), at full speed or spinning in place alike.
  - At full speed the Hand of Heaven (5 su/frame) turns on a 40 su radius, less than its own 64 su length. The Laser Battery (4.5) turns on 36 su, the Prospector and the Artillery Rig (4) on 32 su.
  - So a tracked unit would need to slow only where even that arc doesn't fit.
  - Under this law a larger ratio both widens the arcs and slows every turn, spinning included. Ratio 16 gives 3.6° a frame, 90° in about 25 frames (1.6 s), and an 80 su radius for the Hand of Heaven at full speed.
- **Crushing:** "TBD: whether a crusher turning in place (Stationary + Turning) can crush, or is blocked by, an adjacent crushable enemy". The Hand of Heaven will be the game's first crusher, in M5.
- **The Transport** is given only "MaxSpeed 4.5 su/frame" above ground. Its UnitBase is DrillUnit, which Crushing doesn't list ("TrackedVehicle, Mech"), though above ground it "behaves as tracked".

## Options
1. **Turning rate:**
   - **(a) The same at every speed, as the law gives** (the placeholder). The ratio sets both how wide the arcs are and how fast every turn is.
   - **(b)** The law while moving, plus a separate spin rate in place.
   - **(c)** A turning rate that falls with speed, so slowing does turn faster, as "trading speed for turning rate" suggests. For example: a set spin rate when stopped, falling to a lower rate at full speed.
2. **Turning in place beside an enemy crushable:**
   - **(a) It neither crushes nor pushes: the spin stops at contact** (the placeholder), as a stationary crusher is a hard obstacle.
   - **(b)** It crushes what its turning shape sweeps.
3. **The Transport's numbers:**
   - **(a) The Laser Battery's: Acceleration 0.4, Deceleration 1, ratio 8** (the placeholder).
   - **(b)** Numbers of its own.
4. **The surfaced Transport:**
   - **(a) It doesn't crush** (the placeholder), by the base list.
   - **(b)** It crushes, as a tracked vehicle above ground.
