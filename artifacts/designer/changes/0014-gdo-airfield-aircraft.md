# 0014 — GDO Airfield: limited-ammo aircraft, 4 aircraft per Airfield, two aircraft units

- **Date**: 2026-11-23T13:58:37-05:00
- **Answers**: —
- **Docs**: `factions/gdo/structures.md > Airfield` (Produces, Aircraft Support), `> Propulsion Labs`, `factions/gdo/units.md > Air-to-Ground Hovercraft (name TBD)`, `> Air-to-Air Fighter (name TBD)` (new stubs), `README.md`

## What changed and why
The user defined GDO's air force:
- **GDO aircraft carry limited ammo** and rearm at an Airfield, which is their landing and reloading zone. This is a new mechanic; no other faction has it yet.
- **Each Airfield supports at most 4 flying units.**
- **Air-to-Ground Hovercraft** (name TBD): HoverCraft base, air-to-ground missiles (TargetDomain Ground), produced at the Airfield with no extra prerequisite.
- **Air-to-Air Fighter** (name TBD): air-to-air missiles (TargetDomain Air), produced at the Airfield, requires Propulsion Labs. UnitBase not decided yet (HoverCraft or Glider).

## Implementation impact
Roster stubs only; not ready to build. Still to design: ammo counts, the rearm flow (automatic return when empty? landing? rearm time?), how aircraft are tied to an Airfield, what happens at capacity or when an Airfield dies, all stats and costs, and names. Follow-up entries will come as these are decided.

## Verification
None yet (design stub).
