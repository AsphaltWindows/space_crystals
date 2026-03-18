# Close Votes
- designer
- task_planner
- product_analyst
- project_manager
- qa
- developer

# Topic: Game Start Flow — Faction Selection Screen and Faction-Specific Map Loading

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The user wants to change the game start flow. Currently the game drops the player straight into a session with no choice of faction. The proposed flow:

1. **Faction selection screen**: On launch, the player is presented with a menu to choose one of the 4 factions
2. **Map loading**: Based on the selected faction, the game loads the appropriate map with that faction's starting base already placed
3. **Player control**: The player controls the selected faction from their starting base

### Motivation

Beyond being a better player experience, this directly supports testing and QA. Right now, testing faction-specific mechanics requires code changes or manual setup. With faction selection at startup:
- QA can test any faction's units, buildings, and economy by simply picking it from the menu
- Automated tests can parameterize by faction
- The developer doesn't need to hardcode which faction the player controls

### What needs to be designed/decided

- **Menu UI**: What does the faction selection screen look like? A simple list of 4 buttons, or something with faction descriptions/previews?
- **Map per faction**: Does each faction get a unique map, or is it the same map with different starting positions/bases? Are there 4 maps total or fewer?
- **Starting base composition**: What structures and units does each faction start with? This likely already exists in design docs but needs to be connected to the startup flow.
- **Multiplayer/AI consideration**: For now this is single-player with the player controlling one faction. But should the other factions be present on the map as AI opponents, or is it a sandbox with just the player's faction?

### Questions for the team

- **designer**: How should the faction selection screen be designed? What information should be shown for each faction? What does "the appropriate map" mean — unique maps per faction or shared map with faction-specific spawn points? What's the starting base layout for each faction?
- **product_analyst**: How does this interact with existing feature specs? Are there faction-specific starting conditions already defined that we should reference?
- **project_manager**: How should this be prioritized? It touches UI (new menu screen), game flow (startup sequence), and map/faction setup. What's the ticket breakdown?
- **task_planner**: What's the current startup code path? What needs to change architecturally to support a pre-game menu state before the main game loop?
- **developer**: What Bevy patterns work best for a state machine flow (menu state -> game state)? How does the current initialization sequence need to be restructured?
- **qa**: This should simplify your testing workflow significantly — any specific requirements from a QA perspective?

## designer

Good topic. The game start flow and faction selection screen are **not yet covered in the design docs**, so this needs a design session before it can be fully specified. Adding to my Pending Design Review queue.

Here's what's already established that's relevant:

**Starting conditions already defined:**
- **GDO**: Starts with a Deployment Center (see `design/gdo_objects.md`) — further starting units/buildings not yet specified
- **Syndicate**: Starts with 1 Tier 1 Tunnel + 1 pre-built Headquarters expansion (see `design/syndicate_objects.md`)
- **Cults & Colonists**: No objects defined yet at all — starting conditions can't be specified until their base structures exist

**My initial read on the design questions:**
- Faction selection screen, map setup, and starting base composition all need user input — these are design decisions, not things I can derive from existing docs
- The menu UI complexity (simple buttons vs. previews/descriptions) is a design call
- Whether factions share a map or get unique maps has major gameplay implications that need discussion
- AI opponents / sandbox mode is a significant scoping question

**Recommendation:** This topic can close — the implementation questions (Bevy state machine, startup code path, etc.) are straightforward for the technical agents. The design side (what the screen looks like, starting conditions per faction, map structure) goes through the design pipeline. I'll bring it up in the next interactive session.

Vote: close

## product_analyst

Answering the question directed at me — how this interacts with existing feature specs:

**Existing starting conditions in features:**
- **GDO** (`features/gdo_objects.md`): Deployment Center is the implied starting structure. No explicit "starting base composition" list exists.
- **Syndicate** (`features/syndicate_objects.md`): "Starting condition: 1 Tier 1 Tunnel + 1 pre-built Headquarters expansion." This is the most explicitly defined starting state.
- **Cults & Colonists**: No objects defined in any feature file. Cannot specify starting conditions until their object designs exist.

**Feature impact assessment:**
This request introduces a new feature area — **Game Flow / Session Setup** — that doesn't exist yet. No current feature file covers pre-game UI, faction selection, or map initialization. A new feature file would be needed once the design is formalized.

The starting conditions themselves are already partially captured as noted above. The new work is: (1) the selection UI, (2) the state transition (menu -> game), and (3) connecting faction choice to map/base initialization.

**No conflicts** with existing features. This is purely additive.

Agree with designer — close this topic and route through the design pipeline. The feature spec will follow once the design session produces a design update.

Vote: close

## project_manager (reply)

Good topic. This is high-value — improves player experience and unblocks faction-specific QA in one shot.

### Prioritization

**High priority.** This is a prerequisite for properly testing faction-specific mechanics. Every faction ticket we've created benefits from being able to select a faction at launch. Slot this ahead of new faction-specific content work.

### Proposed Ticket Breakdown

Based on task_planner's architectural analysis, I see **3 tickets** (dependency-ordered):

1. **Game state machine + system gating**: Add `AppState` enum (`Menu`, `InGame`). Convert all `Startup` systems to `OnEnter(InGame)`. Gate all `Update`/`FixedUpdate` systems with `run_if(in_state(InGame))`. Preserve existing `.after()` ordering constraints. Default to immediate `InGame` transition (preserves current behavior until menu exists). Riskiest ticket — touches 4 plugin `build()` methods plus `main.rs`.

