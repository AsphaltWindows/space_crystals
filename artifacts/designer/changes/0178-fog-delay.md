# 0178 — Fog delay 0.5 s (smooths edge-of-sight chases)

- **Date**: 2026-11-24T22:55:00-05:00
- **Answers**: Q0018 (point 1)
- **Docs**: `core/vision.md > Visible > Fog delay` (new)

## What changed and why
The user accepted a 0.5 s grace period, then suggested a lingering-vision mechanic; agreed to make it the single rule. A tile that leaves all sight stays Visible for 0.5 s (8 frames, placeholder) before becoming Explored. Edge-of-sight targets no longer flicker, so Attack/Follow don't drop to last-seen position on one-tick gaps — without a separate held-target rule and without leaking info.

## Implementation impact
- Visibility: per tile (per player) a "last seen" timestamp; Visible if in sight now or within 8 frames.
- No separate grace period in AttackingObject/Follow (0034 logic unchanged, it just triggers less).
- Interacts with 0177's attacker reveal (independent timers).

## Verification
Chase an enemy at the edge of sight: no stutter. Walk a scout past hidden units: they stay visible ~0.5 s after the scout's sight leaves them.
