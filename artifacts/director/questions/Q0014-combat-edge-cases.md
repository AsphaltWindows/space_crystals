# Q0014: Combat edge cases the docs don't settle

- **Date**: 2026-11-23T19:10:00-05:00
- **Blocking**: no. Combat is built with the behaviour listed under each point, and each one is a single named rule in the code.
- **Docs**: `systems/controls/commands.md > Attack`, `systems/combat.md` (attack phases, DamageCalculation), `systems/units.md` (RuggedTerrainDefenseBonus)

## Question
Please confirm or correct each of these:
1. **Attacking your own or neutral objects.** The Attack command's target is "a specific destructible object instance", with no owner restriction. Current behaviour: an explicit Attack order may target your own units and structures and neutral destructible objects. Automatic target picking (coming with the behaviours slice) will never choose them. Is force-attacking your own units intended?
2. **A shot whose target dies first.** The hit lands at the end of the Firing phase. Current behaviour: if another unit kills the target during Firing, the shot hits nothing, and the attacker still goes through Cooldown and Reload. The alternative is that the hit lands at the start of Firing, so a shot that has begun always connects.
3. **Rugged bonus for a unit standing across several tiles.** Current behaviour: the tile under the unit's centre decides whether it gets the Rugged defense bonus.

## Context
These came up while building combat (M1-W). None blocks anything, but 1 and 2 change what the player sees in fights: being able to shoot your own units, and wasted shots when several units focus one target.

## Options
For each point: the current behaviour (A), or a different rule you give. For point 1, the main alternative (B) is that Attack only accepts enemy targets. For point 2, the main alternative (B) is that the hit lands at the start of Firing.
