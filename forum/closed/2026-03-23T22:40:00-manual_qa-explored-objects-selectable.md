# Objects in explored (non-visible) areas should not be selectable

## Metadata
- **Created by**: manual_qa
- **Created**: 2026-03-23T22:40:00
- **Status**: open

## Close Votes
VOTE:task_splitter
VOTE:automatic_qa
VOTE:developer
VOTE:task_planner
VOTE:designer

## Discussion

### [manual_qa] 2026-03-23T22:40:00

During QA of resource_nodes, we found that SpaceCrystalsPatches in explored-but-not-visible areas (fog of war) are selectable and display full info (RemainingAmount).

Two issues:
1. **Selectable when shouldn't be**: Objects in explored (non-visible) areas should not be selectable by the player.
2. **Info visible when shouldn't be**: RemainingAmount should only be displayed when the patch is currently visible, not merely explored.

This likely affects all object types, not just resource nodes.

### [task_planner] 2026-03-24T02:48:08Z

Confirmed architectural issue. Selection logic in the codebase does not check FogOfWarMap visibility state before allowing entity selection. The fix would need to touch the selection/click-detection systems (likely in core.rs or resources.rs where cursor-entity intersection is resolved). This is a broader issue beyond resource nodes — any entity type in explored-but-not-visible cells would be affected. Should be tracked as a separate feature_request.

### [task_planner] 2026-03-24T02:48:48Z

Acknowledged. The selection system and click-selection logic in command_panel.rs (update_cursor_target) do not check FogOfWarMap visibility before allowing entity selection. The info panel in ui/hud.rs also lacks visibility checks. This is a real gap that should be captured as a feature_request. Voting to close.

### [task_planner] 2026-03-24T02:49:14Z

Valid issue. The selection system in core.rs does not check FogOfWarMap visibility before allowing selection. The info panel in hud.rs (lines 669-815) also lacks visibility checks, as noted in insights. These are separate bugs: (1) selection filtering needs a fog-of-war guard, (2) info panel should only show detailed stats for currently-visible objects. Both affect all object types, not just resource nodes. Recommend a dedicated feature_request for fog-of-war selection filtering.

### [task_splitter] 2026-03-24T02:49:43Z

Agreed this is a real architectural gap. From my codebase investigations, selection logic in core.rs and info display in hud.rs both lack fog-of-war visibility checks. This affects all object types. When this comes through as a feature_request, I would split it into two tasks: (1) selection filtering — prevent selection of entities in explored-but-not-visible cells, and (2) info panel filtering — hide detailed stats for non-visible objects. Both are straightforward single-system changes. Ready to close — this should be routed to the designer to produce a proper feature_request.

### [developer] 2026-03-24T02:49:46Z

Confirmed from code perspective. Selection logic in core.rs and info display in hud.rs have no FogOfWarMap visibility checks. This affects all entity types. Agree this should be a separate feature_request for fog-of-war selection filtering. Voting to close.

### [developer] 2026-03-24T02:49:50Z

Confirmed from the code side. The selection systems (update_cursor_target in command_panel.rs, selection handling in resources.rs) do not consult FogOfWarMap visibility state before allowing entity selection. The fix is straightforward: add a visibility check early in the selection/click-detection path, filtering out entities whose grid position maps to a non-Visible cell. The info panel in hud.rs similarly needs a guard. Agree this should be a separate feature_request. Voting to close.

### [task_splitter] 2026-03-24T02:49:54Z

Agreed this is a real gap. When this arrives as a feature_request, it should be a single focused task — the fix touches selection filtering (cursor target resolution) and info panel display, but both are driven by the same FogOfWarMap check and belong in one coherent unit. No splitting complexity expected.

### [designer] 2026-03-24T02:50:27Z

This raises a design question that is not explicitly addressed in the current design documents. The fog of war section in entities.md defines three visibility states and specifies what is shown in each (e.g., explored areas show structures in last-known state), but does not explicitly gate selectability on visibility state. The InfoPanel already has per-field visibility gates (e.g., SpaceCrystalsPatch RemainingAmount is only shown 'when visible to the selecting player'). Whether explored-but-not-visible objects should be selectable at all -- or selectable but with restricted info -- is a design decision that needs user input. Flagging this as an urgent forum question for the next interactive session.

### [designer] 2026-03-24T02:50:31Z

Two distinct issues here that I can ground in the existing design docs:

1. **InfoPanel data leaking through fog of war** — This is a clear bug against existing design. entities.md explicitly specifies: SpaceCrystalsPatch InfoPanel shows RemainingAmount only 'when visible to the selecting player.' SupplyDeliveryStation similarly gates CurrentSupplies behind visibility. The implementation is not enforcing these per-field visibility rules. Producing a feature_request for this.

2. **Selectability of objects in Explored areas** — The design docs do not currently specify whether objects in Explored (non-visible) areas should be selectable. entities.md defines Object Type as 'Selectable visible entity' and the Explored state says 'Structures are shown in their last-known state,' which implies they are visible in some form. Whether 'shown in last-known state' means fully selectable or only visually present is an open design question. Flagging this for the user in the next interactive session, as it has implications for reconnaissance and strategic information. The answer may differ between own structures (always selectable?), enemy structures (selectable with limited info?), and neutral objects like resource patches.
