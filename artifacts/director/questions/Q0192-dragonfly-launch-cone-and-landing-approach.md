# Q0192 — The Dragonfly: must it face its target exactly to launch a homing missile? And its landing approach

- **Date**: 2026-11-28T00:26:30-05:00
- **Blocking**: no. M6-GL built the Dragonfly on the readings below, marked as placeholders.
- **Docs**:
  - `factions/gdo/units/dragonfly.md > Placeholder stats` ("No turret (fires forward)"; "Attack: **HeadDisjointed** (homing air-to-air missile)"; MaxCentripetalAcceleration 0.6, "about 330 su turn radius at full speed");
  - `systems/controls/command_behaviors.md > AttackingObject` (point 2: a unit attacking from its base faces its target before it fires);
  - `factions/gdo/structures/airfield.md > Returning and rearming`.

## Question
1. **Does a homing missile need the Dragonfly to face its target exactly?** It has no turret and must keep moving, so it launches only while flying straight at its target. Against a Sparrow circling on its 1 GU idle circle, it can't turn tightly enough to point at it: in a test it launched nothing for 25 s, though head-on it fires at once. Since the missile homes anyway, may it launch within a cone around the Dragonfly's heading?
2. **Its landing approach** (placeholders):
   - (a) A Dragonfly flying home slows from MaxSpeed to IdleSpeed 4 GU before its slot.
   - (b) It lines up by flying straight on while the slot is within its tightest turn. If it overflies, it comes round again, and it lands facing the way it came.
   - (c) With no free slot, it circles over the Airfield's centre until one frees up.

## Context
M6-GL built the Dragonfly: Propulsion Labs, its production, the missile, and the vertical landing on the Airfield's slots.
- **Point 1.** Every unit that fires from its base (no turret) fires once it faces its target. The game's tolerance is 1°, which only absorbs rounding, because a unit that can turn in place turns onto the exact direction. A glider can't turn in place, so it fires only when its flight line points at the target. The Sparrow strafes this way with guns, which suits a weapon that fires along its heading. For a homing missile it means dogfights need head-on passes.
- **Point 2.** The doc gives the vertical landing on a slot, but not how the glider lines up for it, nor what it does when every slot is taken.

## Options
1. (a) Face it exactly (as built). (b) A launch cone for homing missiles from a turretless base, e.g. ±30°, ±45° or ±90°. Of today's units only the Dragonfly is affected: the other homing units, the Raptor and the Rocketman, have a turret or turn in place. (c) Something else, e.g. the Dragonfly flies out and comes back for a head-on pass.
2. (a) All three as built. (b) Different numbers or rules, e.g. it waits at its rally point when no slot is free.
