# Q0090 — What a player perceives of events they don't see: an unseen shooter's colour, another player's defeat, and late-seen marks

- **Date**: 2026-11-25T13:23:15-05:00
- **Blocking**: no. Proceeding as built (below): option (a) for each point, marked `Placeholder (M4-V)` in `crates/game/src/viewer_events.rs`.
- **Docs**:
  - `core/vision.md > Attacks from outside vision` (0293: "every shot, tracer or projectile, whoever fires it and at whatever, is drawn only over your Visible tiles");
  - `core/game_rules.md > On Defeat`, `> End of Game`;
  - `systems/combat.md > AoEDamage`.

## Question
Slice M4-V put one per-player filter in front of everything the client draws from the game's events, so each player perceives only what the design lets them. Three cases are left that the docs don't settle:
1. **An unseen shooter's colour.** A shot's glow and a projectile's body are drawn in the shooter's player colour, over the viewer's Visible tiles. When the viewer doesn't see the shooter, should the colour still tell whose it is?
2. **Another player's defeat, before the game ends.** With three or more players, who learns that a player was defeated while the game goes on?
3. **Marks seen late.** Should a death marker or a blast mark show to a viewer whose tile turns Visible while the mark is still fading, even though they didn't see the event itself?

## Context
- **Point 1.** In a 1v1 the colour tells nothing new. With three or more players it tells which enemy fired, before the attacker is revealed: "only to the owners of what it damaged, and only if it is still alive" (`> Attacks from outside vision`).
- **Point 2.**
  - Nothing shows a defeat to the other players yet, but the filter lets every player have the event, so a later notice ("Player 3 was defeated") would reach everyone.
  - The docs say every player sees Victory, Defeat or Draw at the end, and that the defeated player's units turn neutral. Others notice that wherever they see those units.
- **Point 3.**
  - A death marker fades over 2.5 s, and a blast mark over 0.75 s (placeholder looks, Q0001).
  - Both are checked as they are drawn, so a unit that walks up and makes the tile Visible meanwhile sees the rest of the fade.
  - A shot or projectile is different: it shows only over tiles Visible as it flies.

## Options
1. **The unseen shooter's colour:**
   - (a) as built: always the shooter's player colour;
   - (b) a neutral colour when the viewer doesn't see the shooter, which the client can do on its own.
2. **Another player's defeat:**
   - (a) as built: every player learns of every defeat (no notice is shown yet);
   - (b) only the defeated player, and their team in team games. Others learn it only from what they see.
3. **Marks seen late:**
   - (a) as built: a mark shows wherever the viewer's tile is Visible during its fade;
   - (b) a mark only for events that happened on a tile the viewer saw at the time.

Each option is a small, client-only change.
