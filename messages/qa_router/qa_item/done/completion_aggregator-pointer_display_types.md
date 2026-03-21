# pointer_display_types

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# pointer_display_types

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added PointerDisplayType system to control_system.md. This defines 8 pointer/cursor display types that communicate to the player what action will occur on click:

- **Inactive**: Muted appearance. Shown when nothing is selected or hovering over an invalid target.
- **Move**: Default on ground, friendly objects, after Move hotkey, and for building rally points.
- **Attack**: Default on enemy units, after Attack hotkey on valid target.
- **AttackGround**: After AttackGround hotkey over ground.
- **Patrol**: After Patrol hotkey over ground.
- **GatherResources**: Default when resource gatherer hovers over resource, after Gather hotkey, and for Supply Tower ScheduleDeliveries AwaitingTarget.
- **ReturnResources**: Default when resource gatherer carrying resources hovers over drop-off point.
- **Enter**: Default when Syndicate unit hovers over own Tunnel (if entry possible).

Resolution rules are defined for three contexts:
1. **DefaultState** (right-click preview) — pointer previews what right-click would do based on RightClickResolution.
2. **AwaitingTarget** — pointer reflects the pending command and whether the CursorTarget is valid for it. Invalid targets show Inactive.
3. **AwaitingPlacement** — no pointer type; the building ghost preview serves as the cursor.

Key design decisions:
- Attack hotkey over ground shows Attack pointer (issues AttackMove), NOT AttackGround.
- Attack hotkey over friendly/neutral shows Inactive (no valid action).
- Inactive pointer is visually muted/subdued — not alarming, just clearly inactive.
- AwaitingTarget[ScheduleDeliveries] reuses GatherResources pointer.

Modified file: artifacts/designer/design/control_system.md (new PointerDisplayType section added after CursorTarget, before BasicCombatUnitInterfaceState).

## QA Instructions

1. Select a combat unit. Hover over empty ground — pointer should show **Move**.
2. Hover over an enemy unit — pointer should change to **Attack**.
3. Hover over a friendly unit — pointer should show **Move**.
4. Press the Attack hotkey (A). Hover over an enemy — pointer shows **Attack**. Hover over ground — pointer still shows **Attack** (will issue attack-move). Hover over a friendly unit — pointer shows **Inactive**.
5. Press Attack Ground hotkey (D). Hover over ground — pointer shows **AttackGround**. Hover over a unit — pointer shows **Inactive**.
6. Press Patrol hotkey (S). Hover over ground — pointer shows **Patrol**. Hover over a unit — pointer shows **Inactive**.
7. Press Escape to cancel. With nothing selected, pointer should show **Inactive** everywhere.
8. Select a resource gatherer. Hover over a resource node — pointer shows **GatherResources**. With resources being carried, hover over a drop-off point — pointer shows **ReturnResources**.
9. Select a Syndicate unit near an own Tunnel. Hover over the Tunnel — pointer shows **Enter**. Press Move hotkey (Q), hover over the Tunnel — pointer shows **Move** (explicit move overrides default enter).
10. Select a production building. Hover over ground — pointer shows **Move** (rally point).
11. Enter building placement mode — the building ghost should replace the pointer entirely (no separate pointer type visible).
12. Select a Supply Tower, enter ScheduleDeliveries. Hover over a valid target — pointer shows **GatherResources**. Hover over an invalid target — pointer shows **Inactive**.
