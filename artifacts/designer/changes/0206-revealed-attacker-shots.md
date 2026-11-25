# 0206 — A revealed attacker's shots are drawn along their whole path

- **Date**: 2026-11-25T04:00:00-05:00
- **Answers**: Q0056
- **Docs**: `core/vision.md > Attacks from outside vision` (Shots are drawn where they travel through vision)

## What changed and why
User chose **B**: while an attacker in the fog is revealed to a player, that attacker's own shots are drawn from the attacker to the target, with no gap at the fog edge. This reveals nothing extra. Other shots from the fog are still clipped to Visible tiles. No ground around the attacker is revealed.

## Implementation impact
Client-only: shot clipping treats a revealed attacker's own shots as visible.

## Verification
Peacekeepers fire from the fog at a Bio Portal: each revealed Peacekeeper's tracer starts at the unit itself.
