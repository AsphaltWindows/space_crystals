# tunnel-transit-light-infantry

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Clarified tunnel transit tier requirements in `artifacts/designer/design/syndicate_objects.md`.

The Tier 1+ Infantry category for Tunnel Network transit now explicitly includes both Light Infantry and Heavy Infantry unit bases. Previously the parenthetical only listed Heavy Infantry, which was ambiguous. The Enter command remains restricted to Syndicate faction units only, with the tier check being a separate validation against the unit's base category.

**Change**: Line 8 of syndicate_objects.md updated from:
- `Tier 1+: Infantry (Heavy Infantry)`
to:
- `Tier 1+: Infantry (Light Infantry, Heavy Infantry)`

## QA Instructions

1. Select a Syndicate unit with a LightInfantry unit base (if one exists in the current build).
2. Right-click on an own Tier 1 Tunnel.
3. **Expected**: The unit should show the Enter pointer and issue an Enter command, walking to Side A and entering the Tunnel Network.
4. Verify that non-Syndicate units (any faction) cannot enter tunnels regardless of unit base.
5. Verify that Vehicle-category units (Wheeled, Tracked, etc.) still cannot enter a Tier 1 Tunnel — they require Tier 2+.
6. Verify that Air-category units (Hover Craft, Glider) still cannot enter Tier 1 or Tier 2 Tunnels — they require Tier 3.
