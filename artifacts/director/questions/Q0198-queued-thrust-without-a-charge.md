# Q0198 — A Shift-queued thrust that starts with no charge left

- **Date**: 2026-11-28T04:25:48-05:00
- **Blocking**: no. Proceeding on (a):
  - the player can queue more thrusts than the Prism has charges left;
  - a queued thrust that starts with no charge left ends at once, without moving, and the queue goes on;
  - R greys out once every charge is spent.
- **Docs**: `factions/syndicate/units/prism.md > Movement`; change 0434 (point 2 and the Verification).

## Question
Can a player Shift-queue more thrusts than the Prism has charges left? If so, what does a queued thrust do when it starts with no charge?

## Context
- 0434 says: "Each queued thrust spends its charge when it starts", and "R + Shift-clicks on three points: three thrusts in a row, then R greys out."
- A Prism with three charges can be given four Shift-clicked thrusts before any of them has started. The fourth would start with no charge.

## Options
- **(a) Queue freely.** A thrust that starts with no charge ends at once, without moving, and the next queued order runs. R greys out when the last charge is spent. This is what will be built.
- **(b) The gate counts the thrusts already queued.** Once every charge left is spoken for, R greys out and a further Shift-click is refused.
  - A charge is still spent only when its thrust starts. So a queued thrust that a new order cancels keeps its charge.
