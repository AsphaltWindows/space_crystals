# 0362 — Q0130: the site waits for its own Recruits to step off the footprint

- **Date**: 2026-11-27T01:10:00-05:00
- **Answers**: Q0130
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics > Group construction rules`

## What changed and why
User: "Either d or a. If d isn't too challenging, I'd go with that."
- **(d):** Recruits of the same order standing on the footprint don't fail it. The site waits until they have stepped off, then starts.
- Other units on the footprint still fail the whole order.
- This replaces "including Recruits of the same order" in 0339.
- **Fallback (a), if (d) is costly:** the ghost is red while any selected Recruit stands on the footprint. Tell us if you take the fallback.

## Implementation impact
- The site start is gated on the order's own Recruits leaving the footprint.

## Verification
- Place a building under a group of Recruits: they walk off, and construction starts with no failure.
- An enemy standing there fails it with a refund.
