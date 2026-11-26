# Q0130 — Placing a Cults building where the order's own Recruits stand: warn, refuse, or wait for them?

- **Date**: 2026-11-25T22:30:31-05:00
- **Blocking**: no — proceeding on assumption: (c), no warning. The ghost judges the ground and other units as for every builder, and the failed order is refunded as 0339 says.
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics > Group construction rules` (Blocked site); entry 0339; Q0097 point 3

## Question
0339 makes Recruits of the order standing on the footprint block the site, and the whole order fails. Should the placement ghost warn of that, refuse placement, or should the site wait for its own Recruits to step off?

## Context
- **M5-E built 0339 as designed:**
  - Each Recruit walks to the part of the footprint closest to it.
  - The site starts when the first one arrives.
  - Any unit on the footprint at that moment, the order's own Recruits included, fails the whole order, refunded once.
- **It found the common case fails.** A player selects a group of Recruits and places a building right where they stand. The Recruits beside the footprint reach the wall at once, while those standing on it are still walking off. So the order fails immediately, with a refund, and nothing on screen warned the player.
- **Other units on the footprint** are visible to the player, so they are a fair blocker. The order's own Recruits leave anyway, so they only block because of the timing.

## Options
- **(a) Refuse:** the ghost is red while any selected Recruit stands on the footprint, as for other blockers. The player moves them first.
- **(b) Warn:** placement is allowed, but the Recruits on the footprint are highlighted on the ghost as blockers.
- **(c) No warning** (as built): the refund and the failure message are enough.
- **(d) Wait for them:** the site waits until the order's own Recruits have stepped off before it starts. Other units on the footprint still fail it. This reverses only the "including Recruits of the same order" part of 0339.

Proposed: (d). It removes the accidental failure in the most common placement, and keeps 0339's rule for every other blocker. If 0339 should stand as it is, (a).
