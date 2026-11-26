# 0372 — A Recruit holding a Build queued counts as one of the order's own Recruits

- **Date**: 2026-11-27T09:10:00-05:00
- **Answers**: Q0131
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics > Group construction rules`

## What changed and why
User: **(a)**. A Recruit that holds this Build queued behind another order (a Shift chain) and stands on the footprint makes the site **wait**; it does not fail the order. A player's own Shift chain shouldn't cancel itself because two buildings were placed close together.

## Implementation impact
None; this is as built in M5-J1. Remove the placeholder mark.

## Verification
- With 2 Recruits, Build a Storage, then Shift-place a Hall whose footprint covers one Recruit's work spot at the Storage.
- The Hall site waits, and starts once that Recruit leaves the footprint.
