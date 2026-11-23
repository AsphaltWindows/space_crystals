# 0018 — Landed GDO aircraft are ground targets

- **Date**: 2026-11-23T14:49:02-05:00
- **Answers**: —
- **Docs**: `factions/gdo/structures.md > Airfield > Aircraft Support > Returning and rearming`

## What changed and why
User decision: while parked on its Airfield, an aircraft can be hit **only by anti-ground weapons**, i.e. TargetDomain Ground or Universal. Air-only weapons can't hit it. After take-off it's an air target again. Raiding an Airfield with ground forces can destroy parked aircraft, while enemy anti-air can't touch them on the pad.

## Implementation impact
Target-domain checks must treat a landed aircraft as Ground and an airborne one as Air. Still stubs overall; rearm and landing times are TBD.

## Verification
(For later.) A parked Hovercraft can be attacked by a Peacekeeper (Ground) and a Raptor (Universal), but not by an Air-only weapon. Once airborne, the Peacekeeper can no longer target it.
