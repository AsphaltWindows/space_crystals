# 0377 — A Garrison counts as a threat while an occupant could attack the scanner

- **Date**: 2026-11-27T09:50:00-05:00
- **Answers**: Q0139
- **Docs**: `systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` (also used by BaseAutoTargeting); `factions/cults/structures/garrison.md`

## What changed and why
User: **(b)**. In target scans, a Garrison is a **threatening** target while any occupant's weapon, as fired from inside, could attack the scanner's domain. This includes the Militant stand-in for Recruits and Disciples. An empty Garrison, or one with no weapon that can reach the scanner's domain, is not a threat. Enemies no longer ignore a Garrison that is shooting them.

## Implementation impact
- Replaces M5-N2's "never a threat".
- The threat test for a Garrison reads its occupants' weapons (domain as fired from inside).

## Verification
- An enemy Peacekeeper sees an armed enemy unit that can't hit it, and a Garrison with a Militant inside.
- It targets the Garrison first.
- Empty the Garrison: it is no longer preferred.
