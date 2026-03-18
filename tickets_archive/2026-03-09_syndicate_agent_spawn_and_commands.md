# Ticket: Syndicate Agent spawns at Tunnel center with no commands (critical blocker)

## Current State
When the Headquarters expansion produces an Agent, the unit spawns at the parent Tunnel's grid center position (underneath the Tunnel mesh). The spawned Agent has no Move command or any other commands available in the command panel, making it completely non-functional. This blocks ~8 Syndicate QA tasks.

## Desired State
1. **Spawn position**: Agents (and Guards) produced by the Headquarters should follow the Tunnel expansion production flow:
   - If a rally point is set on the surface: the unit auto-ejects from the parent Tunnel's Side A and moves to the rally point.
   - If no rally point is set, or rally is set on the parent Tunnel: the unit enters the Tunnel Network and is available for ejection from any sufficiently-tiered Tunnel.
2. **Command initialization**: Spawned Agents must have their full ObjectInterfaceState initialized (Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel). Note: Agents do NOT use BasicCombatUnitInterfaceState — they have a custom ObjectInterfaceState with resource and building commands.
3. **Guard initialization**: Same spawn/eject flow applies. Guards use BasicCombatUnitInterfaceState (Move, Stop, Attack, Enter).

## Justification
- `features/syndicate_objects.md`: Tunnel expansion production flow specifies Side A ejection for surface rally points, Tunnel Network entry otherwise. Headquarters is an underground expansion inside a Tunnel.
- `features/syndicate_objects.md`: Agent ObjectInterfaceState defines full command set (Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel).
- Forum topic `syndicate_agent_spawn_blocker.md`: QA-reported critical blocker affecting ~8 downstream tasks across the entire Syndicate QA pipeline.

## QA Steps
1. [human] Start a game as Syndicate. Select the starting Tunnel, open Eject menu (C), and eject any pre-placed Agent. Verify the Agent appears at Side A with full commands.
2. [human] Select the Headquarters (underground). Set a rally point on the surface by right-clicking open ground. Produce an Agent (Q). Verify the Agent ejects from the parent Tunnel's Side A and moves toward the rally point.
3. [human] Verify the ejected Agent's command panel shows: Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel.
4. [human] Clear the rally point (or set it on the parent Tunnel). Produce another Agent. Verify the Agent does NOT appear on the surface — it should enter the Tunnel Network instead.
5. [human] Open the Tunnel's Eject menu (C) and verify the newly produced Agent appears in the network unit list. Eject it and confirm it appears at Side A with full commands.
6. [human] Produce a Guard (W) with a surface rally point. Verify the Guard ejects from Side A and moves to the rally. Verify the Guard's command panel shows Move, Stop, Attack, Enter.

## Expected Experience
- Step 1: An Agent unit appears at the Tunnel's Side A edge. Selecting it shows a command panel with Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel.
- Step 2: After production completes, an Agent walks out from Side A of the Tunnel and pathfinds toward the rally point on the ground.
- Step 3: The command panel displays all seven commands. Each is clickable/hotkey-accessible.
- Step 4: No unit appears on the map surface after production completes. The Tunnel Network unit count increases.
- Step 5: The Eject menu shows the Agent type with an incremented count. Clicking it causes an Agent to emerge from Side A with full commands.
- Step 6: A Guard emerges from Side A and moves to the rally point. Its command panel shows the four BasicCombatUnit commands.
