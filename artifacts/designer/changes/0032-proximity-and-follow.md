# 0032 — Move to a unit = Follow; proximity = 8 su edge to edge

- **Date**: 2026-11-23T19:08:12-05:00
- **Answers**: Q0009 (proximity part; silhouette rotation still open)
- **Docs**: `systems/controls/behaviors.md > MovingToObject > Proximity`, `> Completion` (rewritten); `systems/controls/commands.md > Move`

## What changed and why
User decisions:
- **Proximity** for moving to an object = gap ≤ **8 space units**, edge to edge (consistent with 0031).
- **Move targeting a unit = Follow.** This includes right-clicking an own unit. The follower doesn't complete on arrival: it waits while within proximity and follows again when the target moves away. It continues until a new order or until the target dies or leaves the map (enters a transport, building or Tunnel Network). The "leaves the map" ending is my resolution, flagged to the user.
- **Blocked near a followed unit:** stuck within 2 GU behind non-moving units counts as having reached it (wait, no re-routes).
- Move targeting a static object (structure, resource, station) still completes on reaching proximity.

## Implementation impact
- **New behavior:** Move with a unit TargetObject never completes on its own. Right-click on an own unit now issues a follow.
- Shift-queued orders after a follow won't run until the follow ends (like Patrol).
- Still open: whether the 2-GU blocked rule also applies to **static** objects (crystal patch, building) surrounded by a crowd; following an enemy unit via the Move hotkey when it leaves vision (ties to Q0010 chase-into-fog).

## Verification
- Right-click own Peacekeeper B with A selected: A walks to B and waits within ⅛ tile. Move B: A follows and keeps following until given another order.
- B enters a Supply Chopper: A's follow ends.
- Right-click a crystal patch: the unit walks to it and its order completes there.
