# 0461 — An Attack on a target some units' weapons can't hit

- **Date**: 2026-11-27T15:00:00-05:00
- **Answers**: Q0186
- **Docs**: `systems/controls/interface.md > SelectionGroup commands > Attack goes to everyone` (new bullet)

## What changed and why
1. **Mixed selection: option (b)** (user). A unit whose weapon can't hit the clicked target's domain is treated like a unit that can't attack: it **follows the target with the group** (a Move on that object). Examples are a Twister on a ground unit or structure, and Peacekeepers on a Sparrow. A group ordered together stays together, so the Twisters cover the Raptors.
2. **Alone: option (a), as built** (user). If **no** selected unit can attack the target, the click is an **invalid target**. It is ignored, the player gets feedback, and the Attack still waits for a target.

The rule covers every weapon domain, not only the Twister.

## Implementation impact
- **Changed (mixed):** units that can't hit the target now get a Move on it, instead of being left out of the order.
- **Unchanged (alone):** the invalid-target refusal.
- **Unchanged:** right-click behaviour.

## Verification
1. Select Twisters and Raptors, then A-click an enemy tank. The Raptors attack it and the Twisters follow it with them.
2. Select only Twisters and A-click the tank. You get "Invalid target", nothing is ordered, and the cursor still waits for a target.
3. Select Peacekeepers with Rocketmen and A-click a Sparrow. The Rocketmen attack it and the Peacekeepers follow.
