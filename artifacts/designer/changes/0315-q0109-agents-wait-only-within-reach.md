# 0315 — Agents wait only within reach of the hatch; otherwise they keep trying (first step for Q0109)

- **Date**: 2026-11-26T15:15:00-05:00
- **Answers**: Q0109 (first step; please measure and report back)
- **Docs**: `factions/syndicate/units/agent.md > Gathering > Gathering rules` (Waiting at the hatch)

## What changed and why
The user's reasoning: an Agent that isn't in range to deliver shouldn't be "waiting". It should still be actively trying to reach the hatch, sidestepping on the way.

**New rule:** an Agent waits only once it is within reach of the hatch. Until then it is never "arrived": it keeps moving toward the hatch and steering around blockers. It is exempt from the occupied-target arrival rule.

This replaces "waits within reach (or as close as it can get)". It is close to the Director's option (b).

The user chose to try this rule **alone first**. Option (a), waiting Agents yielding like idle units, was offered as a backstop and is **not** adopted for now.

## Implementation impact
- Agent hatch legs no longer complete as "arrived" when out of reach.
- **Please re-run the Q0109 measurements**, especially the north-west cases and 5–6 Agents at 2 GU, and file a follow-up if any case still gridlocks. The user will then consider adding (a).

## Verification
In `dev_skirmish`'s soak with 4–6 Agents, no Agent should go 60 s without delivering.
