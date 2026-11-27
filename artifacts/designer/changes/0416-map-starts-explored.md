# 0416 — The map starts explored (default), set by a command-line flag

- **Date**: 2026-11-26T17:05:00-05:00
- **Answers**: —
- **Docs**: `core/vision.md > Starting exploration` (new)

## What changed and why
The user wants an option for the map to start explored, fog of war still on, and for that to be the **default**.
- On: every tile starts **Explored** for every player. Remembered at 0:00: terrain and **neutral objects as at game start** (crystal patches with their starting amounts, Supply Delivery Stations, etc.). **No player objects** are remembered (user chose a: bases must still be scouted).
- Off: the map starts Unexplored, as before.
- Set by a **command-line flag** for now; later in the game creation screen. Restart keeps it.

## Implementation impact
New startup option with the default **explored**; the current default (all Unexplored) becomes the opt-out. The minimap (0415) shows the same.

## Verification
- Start a game with no flag: the whole map shows terrain and crystal patches, dimmed; enemy bases are not shown until scouted; units only where you see.
- Start with the flag set to unexplored: the map is black outside your sight.
- Mine a remote patch an enemy can't see: the enemy still remembers its starting amount until it looks again.
- Restart from the F10 menu: the same setting applies.
