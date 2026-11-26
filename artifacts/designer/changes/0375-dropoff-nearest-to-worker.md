# 0375 — Drop-off is chosen by distance from the worker when it finishes mining; ties go to the earlier id

- **Date**: 2026-11-27T09:35:00-05:00
- **Answers**: Q0136
- **Docs**:
  - `factions/cults/units/recruit.md > Gathering` ("Which Storage");
  - `factions/syndicate/units/agent.md > Gathering` ("Which Tunnel", crystals).

## What changed and why
User: the drop-off should be "the storage that the recruit (and agent) are closer to when they've finished mining, not the storage that's closer to the crystal"; if still equidistant, the earlier id wins.

- **Recruit**: at every return, the nearest own completed Storage **to the Recruit**, edge to edge, measured when it finishes mining. A tie goes to the earlier id. This **replaces 0345's "nearest from its patch"**. A Recruit mining an end patch from its own face's side is nearer that face's Storage, so it stays with its face.
- **Agent**: the Tunnel whose crystal hatch is nearest **to the Agent** (straight line) when it finishes mining. A tie goes to the earlier id.
  - The rest is unchanged: the Agent picks at the Gather's first return and keeps that Tunnel for the Gather.
  - The user is being asked whether Agents should re-pick at every return, as Recruits do.

## Implementation impact
- The Recruit's Storage choice changes its measuring point, from the patch to the Recruit.
- The Agent's measuring point is now explicitly the Agent at the end of mining.
- The tie-break stays the lower id, as built.
- The oracle's 8-adjacent-patch case (Q0136's throwaway measurement) is worth re-measuring.

## Verification
- On 8 adjacent patches with one Storage per face, Recruits on the end patch of a face deliver to their own face's Storage, not the neighbouring one.
