# 0060 — Direct hits land at the end of Firing; overkill wastes the shot and the cycle

- **Date**: 2026-11-23T20:28:48-05:00
- **Answers**: Q0014 (point 2; point 3 still open)
- **Docs**: `systems/combat.md > FullyConnected` (hit timing and overkill), `systems/combat.md > ValidTarget` (note on explicit Attack, from 0059)

## What changed and why
Option A, as built. For FullyConnected attacks the hit lands at the **end** of Firing. If the target died first, the shot hits nothing and the attacker still does Cooldown and Reloading. Overkill is intentionally costly, which rewards spreading fire.

## Implementation impact
Matches the current behaviour.

## Verification
- 10 Peacekeepers focus a nearly dead enemy: one shot kills it, the other in-progress shots do nothing, and those shooters go through their full cooldown and reload before acquiring new targets.
