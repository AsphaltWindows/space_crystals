# 0420 — HoverCraft aircraft use DragMovement: numbers for the Hornet, Phantom, Prism and drone

- **Date**: 2026-11-26T17:55:00-05:00
- **Answers**: Q0154
- **Docs**: Placeholder stats of `factions/gdo/units/hornet.md`, `factions/syndicate/units/phantom.md`, `factions/syndicate/units/prism.md`, `factions/gdo/units/drone_operator.md`

## What changed and why
Answered from the docs plus the user's standing permission on placeholder stats (option c). The HoverCraft base moves by DragMovement, and `systems/units.md > DragMovement` says air units have **high drag and high thrust**, relying on drag to stop. The TurnRate-style numbers in the 0366/0367 placeholders were a mistake of form; they are replaced by DragMovement numbers in the Supply Chopper's pattern (DragRatio ≈ 0.1, so ~65% of top speed in ~10 frames and a drift of ~1.4 GU from top speed).

| Unit | Forward | Omni | DragRatio | Top speed | TurnRate |
|---|---|---|---|---|---|
| Hornet | 0.8 | 0.1 | 0.1 | 9 | 8°/frame |
| Phantom ("relatively manoeuvrable") | 0.78 | 0.3 | 0.12 | 9 | 10°/frame |
| Prism (drift only) | 0.05 | 0.05 | 0.1 | 1 | 4°/frame |
| Operator's drone (if HoverCraft, Q0155) | 0.9 | 0.1 | 0.1 | 10 | 10°/frame |

The Prism's thrust charges stay separate bursts (Prism > Placeholder stats).

## Implementation impact
Replaces option (a)'s mapping that GK/SK registered. All placeholders.

## Verification
- A Hornet ordered to move reaches about two thirds of top speed within about 1 s of thrust and, on arrival, drifts to a stop over roughly 1.4 GU.
- A Phantom starts, stops and sidesteps visibly quicker than a Hornet.
- A Prism creeps at most 1 su/frame.
