# Q0160 — Twister mines: what they count as, what sets them off, destroyed mines, and buying

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GM, the Twister's mines, builds the placeholders in Options.
- **Docs**: `factions/gdo/units/twister.md > Mines`, `> Placeholder stats`; `core/game_rules.md > Defeat Condition`; `factions/syndicate/units/subterranean_transport.md > Underground movement`; `factions/syndicate/units/shadebringer.md > Sludge (toggle)`; `systems/combat.md > AoEDamage`; change 0267

## Question
1. Is a mine a structure for the Defeat Condition? Can its owner select it?
2. Which enemy ground units set a mine off? In particular, does a HoverVehicle (the Colonists' Drone)?
3. Does a mine destroyed by damage explode?
4. Can Buy Mine be pressed again while a purchase runs, queueing the next?

## Context
- **What a mine is:** "a 16x16 su object, hidden underground", with "mine HP 40".
  - The Transport doc counts mines among underground structures: a burrowed Transport "passes through every underground structure (expansions of any player, a retracted pop-up Laser Turret, GDO mines)".
  - The Defeat Condition: "a player is defeated when they own no buildings". "Every structure counts".
  - 0267: "Detection will be designed after all faction units are done. Until then hidden things (expansions, pop-up turrets, burrowed DrillUnits, mines) can only be hurt by area damage over them".
- **The trigger:** "When an enemy unit — infantry or vehicle — enters it, the mine explodes after a short delay". "Air units neither trigger mines nor take damage from them".
  - Sludge, the other effect on the ground, spares hover units: "Not affected: hover units (HoverVehicle, HoverCraft) and air units." The Colonists' Drone is the only HoverVehicle so far.
- **Blasts:** "explosion Damage 200 total, AoERadius 64 su", and "A unit's damage share = Attack Damage × (unit overlap area / AoE area)".
  - So a blast deals a 16x16 mine wholly inside it at most about 4 (200 × 256 ÷ 12,868). It takes about 10 blasts to destroy a mine, so one blast can't start a chain.
  - Enemy area fire: about 5 from a Grenadier's grenade (100 over 40 su), so 8 grenades destroy a mine; about 6 from an Artillery Rig's shell (300 over 64 su), so 7 shells.
- **Buying:** "Further mines are purchased one at a time". "Buy Mine 25 Space Crystals, 80 frames (5 s), one at a time". The Twister carries up to 3.

## Options
1. **(a) Neither** (the placeholder): a mine isn't a structure for the Defeat Condition, and isn't selectable. **(b) It counts for the Defeat Condition:** a player whose last structures are mines isn't defeated, and since nothing detects mines yet, the enemy can only find them by area fire. **(c) Selectable by its owner,** to see it or remove it, but not counted for defeat.
2. **(a) Every enemy ground unit, HoverVehicles included** (the placeholder), since a HoverVehicle is a vehicle. **(b) HoverVehicles pass over safely,** as they pass over sludge.
3. **(a) No** (the placeholder): it is removed without a blast. **(b) Yes:** it explodes as if triggered, damaging what is near it, its owner's units included.
4. **(a) Refused** (the placeholder), as "one at a time" reads: the player presses F again after each 5 s purchase. **(b) Purchases queue up to the free stock:** three presses buy three mines in 15 s.
