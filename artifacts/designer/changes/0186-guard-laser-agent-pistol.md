# 0186 — Guard: laser, ground + air, needs Optics Lab; Agent: pistol

- **Date**: 2026-11-25T00:30:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Guard` (intro, Produced at, TechPrerequisite, AttackAttributes), `> Agent` (intro, AttackAttributes, commands, right-click); `factions/syndicate/structures.md > Hideout` (Produces, W: Build Guard), `> Optics Lab` (description)

## What changed and why
User redesign of the two Syndicate infantry:
- **Guard**: laser rifle, **TargetDomain Universal** (ground and air; fills the Syndicate's mobile anti-air gap), cannot miss (instant hit), not rapid-fire. **Requires an Optics Lab.** Placeholder numbers: 18 damage, Aim 4 / Fire 1 / Cooldown 3 / Reload 16 = one shot per 24 frames (≈ same 12 DPS as before). Range 3 GU, cost 125 / 7.5 s unchanged.
- **Agent**: **pistol** replaces melee: short range, low-moderate damage, moderate reload, **ground only**, overall weak. Placeholders: 5 damage, range 2 GU, Aim 3 / Fire 1 / Cooldown 2 / Reload 14 = 20 frames (4 DPS).
- Consequence: the Optics Lab becomes the Syndicate's army gate (its cost/time still TBD); before it only Agents fight.

## Implementation impact
- Guard can attack air (auto-targeting, explicit Attack, threat classification: Guards now threaten air units).
- Hideout W unavailable without an Optics Lab (unavailable-command feedback, 0162).
- Agent's attack becomes ranged; melee removed.

## Verification
- Without an Optics Lab, Hideout W is greyed out; after building one, Guards can be queued.
- Guards shoot down an aircraft; each shot lands instantly, ~1.5 s apart.
- An Agent shoots an enemy 2 tiles away for 5 damage per 1.25 s; can't target aircraft.
