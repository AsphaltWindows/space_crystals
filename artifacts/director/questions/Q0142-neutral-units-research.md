# Q0142 — Does a defeated player's research still apply to their units once they are neutral?

- **Date**: 2026-11-26T07:29:56-05:00
- **Blocking**: no. M5-N3 (the Temple) proceeds with (a): neutral units lose the research, and HP follows the MaxHP rule.
- **Docs**: `core/game_rules.md > On Defeat` ("Remaining units become neutral"); `core/tech_and_research.md > Research > Shared Rules` (1); `factions/cults/augmentation.md > Research applies to everyone`, `> Increments > HP when MaxHP changes`

## Question
When a Cults player is defeated, their remaining units become neutral. Do the augmentation amounts of those units still follow the defeated player's Temple levels, or do they drop to the "No research" amounts?

## Context
- `core/game_rules.md > On Defeat`: "their remaining units stay on the map but **stop belonging to anyone** (neutral objects ...). Their orders are cleared and they stay idle where they are. **Modes are kept as they were**".
- `core/tech_and_research.md > Research`: research applies "to that player's eligible objects". Since 0336 a unit holds only which Augmentations it has, and its amounts come from its owner's current Temple levels.
- A neutral unit has no owner, so no levels. For example, an Enhanced Constitution Militant of a player with Level 1 has +40 MaxHP. Once neutral, it would read the "No research" amount, +20.
- This only matters in games of three or more players or teams, where the game goes on after a defeat. A neutral unit never attacks and stays idle, so in practice only its MaxHP and HP show.

## Options
- **(a) Neutral units lose the research** (built). They take the "No research" amounts, and HP follows the MaxHP rule: a decrease only caps HP, so nothing dies of it. This is the literal reading of "that player's eligible objects", and needs no new state.
- **(b) Neutral units keep the amounts they had at the defeat.** A defeated player can't research any more, so their levels are frozen. The code would keep the former owner's levels for their neutral units, which is a small amount of extra state. This reads "Modes are kept as they were" as covering research too.
