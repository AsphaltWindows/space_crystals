# 0273 — Specter with no Tier 3 Tunnel ignores Attack orders while unarmed

- **Date**: 2026-11-25T16:10:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Specter > Attack` (Attack orders and ammo)

## What changed and why
User confirmed my two readings in 0272, with one change:
1. After a kill, it flies the queued orders unarmed (confirmed).
2. With **no Tier 3 Tunnel**, an unarmed Specter **ignores Attack orders**: it has no ammo and no way to re-arm. The Attack is dropped and the next queued order runs; with nothing queued, it circles. This replaces 0272's "circles while holding the Attack".

## Implementation impact
- With ammo = 0 and no own T3 Tunnel, Attack orders are skipped.

## Verification
With no Tier 3 Tunnel, order an unarmed Specter to attack and then shift-Move: it skips the attack and flies the Move.
