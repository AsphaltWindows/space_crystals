# 0346 — Cults economy check: edge-to-edge, ±15%, the ideal scenario plus a realistic opening

- **Date**: 2026-11-26T22:15:00-05:00
- **Answers**: Q0103
- **Docs**: `factions/cults/economy.md > Economy check` (new)

## What changed and why
User:
- **Geometry:** edge to edge, in whole tiles (default).
- **Tolerance:** ±15% (default).
- **Targets:** all the doc's figures, with **one Storage per face** for the ideal comparison.
- **A second scenario, the user's build line:**
  1. Start with the one Storage.
  2. Save for a second Hall **without extra Storages**, spreading new Recruits to farther patches.
  3. Build the second Hall with **all the non-optimal Recruits**.
  4. Then build Storages around the patches, one per face.
  5. Measure against the GDO and the Colonists at 1:30 and 2:48.

The user asked whether the Cults stay level when they expand early. The doc's "level until 1:30" figure was a **pure-mining** best case, so scenario 2 answers that. It has no target yet: report its numbers.

## Implementation impact
- The oracle grows a second scenario.
- Where the second Hall goes, and how the far Recruits are assigned, is your choice within the description.

## Verification
- The oracle reports both scenarios.
- Scenario 1 is pass or fail at ±15%.
- Scenario 2 gives numbers for the user.
