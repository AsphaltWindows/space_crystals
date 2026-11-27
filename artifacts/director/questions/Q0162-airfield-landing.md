# Q0162 — GDO aircraft at the Airfield: landing, take-off, new aircraft, rearming, and the Dragonfly

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GH1, the Airfield and the Hornet, has new Hornets appear over their Airfield. GH2, landing and rearming, builds the placeholders in Options, new aircraft appearing landed. GL, Propulsion Labs and the Dragonfly, lands the Dragonfly on point 5's placeholder.
- **Docs**: `factions/gdo/structures/airfield.md` (`> Assignment`, `> Production limit`, `> Landing slots`, `> Returning and rearming`, `> Placeholder stats`); `factions/gdo/units/{hornet,dragonfly}.md > Placeholder stats`; `factions/syndicate/units/specter.md`; `factions/cults/units/sparrow.md`; `factions/cults/structures/airstrip.md`; `systems/units.md > GliderMovement`, `> Glider`

## Question
1. During the 16-frame landing and take-off, is the aircraft an air target or a ground target?
2. Where does a newly built aircraft appear: landed on a free slot, or in the air? With no rally point, does it stay landed?
3. When its Airfield is destroyed, what happens to an aircraft landed on it?
4. Given an order while it rearms, does it take off with the ammo rearmed so far, or finish rearming first?
5. How does a Dragonfly, a Glider, land on a slot and take off?

## Context
- **Landed:** "Landed aircraft are ground targets": "while parked on the Airfield, an aircraft can be hit only by anti-ground weapons". "Once it takes off it is an air target again". "Landing and take-off: 16 frames (1 s) each".
  - The Specter, landing at a Tunnel's air door: "it can be hit during both, as an air unit".
- **New aircraft:** "An Airfield can't queue another aircraft if its assignments plus the aircraft already in its production queue would exceed 4", and it has "4 landing slots", so a new aircraft always has a free slot.
  - "After rearming, the aircraft stays landed at the Airfield."
  - The Cults' Sparrow is "placed directly on the runway and takes off along it toward C": it "appears over the runway's B end facing C at IdleSpeed and speeds up along the runway".
- **A destroyed Airfield:** "its aircraft become unassigned. An unassigned aircraft can't land to reload until the player reassigns it."
- **Rearming:** "Rearming an aircraft takes RearmTimePerAmmo × missing ammo". The Hornet rearms 5 missiles at 32 frames each (10 s in all), the Dragonfly 1 missile in 64 frames (4 s).
  - "The automatic return can be interrupted by giving the aircraft any other order".
- **The Dragonfly:** its UnitBase is "Glider (must keep moving; circles when idle; wider turns at higher speed)". Gliders "Cannot turn in place or stop."
  - The Specter's UnitBase names its exception: "Glider (with vertical lift-off and landing at the Tunnel air doors)". The Dragonfly's names none.
  - Its numbers: "IdleSpeed 6, MaxSpeed 14 su/frame, Acceleration 0.6, Deceleration 0.6", MaxCentripetalAcceleration 0.6. Since r = v²/a, it turns on a 60 su radius at IdleSpeed, and about 330 su at full speed. The Airfield is 3x3, 192 su a side.

## Options
1. **(a) An air target during both** (the placeholder), as the Specter is: anti-air can hit it until it has landed and from the moment it starts to take off. **(b) A ground target from the start of the landing to the end of the take-off.**
2. **(a) Landed on a free slot; it then takes off for its rally point, and with none it stays landed** (the placeholder). **(b) In the air over the Airfield,** as the Sparrow appears over its runway.
3. **(a) It stays landed where the Airfield was, unassigned** (the placeholder): a ground target until the player orders it off. **(b) It takes off at once,** and hovers (the Hornet) or circles (the Dragonfly) there.
4. **(a) It takes off with what it has** (the placeholder), as the return flight can be interrupted: a Hornet ordered off 5 s into its rearm leaves with 2 of its 5 missiles. **(b) It finishes rearming first,** and the order waits: up to 10 s for a Hornet.
5. **(a) Vertically, as the Specter at an air door** (the placeholder): it approaches its slot at IdleSpeed, stops over it while it lands in the 16 frames, and takes off vertically, leaving at IdleSpeed. **(b) Like a plane:** it touches down at IdleSpeed 6 and rolls to a stop, at its Deceleration 0.6 in 10 frames over about 30 su (half a GU). It takes off rolling, from a standstill to IdleSpeed in 10 frames over about 30 su.
