# 0071 — No smart-casting; Shift while placing

- **Date**: 2026-11-24T00:20:00-05:00
- **Answers**: — (follow-up to 0070 / Q0016 point 4)
- **Docs**:
  - `systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting` (new)
  - `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType] > Shift while placing` (new; resolves the placement TBD from 0070)
  - `core/entities.md > Object Type > Groupable` (pointer)

## What changed and why
1. **No smart-casting, anywhere.** A command from the command panel goes to every instance in the ActiveGroup. The game never hands a group's command to just one member. The user's reasoning is that smart-casting makes mass ability use effortless, which limits how strong abilities can be. Workers follow the same rule.
2. **Groupable is how individual use works, and it is fixed per type.**
   - **Ungroupable:** workers and caster-type units. Each instance is its own Tab stop: cast, Tab, cast. This is the intended middle ground between selecting units one by one with the mouse and smart-casting.
   - **Groupable:** units whose abilities are used all together, e.g. the Riot Shield.
   - A player-controlled grouping toggle was considered and **parked**.
3. **Shift while placing.** Same rule as Shift while targeting (0070): placement stays active while Shift is held, and each Shift-click queues one build order. This only applies when the builder can carry out more queued buildings, e.g. a Cults Recruit, whose cost is paid per order as given. Otherwise a Shift-click works like a plain click and ends placement:
   - Agents go inside the structure they build.
   - A Drone becomes its Beacon.
   - GDO builders have only one ready building at a time.
   - Placement never moves on to another selected worker automatically. The player presses Tab.

## Implementation impact
- Panel commands must go to all instances of the ActiveGroup. Check that no command picks a single unit out of a group.
- AwaitingPlacement and Shift:
  - **Recruit:** stay in placement while Shift is held, and queue build orders on the active Recruit.
  - **Agent, Drone, GDO:** exit placement after the click.
- Nothing changes for existing Groupable values. New caster units will be marked Ungroupable when they are designed.
- **Known doc conflict, still open (being raised with the user):**
  - Recruits are Ungroupable, so a Build order from the panel goes only to the active Recruit.
  - `factions/cults/economy.md > Build Command` says "All selected Recruits walk to the location".
  - Until it is resolved, follow the Ungroupable rule (active Recruit only), since it matches this change.

## Verification
1. Select 3 Drones and press Deploy. Shift-click a valid spot. Only the Drone shown in the panel goes there, and placement ends. Press Tab, then Deploy again for the next one.
2. Select a Recruit, choose a building, and Shift-click 3 spots while holding Shift. Each click deducts the cost. The Recruit builds them in order. Releasing Shift ends placement.
3. Select 4 Peacekeepers and press the Riot Shield key. All 4 raise their shields.
