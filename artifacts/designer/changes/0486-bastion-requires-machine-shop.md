# 0486 — The Bastion requires a Machine Shop

- **Date**: 2026-11-28T14:55:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/bastion.md` (TechPrerequisite)
  - `factions/cults/structures/chop_shop.md` (Production, cost line, panel R)
  - `factions/cults/structures/machine_shop.md`
  - `factions/cults/structures/README.md` (Machine Shop row)
  - indexes

## What changed and why
The user decided the Bastion needs a tech building: the **Machine Shop**, the vehicle-upgrade building beside the Chop Shop. This replaces 0485's "no TechPrerequisite beyond the Chop Shop".

## Implementation impact
- **R: Build Bastion** at the Chop Shop is greyed with "Requires a Machine Shop" until the player owns one.
- The usual unit-prerequisite rules apply: checked when queued, and units already queued finish if the Machine Shop is lost.

## Verification
- Without a Machine Shop, R is greyed and names the requirement.
- Build one, and the Bastion can be queued.
