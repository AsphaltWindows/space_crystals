# 0389 — Recruits get an Attack button (E)

- **Date**: 2026-11-27T12:10:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/recruit.md > ObjectInterfaceState > DefaultState commands`, `> Unit Commands`

## What changed and why
User: "recruits should be able to attack (melee attack)".
- Recruits already had a melee attack (Damage 3, adjacent contact) and attacked by right-clicking enemies.
- Their panel had **no Attack button**: A is Basic Construct, as on the Agent's panel.
- **E: Attack** is added. The letter is the designer's placeholder; the left-hand slots are taken. It works as BasicCombatUnitInterfaceState's Attack:
  - left-click an enemy: attack it in melee;
  - left-click own or neutral: force-attack;
  - left-click ground: Attack Move.

## Implementation impact
- New panel slot E for Recruits, entering AwaitingTarget[Attack].
- If Recruits can't attack at all in the current build (no melee behavior or auto-targeting), that is now required.

## Verification
- Select Recruits, press E and click an enemy unit: they walk up and hit it for 3 per swing.
- Press E and click ground: they Attack Move and engage enemies on the way.
