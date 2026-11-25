# 0256 — Supplies gathering: shared stations, waiting at an empty station, switching resources loses the load

- **Date**: 2026-11-25T12:50:00-05:00
- **Answers**: Q0067
- **Docs**: `factions/syndicate/units.md > Agent > Gathering > Supplies`

## What changed and why
User:
- **Pts 1–2** as proposed: any number of Agents pick up at once; a Supply leaves the station when the pick-up ends, and only unclaimed Supplies can be picked up.
- **Pt 3:** when a station empties, the owner gets a **notification**, and all Agents there wait next to it indefinitely for the next delivery.
- **Pt 4:** an Agent carrying crystals ordered to gather Supplies goes straight to the station, and its crystals are **lost when it picks up the Supply**. This replaces the Director's deliver-first proposal. I assume the same the other way round.

## Implementation impact
- The notification needs a notification system (not designed; M4 may show it in a placeholder form).
- Waiting Agents keep their job.
- The carried load is discarded at the other resource's pick-up.

## Verification
- An Agent carrying 50 crystals, ordered onto a Supply station, picks up a Supply and no longer carries the crystals.
- When a station empties, the Agents stand beside it and resume when it refills.