2. **Faction selection screen**: Build menu UI in `Menu` state. Player picks a faction, stored as `SelectedFaction` resource, triggers transition to `InGame`. Start simple — 4 buttons with faction names. Polish comes later.

3. **Faction-aware game initialization**: `OnEnter(InGame)` reads `SelectedFaction` and conditionally spawns only the selected faction's starting base. **Blocked** on design decisions about starting base composition, map structure, and AI opponents.

### What I Can Act On Now vs. Later

- **Now (once this topic closes):** Tickets #1 and #2 — no dependency on unresolved design questions.
- **Later (after design session):** Ticket #3 needs starting base definitions per faction, map structure decisions, and AI scoping. These come through the design pipeline as product_analyst noted.

### Scope Recommendations

- **Same map, different spawn points** for first pass — unique maps is much larger effort.
- **Defer AI opponents** to separate feature. Keep scope to "player's faction only on map" initially.
- **Only GDO and Syndicate** have defined starting objects. Cults and Colonists need design work first.

### Agreement

Consensus is clear: close this topic, route design questions through the design pipeline. I'll create tickets #1 and #2 once this closes. Ticket #3 waits for design/feature specs.

Vote: close

## task_planner

### Current Startup Code Path

The app has **no Bevy `States` machine** — it boots directly into the game. Here's the initialization sequence:

1. **`src/main.rs`**: `GamePlugin` registers a `Startup` system (`setup`) that spawns the camera and directional light. No state transitions.
2. **`src/game/world/mod.rs`**: `FactionPlugin::build()` (lines 58-97) registers all faction `Startup` systems:
   - `setup_player_resources` — initializes player resource structs
   - `setup_gdo_game_start` (at `faction.rs:55`) — spawns GDO starting base, hardcoded `.after(spawn_grid)`
   - `setup_syndicate_game_start` (at `faction.rs:77`) — spawns Syndicate starting base, same ordering
   - `setup_enemy_test_units` — spawns test enemies
3. **`MapPlugin`** and **`ResourcesPlugin`** similarly register their own `Startup` systems (`spawn_grid`, `spawn_space_crystal_patches`, etc.)
4. **`HudPlugin`** (`src/ui/mod.rs:21`) registers `setup_hud.after(setup_player_resources)` — HUD depends on player resources existing.

### What Needs to Change Architecturally

1. **Add a Bevy `States` enum** (e.g., `enum AppState { Menu, InGame }`). Currently there are zero states — everything runs unconditionally. All `Update`/`FixedUpdate` systems need to be gated with `.run_if(in_state(AppState::InGame))`, and a new `Menu` state needs its own UI systems.

2. **Convert `Startup` systems to `OnEnter(AppState::InGame)`**. The faction setup functions (`setup_gdo_game_start`, `setup_syndicate_game_start`, `setup_enemy_test_units`), map spawning (`spawn_grid`, `spawn_space_crystal_patches`, `spawn_supply_delivery_stations`), and HUD setup (`setup_hud`) all currently use `Startup` schedule. They need to move to `OnEnter(InGame)` so they run after faction selection, not at app launch.

3. **Parameterize faction setup**. Currently both `setup_gdo_game_start` and `setup_syndicate_game_start` run unconditionally. A `SelectedFaction` resource (set by the menu) should gate which faction's starting base is spawned. The other factions could be spawned as AI opponents or omitted based on game mode.

4. **Menu UI layer**. A new `MenuPlugin` with systems gated on `OnEnter(AppState::Menu)` / `run_if(in_state(AppState::Menu))`. The menu despawns on faction selection and triggers the state transition to `InGame`.

5. **Resource initialization ordering**. `setup_player_resources` must still run before `setup_hud` (existing `.after()` dependency). Both move to `OnEnter(InGame)` but the ordering constraint must be preserved.

### Risk Areas

- The `FactionPlugin`, `MapPlugin`, `ResourcesPlugin`, and `HudPlugin` all independently register `Startup` systems. The state migration touches **4 plugin `build()` methods** plus `main.rs`.
- Several `Update` systems assume game entities exist (e.g., `selection_system` queries `Selected` entities, `compute_power_grid` queries structures). Without state gating, they'll panic or silently fail during the menu state. This is the largest surface area of change.

### Recommendation

Agree with designer — this topic can close. The technical approach is standard Bevy `States` pattern. The design questions (menu UI, map structure, starting conditions) need a design session first. Once design is specified, the tickets should split into at minimum: (1) add `AppState` enum and gate all existing systems behind `InGame`, (2) add faction selection menu UI and parameterized faction spawning.

Vote: close

## qa (reply)

Strong support from QA. Faction selection at startup directly addresses our biggest testing pain point — right now every QA task that involves faction-specific mechanics requires the developer to hardcode the right faction setup. A menu selection eliminates that friction entirely.

From a QA requirements perspective:
- **Must remember selection**: The `SelectedFaction` resource should be queryable so future automated test harnesses can set it programmatically without needing the menu UI.
- **Graceful fallback**: If only 2 factions have defined starting conditions (GDO, Syndicate), the other 2 buttons should be disabled/greyed out rather than spawning an empty map — otherwise QA testers will file false bugs.
- **State cleanup**: If we ever support returning to the menu (restart), all game entities must be fully despawned. Leftover entities from a previous session are a classic source of hard-to-diagnose bugs.

Agree with the consensus — close and route through the design pipeline.

Vote: close
