# Ticket: Faction Selection Screen

## Current State
After the game app state machine ticket is implemented, the game will have `AppState::Menu` and `AppState::InGame` states, with a temporary auto-transition from `Menu` to `InGame`. There is no menu UI — the player has no way to choose which faction they control. Both GDO and Syndicate starting bases are spawned unconditionally.

## Desired State
A faction selection screen is displayed when the game starts in `AppState::Menu`:

1. **Menu UI**: A centered panel with one button per available faction. For now, only **GDO** and **Syndicate** buttons are enabled (these have defined starting objects). **Cults** and **Colonists** buttons are present but visually disabled/greyed out (their objects are not yet designed).
2. **`SelectedFaction` resource**: When the player clicks an enabled faction button, a `SelectedFaction` resource is inserted (or updated) containing the chosen faction identifier. This resource must be public and queryable so automated test harnesses can set it programmatically without the menu UI.
3. **State transition**: After faction selection, the app transitions from `AppState::Menu` to `AppState::InGame`. The menu UI is fully despawned on state exit.
4. **Faction-conditional spawning**: The `OnEnter(AppState::InGame)` faction setup systems read the `SelectedFaction` resource and spawn only the selected faction's starting base as the player-controlled faction. The non-selected factions with defined starting objects still spawn as opponent bases (existing behavior), but the player's camera and control are associated with the selected faction.
5. **Remove auto-transition**: The temporary auto-transition from the state machine ticket is removed, replaced by the menu-driven transition.

## Justification
Forum topic `forum/faction_selection_game_start.md` established consensus that faction selection at startup is high priority. It improves player experience and unblocks faction-specific QA testing (QA can test any faction by selecting it from the menu rather than requiring code changes). QA specifically requested: disabled buttons for undefined factions (prevent false bug reports), queryable `SelectedFaction` resource (automated testing support), and full entity cleanup on state transitions (future restart support). This is ticket #2 of 2, dependent on the game app state machine ticket.

## QA Steps
1. Run `cargo build` — project compiles without errors.
2. Launch the game — a faction selection screen appears instead of immediately entering gameplay.
3. Verify the screen shows buttons for GDO, Syndicate, Cults, and Colonists.
4. Verify the Cults and Colonists buttons are visually disabled and cannot be clicked.
5. Click the **GDO** button — the menu disappears and the game loads with the player controlling the GDO faction. The GDO starting base is present and player-controlled. Verify units can be selected and commanded.
6. Close and relaunch the game. Click the **Syndicate** button — the game loads with the player controlling the Syndicate faction. The Syndicate starting base (Tunnel + Headquarters) is present and player-controlled.
7. Verify no menu UI elements remain visible during gameplay.
8. Verify the HUD displays correctly for the selected faction (resource panel shows the selected faction's resources).

## Expected Experience
On launch, the player sees a clean faction selection screen with faction names as buttons. GDO and Syndicate are clickable; Cults and Colonists are greyed out. Clicking an enabled faction smoothly transitions to the game with that faction's starting base under the player's control. The experience feels like a natural game startup flow rather than being dropped into an arbitrary state.
