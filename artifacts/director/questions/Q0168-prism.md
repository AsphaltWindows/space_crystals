# Q0168 — The Prism: its panel, aiming a thrust, and orders while it launches

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — SP (the Orbital Command and the Prism) and SR (the Laser Battery's Relay) build the placeholders in Options.
- **Docs**: `factions/syndicate/units/prism.md`; `factions/syndicate/units/laser_battery.md > Relay (ability, deployed only)`; `systems/controls/interface.md > Standard Slot Assignments`, `> BasicCombatUnitInterfaceState`; `factions/gdo/units/artillery_rig.md` (orders while deploying); `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`

## Question
1. What is the Prism's panel, armed and unarmed?
2. How is a thrust aimed, and can thrusts be queued?
3. Are orders given while it launches and unfolds held until it has unfolded?

## Context
- **The doc gives no panel.** The armed Prism can be given "a specific target (Attack)".
  - "A: Attack, on every unit"; "A is shown whenever at least one selected group can attack".
  - Units without an attack: "Panel: Q Move, X Stop, R: Burrow / Surface, F: Unload All." (the Transport); "Panel: Q Move, X Stop, R: Deploy into Outpost" (the Prospector).
  - Combat units use BasicCombatUnitInterfaceState: Q Move, E HoldPosition, A Attack, S Patrol and X Stop, among others.
- **Thrust:** "Directional thrust: a few charges, each giving a quick burst of displacement in a chosen direction". "Thrust charges: 3; each moves it 3 grid units over 1 s in a chosen direction." "Charges never come back".
  - Otherwise it drifts at "MaxSpeed 1 su/frame", 0.25 GU a second. One thrust covers as much as 12 s of drifting, and the three make 9 GU in all.
- **Launching:** "Launch 32 frames (straight up) and transform 32 frames (vulnerable during both)": 4 s in all.
  - The vehicle deploy model holds orders: "the last order given (without Shift; each new one replaces the previous) is held, and it is the first thing the unit tries once deployment finishes".
  - The Colonists' transformations take none: "A Beacon under deployment and a building under Warp In show only the Cancel command".

## Options
1. **(a) Q Move, X Stop and R Thrust, with A Attack while armed** (the placeholder), as the Transport's and the Prospector's panels. **(b) BasicCombatUnitInterfaceState plus R Thrust,** as the Twister's "BasicCombatUnitInterfaceState plus R: Lay Mine": it adds Hold Position and Patrol, for a unit that drifts at 1 su/frame.
2. **(a) Targeted** (the placeholder): the player clicks a point, and the Prism thrusts toward it, 3 grid units whatever the distance. Not queueable. **(b) The same, but queueable with Shift,** so the player can chain thrusts along a path.
3. **(a) Held,** as in the vehicle deploy model (the placeholder): an order given during the 4 s is carried out once it has unfolded. **(b) Refused until it has unfolded,** as the Colonists' transformations: the Prism, which can't cancel its launch, has an empty panel for 4 s.
