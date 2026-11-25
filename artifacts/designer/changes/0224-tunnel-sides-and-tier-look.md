# 0224 — Tunnel: two paths on Side A, resource hatches, look by tier, air doors at Tier 3

- **Date**: 2026-11-25T07:00:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/tunnels.md > Tunnel` (sides; Look by tier)

## What changed and why
User's picture of the Tunnel:
- **Side A** has two paths, one for entering and one for exiting.
- **Sides B and C** are hatches for Space Crystals and Supplies. Each resource must go to its own hatch (an existing rule, now explicit).
- **Side D** has no function.
- **Look by tier:**
  - **T1:** narrow, shaggy paths (infantry only).
  - **T2:** wider, paved paths (vehicles).
  - **T3:** adds two horizontal circular sliding doors, one for flying units entering and one for flying units exiting. Aircraft use these, not Side A.

## Implementation impact
- Art direction for the three tiers.
- Gameplay: separate entry and exit paths on Side A (throughput rules TBD); air units enter and exit through the T3 doors (their exact position and rules TBD).

## Verification
—
