# Ticket: Game App State Machine

## Current State
The game has no Bevy `States` enum. All systems run unconditionally from app launch. Startup systems (faction setup, map spawning, HUD setup, resource initialization) all use the `Startup` schedule and execute immediately. All `Update` and `FixedUpdate` systems run every frame with no state gating. There is no concept of a pre-game menu — the player is dropped directly into gameplay.

## Desired State
An `AppState` enum with at least two variants (`Menu` and `InGame`) controls the game flow:

1. **`AppState` enum**: Defined as a Bevy `States` enum with `Menu` and `InGame` variants. Default/initial state is `Menu`.
2. **Startup system migration**: All current `Startup` systems that spawn game entities (faction bases, grid, space crystal patches, supply delivery stations, HUD, player resources, test enemies) are converted to `OnEnter(AppState::InGame)` systems. Existing `.after()` ordering constraints (e.g., `setup_hud.after(setup_player_resources)`, faction setup `.after(spawn_grid)`) must be preserved.
3. **Update/FixedUpdate gating**: All existing `Update` and `FixedUpdate` systems are gated with `.run_if(in_state(AppState::InGame))` so they do not run during the `Menu` state.
4. **Camera and light**: The camera and directional light setup (in `main.rs` `setup`) may remain as a true `Startup` system since these are needed in both states, OR can be moved to `OnEnter(InGame)` — developer's discretion based on whether the menu needs a camera.
5. **No menu UI yet**: This ticket does NOT add a menu screen. A temporary auto-transition from `Menu` to `InGame` on the first frame ensures existing behavior is preserved until the faction selection ticket is implemented.

Plugin `build()` methods that need modification:
- `FactionPlugin` (`src/game/world/mod.rs`)
- `MapPlugin`
- `ResourcesPlugin`
- `HudPlugin` (`src/ui/mod.rs`)
- `GamePlugin` / `main.rs`

## Justification
Forum topic `forum/faction_selection_game_start.md` established consensus that a game state machine is prerequisite for faction selection at startup. Task_planner confirmed zero states exist currently and identified all systems requiring migration. This is ticket #1 of 2 — the faction selection screen (ticket #2) depends on this state machine existing.

## QA Steps
1. Run `cargo build` — project compiles without errors.
2. Run the game — it should auto-transition from `Menu` to `InGame` and behave identically to the current experience (grid spawns, factions spawn, HUD appears, units are controllable).
3. Verify the GDO starting base spawns correctly with Deployment Center and starting units.
4. Verify the Syndicate starting base spawns correctly with Tunnel and Headquarters.
5. Select units and issue move/attack commands — confirm all Update/FixedUpdate gameplay systems function normally.
6. Verify the HUD displays correctly (resource panel, selection panel, command panel).
7. Open the Bevy inspector (if available) or check logs to confirm `AppState` resource exists and is set to `InGame`.

## Expected Experience
The player should notice zero difference from the current game behavior. The game launches and immediately enters gameplay with all factions, map, and HUD present as before. This ticket is purely infrastructural — it adds the state machine without changing visible behavior.
