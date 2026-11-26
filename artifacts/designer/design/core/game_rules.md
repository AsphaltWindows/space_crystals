# Game Rules

## Defeat Condition
In a normal game, a player is **defeated when they own no buildings**. This applies to every faction. (Custom scenarios may define other objectives.) Faction notes:
- **Every structure counts**, for every faction — including GDO Extraction Plates, Colonists Beacons, and Syndicate Laser Turrets.
- **Structures under transformation count** (user, 2026-11-24): a Beacon under deployment and a building being warped in are structures from their first frame (factions/colonists/economy.md > Beacon Deployment > A Beacon from the first frame). If a player whose only building is a Beacon under deployment cancels it, they are **defeated in that same tick**; the cancel is not refused.
- **The Syndicate**: when a Tunnel is destroyed, all of that Tunnel's expansions become neutral (no longer owned by the Syndicate player), but its Laser Turrets stay Syndicate-owned (pop-up turrets become upright). So losing the last Tunnel defeats the Syndicate only if they own no Laser Turrets. See factions/syndicate/tunnels.md > Tunnel Network > Losing Tunnels.

## On Defeat
**Everyone learns of it** (user, 2026-11-26; Q0090): when a player is defeated while the game goes on, **all players** are told.

(user, 2026-11-24)
- **Remaining units become neutral.** When a player is defeated, their remaining units stay on the map but **stop belonging to anyone** (neutral objects, like the Syndicate expansions of a destroyed Tunnel). Their orders are cleared and they stay idle where they are. **Modes are kept as they were** (user, 2026-11-24): e.g. a Peacekeeper with its Riot Shield raised stays shielded (a mode isn't an order); an attack already in its Firing/Cooldown phase finishes.
- Being neutral, they **never attack** anyone and are **never auto-targeted** (systems/combat.md > ValidTarget; auto-targeting never picks neutral objects). Any player can still destroy them with an explicit Attack order (systems/controls/commands.md > Attack).
- They block movement like other units and never step aside (only own idle units yield), so they stay as obstacles until destroyed.
- A defeated player stays defeated: nothing they had can be commanded or build again.

## End of Game
(user, 2026-11-24) When only **one player or team** is left undefeated, it **wins**. Every player sees Victory or Defeat, the game stops, and the replay is saved.

**Draw** (user, 2026-11-24): if the last remaining players or teams are all defeated **in the same tick** (e.g. they destroy each other's last buildings simultaneously), nobody is left undefeated: the game ends as a **draw**. Nobody wins, and every player sees "Draw".

**Defeated before the end (team games)** (user, 2026-11-24): a player defeated while their team is still playing keeps watching with **their team's vision only** (what their allies see), never a full map reveal, so they can't pass on information their allies couldn't have. They can't command anything.

**Defeated before the end (no team left)** (user, 2026-11-24): a player defeated while the game continues and who has **no undefeated allies** (e.g. in a free-for-all of 3 or more) **leaves to a results screen** and stops watching; the game goes on for the others. (Rejected: watching with only their own map memory, which shows nothing live; full map vision, which could leak information to the remaining players.) Results screen contents: TBD.

**A defeated Syndicate player's units inside the Tunnel Network** (user, 2026-11-25; Q0069) are **destroyed**, not turned neutral: no exit remains.
