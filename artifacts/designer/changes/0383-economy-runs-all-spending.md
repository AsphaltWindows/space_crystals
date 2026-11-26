# 0383 — Economy comparison: everything spent counts, Generators included

- **Date**: 2026-11-27T10:50:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/economy.md > Economy check` (scenario 2)

## What changed and why
User: Generators count as economy, and "we're not building anything other than economy in these experiments". So "spent on economy" is simply **everything spent** in the run. This replaces 0382's item list and its separate Generator line.

## Implementation impact
- Spent on economy = total crystals spent, net of refunds.
- If a scenario ever needs a non-economy building, report that separately.

## Verification
- Bank + spent = starting bank + total mined, at every checkpoint.
