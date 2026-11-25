# 0272 — Specter: attack orders when unarmed, and what happens to the queue

- **Date**: 2026-11-25T16:00:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Specter > Attack` (Attack orders and ammo)

## What changed and why
User:
- An **unarmed** Specter ordered to attack flies to the nearest own Tier 3 Tunnel to re-arm. **On entering, its orders are wiped**, and it stays inside.
- **After bombing:**
  - if the target is destroyed, the Attack completes and the queue continues; an empty queue means idle, and an unarmed idle Specter flies home (0271);
  - if the target survives, the Attack still stands. Being unarmed, it must re-arm, so it ignores later queued orders and flies home, where they are wiped.

Designer's readings, flagged to the user:
- after a kill, it flies queued orders unarmed;
- with no Tier 3 Tunnel, an unarmed Specter holding an Attack circles.

## Implementation impact
- Re-arming is part of carrying out an Attack when ammo = 0; it targets the nearest T3 Tunnel.
- Entering the network clears the order queue.
- This resolves the "queued orders: TBD" from 0271.

## Verification
- Attack + shift-Move: if the target dies, the Specter flies the Move and then home.
- If the target survives, it flies home, lands and stays inside; the Move is never flown.
- An unarmed Specter ordered to attack heads to the nearest Tier 3 Tunnel.
