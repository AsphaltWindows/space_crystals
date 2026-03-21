# underground_surface_walkability

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# underground-surface-walkability

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Fix: Underground expansions should NOT block surface movement. As defined in `artifacts/designer/design/syndicate_objects.md` under 'Tunnel Expansions':

> 'Underground expansions are placed spatially within the Tunnel Area on the grid, occupying cells just like surface buildings.'

And:

> 'Expansions are invisible to enemies without detection and can be walked over by surface units.'

Underground expansion structures exist on the underground layer. Surface units must be able to walk over the grid cells occupied by underground expansions. Only the Tunnel itself (a surface structure) blocks surface movement — its underground expansions do not.

**Note:** Some of this functionality may already be partially implemented. Downstream agents should check the current codebase state before implementing.

## QA Instructions

1. Build a Tunnel and construct underground expansions within its Tunnel Area.
2. Order a surface ground unit to walk through the grid cells occupied by underground expansions.
3. Verify the unit walks through without collision — the underground structures do not block surface movement.
4. Verify the Tunnel itself (4x4 surface structure) still correctly blocks surface movement.
5. Verify enemy units can also walk over underground expansions.
