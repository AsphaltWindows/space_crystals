# 0414 — A temporary playtest menu: Restart and Quit (F10 and at game end)

- **Date**: 2026-11-26T16:45:00-05:00
- **Answers**: — (playtest feedback from the user)
- **Docs**: `core/game_rules.md > Playtest menu (temporary)` (new)

## What changed and why
The user asked for a restart option during playtests: "when the game ends or if I press F10 I would like restart (and quit) options. A simple temporary menu."
- Opens at **game end** (with Victory/Defeat/Draw) and on **F10**.
- **Restart**: the same map, slots, factions and computer players (profiles) the game was started with, from the beginning.
- **Quit**: closes the game.
- **Resume** (F10 only), or F10 again: closes it.
- Designer's placeholders: the game **pauses** while the F10 menu is open; no confirmation step.

## Implementation impact
New UI. It is temporary: a real in-game menu (Surrender, settings) comes later and will replace it. Whether a restarted game starts a new replay is the Director's call (each game should keep its own replay).

## Verification
- Press F10 mid-game: the menu appears and the game is paused (units frozen); F10 or Resume: play continues.
- Restart: the same map, factions and computer opponents start again from 0:00, starting bases as at launch.
- Quit: the game closes.
- Win or lose a game: the menu appears with the result, offering Restart and Quit.
