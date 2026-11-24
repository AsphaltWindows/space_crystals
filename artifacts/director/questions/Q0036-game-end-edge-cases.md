# Q0036: End of game edge cases: simultaneous last defeats, free-for-all spectating, a raised Riot Shield on defeat

- **Date**: 2026-11-24T02:23:57-05:00
- **Blocking**: no. M2-F2 is built with the current behaviour listed under each point.
- **Docs**: `core/game_rules.md > End of Game`, `> On Defeat`; changes 0101 and 0102; `factions/gdo/units.md > Peacekeeper > Riot Shield`.

## Question
1. **The last two players or teams are defeated in the same tick**, for example by trading their last buildings. Nobody is left undefeated. What is the result?
   - **Current behaviour:** the game ends with no winner, and every player sees Defeat.
2. **A player defeated in a free-for-all game with 3 or more players and no teams.** `> End of Game` gives team games "their team's vision only". In a free-for-all the defeated player has no allies, so what do they see while the game continues?
   - **Current behaviour:** they keep watching with only their own explored map and remembered objects. No live vision is left, because their units are neutral and they own no buildings. They see a "Defeated: spectating" banner and can't command anything.
3. **Peacekeepers with the Riot Shield raised when their owner is defeated.** `> On Defeat` clears their orders and leaves them idle. The shield is a toggled state, not an order.
   - **Current behaviour:** the shield stays raised, and an attack already in progress finishes its shot or cooldown.

## Context
Defeat and the end of the game were built in M2-F2 from `core/game_rules.md` and changes 0101 and 0102. The docs don't cover these three cases. Points 1 and 2 only arise in rare finishes and in games with 3 or more players, and nothing is blocked.

## Options
- **Point 1:**
  - A, a draw: nobody wins, and everyone sees "Draw".
  - B, as now: everyone sees Defeat.
  - C, a tie-break rule you give.
- **Point 2:**
  - A, as now: own memories only.
  - B, the player leaves to a results screen and stops watching.
  - C, full map vision. Rejected for team games because the player could pass on information. In a free-for-all, the same risk applies against the other players.
- **Point 3:**
  - A, as now: the shield stays raised.
  - B, the shield is lowered when the unit becomes neutral.
