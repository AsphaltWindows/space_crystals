# 0379 — A defeated player's neutral units keep their research, frozen at the defeat

- **Date**: 2026-11-27T10:10:00-05:00
- **Answers**: Q0142
- **Docs**: `core/game_rules.md > On Defeat`; `factions/cults/augmentation.md > Research applies to everyone`

## What changed and why
User: **(b)**. When a player is defeated, their completed research is frozen at that moment. It keeps applying to their former, now neutral, units. For example, an Enhanced Constitution Militant keeps its Level 1 amounts. This matches "Modes are kept as they were": nothing on the units changes because their owner lost.

## Implementation impact
- Replaces M5-N3's (a).
- Neutral units need a reference to the frozen research levels of their former owner (or a copy of them).
- This applies to every faction's research, not only Cults augmentation.

## Verification
- In a 3-player game, research Enhanced Constitution Level 1, augment a Militant, and get defeated.
- The neutral Militant keeps its MaxHP (+40) and HP.
