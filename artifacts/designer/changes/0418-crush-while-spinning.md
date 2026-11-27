# 0418 — A crusher turning in place crushes what it sweeps

- **Date**: 2026-11-26T17:40:00-05:00
- **Answers**: Q0153 part 2 (part 4 follows)
- **Docs**: `systems/units.md > UnitCollision > Crushing` (replaces the TBD)

## What changed and why
The user chose (b): a crusher spinning in place crushes the crushable enemies its turning Silhouette sweeps over, so a single infantry unit at a tank's corner can't stop it turning. Own and allied crushable units still block the spin (no friendly crushing). Stationary and not turning: still a hard obstacle.

## Implementation impact
Replaces the placeholder (a) "spin stops at contact" for enemies. Crush trigger becomes: Silhouette overlap while Moving, Reversing **or Stationary + Turning**.

## Verification
- Put an enemy Light Infantry against a stopped Hand of Heaven's side, then order the tank to face it: the tank turns and the infantry dies when the hull sweeps it.
- Same with an own Recruit: the tank stops turning at contact; nobody dies.
- A stopped, non-turning tank: enemy infantry can't walk into it.
