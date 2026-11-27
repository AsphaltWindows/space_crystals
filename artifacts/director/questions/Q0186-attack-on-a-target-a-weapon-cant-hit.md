# Q0186 — An explicit Attack on a target some units' weapons can't hit (the Twister on a ground unit)

- **Date**: 2026-11-27T14:31:10-05:00
- **Blocking**: no. Slice M6-GF built the reading in Context, and the Director is merging it.
- **Docs**: `systems/combat.md > ValidTarget` (point 3, TargetDomain); `systems/controls/commands.md > Attack`; `systems/controls/interface.md > AwaitingTarget` (Invalid target clicks); `factions/gdo/units/twister.md` ("TargetDomain: **Air** (anti-air only)"); design changes 0397, 0398 and 0409.

## Question
The Twister's machine gun hits air units only. When a selection is given an explicit Attack (A + click) on an enemy **ground** unit or a structure:
1. **A mixed selection** (Twisters with Raptors, say): do the Twisters follow the target with the group, as the units that can't attack do under 0398 and 0409, or do they keep their current orders?
2. **Twisters alone**: is the click an invalid target (ignored, with feedback, the targeting still waiting), or do they follow the target?

## Context
- `combat.md > ValidTarget` point 3 matches a weapon's TargetDomain to the target's domain. It is written for automatic targeting. `commands.md > Attack` doesn't say what an explicit Attack does with a target that a unit's weapon can't hit.
- 0397 and 0398 say an Attack goes to every selected unit, and "non-attackers treat Attack and Attack Move as Move: on a target they move to it". 0409 applies this to shielded Peacekeepers, which "follow the enemy with the group". The client builds it so: a unit that can't attack at all gets a Move on the target.
- A Twister can attack, but not this target. So it isn't a "non-attacker" as 0398 means it, and the docs don't settle which way it goes.
- **As built (M6-GF):**
  - **Mixed:** the Raptors attack the target. The Twisters are left out of the order and keep whatever they were doing, as units that can't target the ground ignore an Attack Ground.
  - **Alone:** the click is an invalid target (`interface.md > AwaitingTarget > Invalid target clicks`): "Invalid target", no order, and the Attack still awaits a target. The Director's plan asked for this ("An Attack on a Peacekeeper is refused, with feedback").
  - **A right-click** by Twisters on an enemy ground unit makes them follow it, as for any unit on a target it can't attack. This is unchanged.

## Options
1. **Mixed selection:**
   - (a) As built: the Twisters are left out and keep their orders.
   - (b) They follow the target with the group, as 0398's non-attackers and 0409's shielded Peacekeepers do. This is a small client change: such a unit gets a Move on the target, as a non-attacker does.
2. **Alone:**
   - (a) As built: an invalid target, with feedback.
   - (b) They follow it, as a right-click does.
- **The rule as built covers every unit, not only the Twister.** A ground-only weapon at an aircraft is the same case: Peacekeepers ordered to attack a Sparrow are now left out (mixed) or refused (alone). The answer would apply to all of them.
