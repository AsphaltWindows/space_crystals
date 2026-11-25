# 0292 — Supplies gathering: 8 su reaches, nearest Tunnel by crystal hatch, old load lost at the end

- **Date**: 2026-11-26T10:00:00-05:00
- **Answers**: Q0082
- **Docs**: `factions/syndicate/units/agent.md > Gathering > Supplies`

## What changed and why
User:
1. **A:** 8 su reaches for pick-up at a station and for drop-off at the Supplies hatch (placeholders).
2. **B:** "nearest Tunnel" for Supplies is measured to the **crystal hatch**, the same measure as for crystals. This is not the Director's proposal.
3. **A:** the old load is lost when the new pick-up or mining **ends**, both ways.

## Implementation impact
As proposed, except point 2: use the crystal-hatch distance.

## Verification
An Agent ordered off a station mid-pick-up still carries its crystals.
