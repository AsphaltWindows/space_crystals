# 0430 — Shield drops on entering a Bunker; the Tent heals only units on the map, only once complete

- **Date**: 2026-11-26T19:20:00-05:00
- **Answers**: Q0164
- **Docs**: `factions/gdo/structures/bunker.md > Garrison` (No abilities from inside); `factions/gdo/structures/first_aid_tent.md > Placeholder stats`

## What changed and why
- **1 (b)**: a Peacekeeper with its shield up that enters a Bunker **lowers it** (the 6-frame toggle) and then fires from inside; it comes out unshielded.
- **2 (a)**: the Tent heals **only infantry on the map**, not units inside a building.
- **3 (a)**: a Tent under construction heals nobody.

## Implementation impact
1 replaces the placeholder (a). 2 and 3 match the placeholders.

## Verification
- Shielded Peacekeeper enters a Bunker next to enemies: after 6 frames it fires; eject it: it is unshielded.
- Wounded Rifle infantry in a Bunker beside a Tent don't heal; ejected, they heal at 2 HP/s.
- A Tent mid-construction next to wounded infantry: no healing until complete.
