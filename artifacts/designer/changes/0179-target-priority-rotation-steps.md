# 0179 — Target priority: rotation in 45° steps, then distance

- **Date**: 2026-11-24T23:05:00-05:00
- **Answers**: Q0018 (point 2)
- **Docs**: `systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority`, `> Algorithm` (BaseAutoTargeting uses the same)

## What changed and why
Option B. Required rotation (0–180°, either direction) is bucketed into 45° steps (placeholder); within the lowest step, closest wins; then a fixed deterministic order.

## Implementation impact
Replace exact-angle comparison with step index. Applies to turret scanning and base auto-targeting (unit facing for infantry).

## Verification
Enemy A at max range dead ahead, enemy B adjacent at 5°: the unit shoots B first. Enemy C adjacent at 60° vs A ahead: A first.
