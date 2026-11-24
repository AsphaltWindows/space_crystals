# 0101 — On defeat, units become neutral; the game ends when one side remains

- **Date**: 2026-11-24T05:10:00-05:00
- **Answers**: Q0029 (points 1 and 2; Q0029 is now fully answered)
- **Docs**: `core/game_rules.md > On Defeat` (rewritten), `> End of Game` (new)

## What changed and why
**This replaces 0100.** The Designer misread the user's choice: the units don't stay owned and uncontrollable, they **become neutral**.
1. **On defeat**, the remaining units stay on the map but stop belonging to anyone, the same way the Syndicate's expansions turn neutral when their Tunnel dies:
   - Their orders are cleared, and they stay idle where they are.
   - They never attack anyone and are never auto-targeted.
   - Any player can still destroy them with an explicit Attack order.
   - They block movement and never step aside.
2. **End of game** (the user chose A): when only one player or team is left undefeated, it wins. Every player sees Victory or Defeat, the game stops, and the replay is saved.

## Implementation impact
- On defeat: clear every unit's queue and transfer ownership to neutral. Their vision no longer counts for anyone.
- **TBD:** team games, and what a player defeated before the end can still do. The proposal is to keep watching with their team's vision only. It's being asked now.

## Verification
1. Destroy all buildings of one player who still has Peacekeepers out. The Peacekeepers become neutral and idle.
2. Enemy units walk past them without shooting. An explicit Attack order on them works.
3. In a 1v1, the game shows Victory or Defeat and stops as soon as one player is defeated, and a replay is saved.
