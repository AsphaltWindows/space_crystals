# cults_colonists_game_start

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to modify:

1. **`artifacts/developer/src/ui/menu.rs`** (lines 21-24):
   - Change `const AVAILABLE_FACTIONS: [FactionEnum; 2]` to `[FactionEnum; 4]`
   - Add `FactionEnum::TheCults` and `FactionEnum::Colonists` to the array
   - Update test `available_factions_constant_has_two_entries` (line 282-284) to expect 4
   - Update tests `cults_is_not_available` (line 267) and `colonists_is_not_available` (line 272) — they should now assert `true` (or be removed/inverted)

2. **`artifacts/developer/src/game/world/faction.rs`** (lines 17-63):
   - `setup_player_resources()`: Currently hardcodes GDO vs Syndicate binary with a 2-player if/else on line 22-26. Must expand to handle 4 factions:
     - Always spawn the selected faction as player 0 (local), pick a default opponent as player 1
     - For TheCults: spawn faction entity with `(InvisibleEntity, FactionEnum::TheCults, DisplayHud::new(FactionEnum::TheCults))` and player entity with `(InvisibleEntity, Player::new("Player 1", FactionEnum::TheCults, cults_player_id), DisplayHudInfo::new(FactionEnum::TheCults), CultsPlayerResources::default())`
     - For Colonists: same pattern with `ColonistsPlayerResources::default()`
     - Also spawn an opponent. Current code always spawns both GDO and Syndicate. A reasonable approach: always spawn a GDO opponent (player 1) when selecting Cults/Colonists. Or conditionally spawn just the selected + one opponent.
   - **Important**: `CultsPlayerResources::default()` gives `space_crystals: 500, unit_control_used: 0, unit_control_available: 0` (defined at `game/types/factions.rs` line 180-187)
   - **Important**: `ColonistsPlayerResources::default()` gives `space_crystals: 500, alloys: 50, essence: 50, conduits: 0, beacon_capacity_provided: 20, beacon_capacity_used: 0` (defined at `game/types/factions.rs` line 214-224). NOTE: The task description says conduits=10, but the Default impl has conduits=0. Use the Default impl (`::default()`) and let the types be the source of truth.
   - Types `DisplayHud`, `DisplayHudInfo`, `Player`, `GdoPlayerResources`, `SyndicatePlayerResources`, `CultsPlayerResources`, `ColonistsPlayerResources` are all in `crate::game::types::*` (already imported on line 3 via wildcard).

3. **`artifacts/developer/src/game/world/faction.rs`** (after line 92):
   - Add `pub fn setup_cults_game_start(selected: Res<SelectedFaction>)` — just log a message, no structure spawning
   - Add `pub fn setup_colonists_game_start(selected: Res<SelectedFaction>)` — same
   - Follow the pattern of existing game start functions (take `Res<SelectedFaction>`, compute owner, log)

4. **`artifacts/developer/src/game/world/mod.rs`** (lines 68-73):
   - In the `OnEnter(AppState::InGame)` system set, add:
     - `faction::setup_cults_game_start.after(map::spawn_grid)`
     - `faction::setup_colonists_game_start.after(map::spawn_grid)`
   - These run alongside existing game start functions. Since they're stubs (no structure spawning), no additional ordering needed.

### Key patterns to follow:
- **Faction entity pattern** (faction.rs line 29-38): `commands.spawn((InvisibleEntity, FactionEnum::X, DisplayHud::new(FactionEnum::X)))`
- **Player entity pattern** (faction.rs line 41-59): `commands.spawn((InvisibleEntity, Player::new(name, faction, player_id), DisplayHudInfo::new(faction), FactionResources { ... }))`
- **Game start stub pattern**: Minimal function that takes `Res<SelectedFaction>`, computes owner id, logs faction start info
- **Registration pattern** (mod.rs line 68-72): add to the `OnEnter(AppState::InGame)` tuple with `.after(map::spawn_grid)`

### Existing game start functions both always run:
Note that `setup_gdo_game_start` and `setup_syndicate_game_start` currently BOTH run regardless of which faction is selected — they just assign different owner IDs. The new stub functions should follow this same pattern (always run, but determine owner based on SelectedFaction). The `setup_player_resources` function similarly must always spawn both (or all relevant) faction+player entities for the game to work.

### Bevy considerations:
- All game start functions run in `OnEnter(AppState::InGame)` schedule
- `setup_hud` (ui/mod.rs line 32) has `.after(setup_player_resources)` ordering — no change needed
- New systems must be `pub fn` to be accessible from mod.rs

## Dependencies

None — this is a standalone task. The resource types (`CultsPlayerResources`, `ColonistsPlayerResources`) and HUD queries already exist. The `FactionEnum::TheCults` and `FactionEnum::Colonists` variants already exist in `shared/types.rs`. The menu infrastructure (`ALL_FACTIONS`, `is_faction_available`, `faction_button_color`) already handles all 4 factions — only the `AVAILABLE_FACTIONS` filter needs updating.
