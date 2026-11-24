# Q0034: Should another player's Extraction Plate show the crystals left in its patch?

- **Date**: 2026-11-24T01:03:47-05:00
- **Blocking**: no. M2-G shows the number to every viewer, enemies included.
- **Docs**: `factions/gdo/economy.md > ExtractionPlate` (its InfoPanel); change 0105 (other players' structures show name, owner and HP); change 0099 (the plate's InfoPanel shows the patch's remaining crystals).

## Question
When a player selects an **enemy** Extraction Plate, should its InfoPanel show how many Space Crystals are left in the patch under it?

## Context
The plate's InfoPanel section lists the remaining crystals without saying who may see them. 0105 gives other players' structures a reduced panel: name, owner and HP. M2-G read the plate section literally, so an enemy plate shows the count as well. That tells the player how long the enemy's mining will last. Uncovered patches show their crystals to everyone.

## Options
- **A** (current): everyone sees the count, as with an uncovered patch.
- **B**: only the plate's owner and allies see it; enemies get the reduced panel from 0105.
