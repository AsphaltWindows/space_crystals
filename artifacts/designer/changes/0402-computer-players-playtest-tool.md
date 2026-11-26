# 0402 — Computer players: a playtest tool for now, with fair knowledge

- **Date**: 2026-11-27T13:55:00-05:00
- **Answers**: Q0147
- **Docs**: `core/game_rules.md > Computer players (playtest tool)`

## What changed and why
User: 1 **fair**, 2 **(a)**.
- Computer players are a **playtest tool** for now, started from the command line. Their behaviour (the standard, rush and passive profiles, and build orders) is the Director's placeholder. The Designer will design them when they become a feature.
- **Fair knowledge:** a computer player sees only what its own objects see, plus remembered sightings (the fog rules), the map and the starting locations. It never sees the whole map.
- It uses the same commands and rules as a player.

## Implementation impact
- Build as planned. Its knowledge must go through the same vision and fog layer a player's client uses.

## Verification
- In an AI-vs-AI game, an AI doesn't attack a building it has never seen or scouted, apart from the known start locations.
