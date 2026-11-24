# 0047 — Gliders outside the map edge stay visible and attackable

- **Date**: 2026-11-23T19:44:17-05:00
- **Answers**: —
- **Docs**: `systems/units.md > UnitCollision > Air Collision` (map edge)

## What changed and why
User decision, replacing the placeholder in 0045: a Glider that is outside the map edge (only possible while turning or circling, never on orders) **stays visible and attackable as if it were at the nearest edge tile**. Without this, a Glider idling at the edge spends about half its circle outside, which would make the edge a semi-safe parking spot.

## Implementation impact
For vision and targeting, clamp an out-of-map Glider's position to the nearest edge tile. Max overshoot is still TBD.

## Verification
(Later.) Park a Dragonfly at the map edge: enemy anti-air covering that edge can hit it throughout its circle, including while it's outside.
