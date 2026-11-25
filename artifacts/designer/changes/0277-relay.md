# 0277 — Relay (renamed from Refract): how Laser Batteries relay through Satellites

- **Date**: 2026-11-25T16:55:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery > Relay` (new), `> Deploy`, `> Satellite (name TBD)`

## What changed and why
User renamed **Refract → Relay** and specified it.

**Ordering:**
- Selected deployed Batteries choose Relay, which highlights the Satellites they can reach: any Satellite in a Battery's own range, plus all Satellites connected to those by contiguous chains.
- The player clicks the **target Satellite**.

**The relay:**
- Each Battery locks onto its nearest Satellite with its thin beam.
- The beams join as **one beam** along the chain to the target Satellite, which becomes **armed**: it auto-targets ground or air enemies in its range.
- It fires at about the Batteries' attack frequency, discharging them normally.
- Thin 0-damage beams show between surges at every link, including the armed Satellite → target.

**While relaying:**
- The Batteries are locked and can do nothing else.
- Their commands are Stop Relaying, Relay (re-pick), and Amplify (when the target Satellite has an active target), which spends each Battery's whole battery on the next surge.

Supersedes 0276's Refract wording.

## Implementation impact
Concept only. TBD:
- leg/link ranges;
- chain breaks;
- combined damage of several Batteries;
- Batteries with no Satellite in range;
- player-directed targeting;
- sight.

## Verification
—
