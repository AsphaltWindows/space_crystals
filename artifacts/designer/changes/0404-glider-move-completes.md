# 0404 — A glider's Move completes when it first reaches its point

- **Date**: 2026-11-27T14:15:00-05:00
- **Answers**: Q0148 (part 2)
- **Docs**: `systems/controls/command_behaviors.md > MovingToLocation > GliderMovement exception`

## What changed and why
User: **(b)**. A glider's MovingToLocation **completes when it first reaches the target location**. The next queued order then starts; with nothing queued, it circles there as when idle. This replaces "does not complete", so Shift-queued waypoints and Move-then-Attack-Move work for aircraft.

## Implementation impact
- Glider Moves complete on arrival.

## Verification
- Shift-queue Move A then Move B for a Sparrow: it passes A and flies on to B, then circles at B.
