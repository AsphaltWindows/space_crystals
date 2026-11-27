# Q0182 — Four GDO numbers the docs don't give

- **Date**: 2026-11-27T09:26:24-05:00
- **Blocking**: no. M6-GK registered the GDO's kinds with the assumptions below, marked placeholders. None of these kinds is live yet: each goes live in its own slice (GH1 the Hornet, GS the Supply Chopper, GP the Outpost, GM the mine, GF the Raptor and the Twister, GH2 the Dragonfly).
- **Docs**:
  - `factions/gdo/units/supply_chopper.md`;
  - `factions/gdo/structures/outpost.md > Placeholder stats`;
  - `factions/gdo/units/twister.md > Placeholder stats` (the mine);
  - `factions/gdo/units/hornet.md`, `raptor.md`, `twister.md` and `dragonfly.md`;
  - `systems/combat.md > TargetType`, `> DoublyDisjointed` and `> TailDisjointed`;
  - design change 0445 (the Rocketman's missile bursts).

## Question
1. **The Supply Chopper's UnitControlCost:** what is it?
2. **The Outpost's Groupable:** true or false?
3. **The mine's SightRange and armour:** what are they?
4. **The TargetType** of the Raptor, the Twister, the Hornet and the Dragonfly: SingleTarget, or AoE with a radius? The Hornet's case is the open one.

## Context
1. **The Supply Chopper:** its doc gives a Silhouette (60x60) and MaxHP (150), but no UnitControlCost. Every other unit's doc gives one. GK assumed **2**.
2. **The Outpost:** its placeholder stats give Size, symmetry, HP, armour, sight, BuildRadiusExtension and Power, but not Groupable. Every other GDO building's stats give it: true for all but the Deployment Center, which is false. GK assumed **true**.
3. **The mine:** the Twister's doc gives HP 40 and "a 16x16 su object, hidden underground", but no SightRange or armour. GK assumed **SightRange 0 and armour 0/0**.
4. **The TargetType:** none of the four docs gives one. GK assumed **SingleTarget**, since no doc gives an AoERadius.
   - **This is plain for three of them.** The Raptor's and the Dragonfly's missiles are homing (HeadDisjointed), and land on their target. The Twister's machine gun is FullyConnected.
   - **The Hornet's missile is DoublyDisjointed** (user, 2026-11-27): "not homing: the missile flies to where the target was, so it can be dodged; it can Attack Ground". The Rocketman's missile was the same, and 0445 made it burst at the spot (AoE), so that the spot rule needed no answer.
   - **As SingleTarget,** the Hornet would be the only unit in the game that fires a single-target projectile that can miss. Which object it hits at the spot isn't in the design. The code holds a placeholder: TailDisjointed's rule (Q0126), "whatever object (unit or structure) covers the locked point is hit; if several do, the one the shot was aimed at".

## Options
1. The Supply Chopper's UnitControlCost:
   - **(a) 2**, the assumption;
   - **(b) another number.**
2. The Outpost's Groupable:
   - **(a) true,** as the other GDO buildings (the assumption);
   - **(b) false,** as the Deployment Center: each Outpost then gets its own selection group.
3. The mine:
   - **(a) SightRange 0 and armour 0/0** (the assumption). It sees nothing, and takes full damage;
   - **(b) other numbers.**
4. The Hornet's missile:
   - **(a) SingleTarget,** hitting by TailDisjointed's rule (the assumption);
   - **(b) AoE, like the Rocketman's:** it bursts at the spot (0445). It needs an AoERadius, and its burst takes the domain of what it was fired at;
   - **(c) SingleTarget, by another rule** for what it hits at the spot.

   For the Raptor, the Dragonfly and the Twister, SingleTarget seems the only reading. Correct it if not.
