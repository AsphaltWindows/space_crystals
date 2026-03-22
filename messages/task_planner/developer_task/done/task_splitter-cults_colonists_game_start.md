# cults_colonists_game_start

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-factions_resources_r1.md

## Task

Make The Cults and Colonists factions selectable and startable so their HUD resource displays can be verified.

### Changes needed:

1. **menu.rs**: Add TheCults and Colonists to AVAILABLE_FACTIONS (change from [FactionEnum; 2] to [FactionEnum; 4], adding FactionEnum::TheCults and FactionEnum::Colonists).

2. **faction.rs — setup_player_resources()**: Currently hardcoded for 2-player GDO vs Syndicate binary. Expand to handle all 4 factions:
   - When SelectedFaction is TheCults: spawn a Player entity with CultsPlayerResources (default: space_crystals=500, unit_control_used=0, unit_control_available=0). The opponent should be a reasonable default (e.g., GDO player 1).
   - When SelectedFaction is Colonists: spawn a Player entity with ColonistsPlayerResources (default: space_crystals=500, alloys=50, essence=50, conduits=10, beacon_capacity_used=0, beacon_capacity_provided=20). The opponent should be a reasonable default.
   - Must spawn the faction entity with InvisibleEntity + FactionEnum + DisplayHud, and the player entity with InvisibleEntity + Player + DisplayHudInfo + faction-specific resources component.

3. **faction.rs — game start**: Create stub functions setup_cults_game_start() and setup_colonists_game_start() that log a message but don't spawn structures (no structures are designed yet for these factions). Register them in the Startup schedule in world/mod.rs alongside the existing game start functions, gated on the appropriate SelectedFaction.

4. **HUD already works**: The update_resource_bar_system in hud.rs already queries CultsPlayerResources and ColonistsPlayerResources and updates all the correct fields. No HUD changes needed.

### Verification:
- Select TheCults from faction menu, start game. HUD should show 'SC: 500' and 'UC: 0 / 0'.
- Select Colonists from faction menu, start game. HUD should show 'SC: 500', 'Alloys: 50', 'Essence: 50', 'Conduits: 10', 'BC: 0 / 20'.
- Existing GDO and Syndicate game starts must still work correctly.
