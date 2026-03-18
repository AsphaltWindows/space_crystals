# Ticket: Worker-Built Structure Arrival Validation

## Current State
The Syndicate Agent's build command flow (queue command, pathfind to location, begin construction) does not have a formal validation step on arrival. The Agent ticket (`tickets/2026-03-06_syndicate_agent_unit.md`) describes the construction sequence but does not specify what happens if the target tiles are invalid when the worker arrives (e.g., another structure was placed there during transit, tiles became unbuildable). There is also no specification for whether visibility should be checked at command time or arrival time.

## Desired State
Worker-built structure placement (e.g., Agent building a Tunnel) follows a two-phase validation model:

**At command time:**
- The build command is accepted regardless of current fog of war visibility state. No visibility check is performed when the player issues the command.
- The command is queued and the worker begins pathfinding to the target location.

**On worker arrival:**
- All footprint tiles must be validated: tiles must be **Buildable**, **unoccupied** (no existing structure overlap), and meet any faction-specific constraints.
- **No visibility requirement** on arrival — the worker is physically present at the location.
- If validation **passes**: construction begins normally (Tunnel appears at 10% HP per ConstructionHP Rule, etc.).
- If validation **fails**: the build command is **cancelled**, the worker **stops and idles** at its current position. No error feedback beyond the unit idling — the command simply does not execute.

## Justification
Specified in `features/entity_system.md` (Placement Validation section, "Worker-Built Structures"). Design source: `design/entities.md` (Placement Validation section). This validation model is distinct from Direct Placement (which requires Visible tiles and validates immediately). The arrival validation prevents structures from being built on tiles that became invalid during the worker's transit time, and the no-visibility-at-command-time rule allows players to speculatively send workers to locations they haven't scouted yet.

## QA Steps
1. Start a game as Syndicate. Produce an Agent from the Headquarters.
2. Order the Agent to build a Tunnel on a valid, visible location with Buildable tiles and no existing structures.
3. Verify the Agent pathfinds to the location and begins construction successfully.
4. Produce a second Agent. Order it to build a Tunnel on the same tile as the first Tunnel (now occupied).
5. Wait for the second Agent to arrive. Verify the build command is cancelled — the Agent stops and idles at or near the target location. No Tunnel is placed.
6. Order an Agent to build a Tunnel on a tile covered by fog of war (Unexplored or Explored state).
7. Verify the build command is accepted immediately (no rejection at command time).
8. Wait for the Agent to pathfind to the location. If the tiles are Buildable and unoccupied on arrival, verify construction begins. If tiles are invalid (e.g., non-Buildable terrain), verify the Agent stops and idles.
9. Order an Agent to build a Tunnel on a non-Buildable tile (e.g., Rock terrain). Verify the Agent walks there and then idles without building.

## Expected Experience
- Issuing a build command to a fogged tile is accepted without any error — the Agent begins walking toward the target. This differs from GDO placement, where fogged tiles show a red ghost and block placement.
- If the Agent arrives and the tiles are valid, construction begins as normal (Tunnel appears, Agent embeds inside).
- If the Agent arrives and the tiles are invalid (occupied, non-Buildable), the Agent simply stops and stands idle. No error message or special feedback — the player observes that construction did not start and must reissue a command.
- This allows Syndicate players to speculatively queue Tunnel construction in unscouted areas, adding strategic depth at the cost of potentially wasting the Agent's travel time.
