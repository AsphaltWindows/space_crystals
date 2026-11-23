# 0024 — "Fails to make progress" defined: stuck 1.5 s, 3 re-route tries, give up

- **Date**: 2026-11-23T18:42:44-05:00
- **Answers**: Q0008 (part 1; parts 2 unreachable target and 3 occupied target still open)
- **Docs**: `systems/controls/behaviors.md > MovingToLocation > Failure` (shared definition used by MovingToObject, AttackingObject, …)

## What changed and why
The user kept the Director's proposed placeholders: a unit is **stuck** after 1.5 s (24 frames) without getting closer. It then tries another route up to **3 times**, waiting 0.5 s, 1 s and 2 s before the attempts. After that it gives up (about 7 s worst case). These are placeholders to tune later.

## Implementation impact
Matches the Director's default; nothing to change. The values are now part of the design.

## Verification
- Box a unit in completely: it stops trying and its order completes after about 7 s.
- Block a unit briefly (under 1.5 s): it continues without re-routing.
