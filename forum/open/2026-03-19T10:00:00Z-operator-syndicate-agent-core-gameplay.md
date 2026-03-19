# Designer Review: Syndicate Agent Core Gameplay Loop

## Metadata
- **Created by**: operator
- **Created**: 2026-03-19T10:00:00Z
- **Status**: open

## Close Votes

## Discussion

### [operator] 2026-03-19T10:00:00Z

The following items define the Syndicate Agent unit's core gameplay loop -- its command panel, resource gathering, tunnel building, groupability, and spawn behavior. None of these have been implemented yet. **Designer**: please review for correctness and completeness, then produce appropriate `feature_request` messages to get them into the pipeline.

---

### 1. Agent ObjectInterfaceState

The Agent unit needs a unique ObjectInterfaceState (not BasicCombatUnitInterfaceState) because its right-click resolution is context-sensitive with resource and tunnel interactions.

**DefaultState Commands:**
- **A: Build Tunnel** -- enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent to that location (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **B: Drop Off Resources** -- targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side C for supplies). Always visible, **greyed out when Agent is not carrying resources**.

**Unit Commands:** Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel

**Right-Click Resolution:**

| Right-Click Target | Command Issued |
|---|---|
| Crystal field | Gather crystals |
| Supply source | Gather supplies |
| Own Tunnel (carrying resources) | Drop off resources (auto-routes to correct side) |
| Own Tunnel (not carrying resources) | Enter |
| Enemy unit/building | Attack (melee) |
| Ground | Move |

**Multi-Select Note:** Despite being Ungroupable, right-click commands are issued to **all selected Agents** simultaneously (not just the ActiveGroup Agent).

**Requires:** New AgentCarryState component, AgentMenuState enum, new UnitCommand variants (Gather, DropOffResources, BuildTunnel), and right-click branching in the command handler.

**QA Steps:**
1. [human] Select an Agent. Verify the command panel shows two DefaultState commands: A (Build Tunnel) and B (Drop Off Resources).
2. [human] Verify button B (Drop Off Resources) is greyed out when the Agent is not carrying resources.
3. [human] Have the Agent pick up crystals from a Space Crystal Patch. Verify button B is now active (not greyed out).
4. [human] Press A (or hotkey). Verify the interface enters AwaitingPlacement: a ghost Tunnel preview follows the cursor, snapped to grid, tinted green on valid placement and red on invalid.
5. [human] While in AwaitingPlacement, press R. Verify the ghost rotates 90 degrees clockwise. Press Shift+R. Verify counter-clockwise rotation. Press F. Verify horizontal flip. Press Shift+F. Verify vertical flip.
6. [auto] Left-click a valid location. Verify the Agent is dispatched to build at that location (CommandIssuingTransition) and the interface returns to DefaultState.
7. [auto] Enter AwaitingPlacement again, then press Escape. Verify the interface returns to DefaultState without issuing a command.
8. [auto] Right-click a Space Crystal Patch. Verify the Gather crystals command is issued.
9. [auto] Right-click a Supply Delivery Station. Verify the Gather supplies command is issued.
10. [auto] Have the Agent carry crystals. Right-click an own Tunnel. Verify the Drop Off Resources command is issued (Agent walks to Tunnel Side B).
11. [auto] Have the Agent carry supplies. Right-click an own Tunnel. Verify the Drop Off Resources command is issued (Agent walks to Tunnel Side C).
12. [auto] Have the Agent carry nothing. Right-click an own Tunnel. Verify the Enter command is issued.
13. [auto] Right-click an enemy unit. Verify the Attack command is issued.
14. [auto] Right-click empty ground. Verify the Move command is issued.
15. [auto] Select multiple Agents. Right-click on ground. Verify all selected Agents receive the Move command.
16. [auto] Select multiple Agents. Right-click a Crystal field. Verify all selected Agents receive the Gather command.

---

### 2. Agent Spawn & Missing Commands (Critical Blocker)

Two critical bugs prevent Syndicate Agents from functioning:

**Bug 1 (PRIMARY): Missing Unit Commands in AgentMenuState.** The AgentDefault grid only defines two slots (Build Tunnel and Drop Off). All standard unit commands (Move, Stop, Attack, Enter, Gather) are missing. When an Agent is selected, the command panel routes to AgentMenu(AgentDefault), completely bypassing standard unit commands. The Agent needs all 7 commands: Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel.

**Bug 2: Spawn Position.** The HQ production system may place Agents incorrectly. The eject flow uses tunnel_side_world_position to compute Side A position, with a fallback to map center (32,32) if the parent tunnel query fails. Needs verification.

**QA Steps:**
1. [human] Start a game as Syndicate. Select the starting Tunnel, open Eject menu (C), and eject any pre-placed Agent. Verify the Agent appears at Side A with full commands.
2. [human] Select the Headquarters (underground). Set a rally point on the surface by right-clicking open ground. Produce an Agent (Q). Verify the Agent ejects from the parent Tunnel's Side A and moves toward the rally point.
3. [human] Verify the ejected Agent's command panel shows: Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel.
4. [human] Clear the rally point (or set it on the parent Tunnel). Produce another Agent. Verify the Agent does NOT appear on the surface -- it should enter the Tunnel Network instead.
5. [human] Open the Tunnel's Eject menu (C) and verify the newly produced Agent appears in the network unit list. Eject it and confirm it appears at Side A with full commands.
6. [human] Produce a Guard (W) with a surface rally point. Verify the Guard ejects from Side A and moves to the rally. Verify the Guard's command panel shows Move, Stop, Attack, Enter.

---

### 3. Agent Groupable & Construction Fix

**Part 1: Agent Groupable: false.** Each Agent must always be its own SelectionGroup, identical to how Tunnels work. Despite being ungroupable, right-click commands are still issued to **all selected Agents** simultaneously (special case).

**Part 2: Single-Agent Construction Enforcement.** When an Agent is dispatched to build a Tunnel at a location where another Agent is already constructing, the command should be rejected. Only one Agent may construct a given Tunnel at a time. This check should occur at behavior dispatch time -- when BuildTunnel is converted to the building behavior, query all existing building behaviors to check if any other entity targets the same location.

**QA Steps:**
1. Select a single Agent. Verify it forms its own SelectionGroup of size 1.
2. Box-select multiple Agents. Verify each Agent is in its own separate SelectionGroup (Tab/Shift-Tab cycles through individual Agents, not a merged group).
3. With multiple Agents selected, right-click on ground. Verify all selected Agents receive the Move command.
4. With multiple Agents selected, right-click on an enemy. Verify all selected Agents receive the Attack command.
5. Order Agent A to build a Tunnel at a location. While Agent A is constructing, order Agent B to build at the same location.
6. Verify Agent B's build command is rejected -- Agent B should not begin constructing the same Tunnel.
7. Verify only one Agent is embedded in the partially-built Tunnel at any time.
8. Destroy the partially-built Tunnel while Agent A is constructing. Verify Agent A emerges. Now order Agent B to build a new Tunnel at that location. Verify Agent B can construct successfully (the restriction is per-Tunnel, not per-location after destruction).

---

### 4. Agent Resource Gathering Commands & Behaviors

Two new commands and two new behaviors for the Agent resource gathering cycle.

**GatheringResource Behavior (full cycle):**
- Agent moves to the target resource source (MovingToObject sub-behavior)
- **Space Crystals**: MiningDuration of 48 frames at Space Crystal Patch, picks up 50 SC
- **Supplies**: PickUpDuration of 48 frames at Supply Delivery Station, picks up 1 Supply
- After extraction, the Agent **automatically** moves to the nearest own Tunnel's appropriate side (Side B for crystals, Side C for supplies)
- Drop-off: DropOffDuration of 48 frames for both types. One Agent at a time per drop-off side.
- The behavior encompasses the full gather-deliver cycle without further player input.

**DroppingOffResources Behavior (explicit delivery):**
- Agent moves to the target Tunnel's appropriate side based on carried resource type (Side B for crystals, Side C for supplies)
- Drop-off: 48 frames. Auto-routes to correct side -- player only needs to target the Tunnel.

**Interaction:** Gather command triggers GatheringResource (full cycle including auto-delivery). DropOffResources command triggers DroppingOffResources (explicit delivery without gathering first).

**QA Steps:**
1. [semi] Select an Agent and right-click a Space Crystal Patch. Verify the Gather command is issued and the Agent walks toward the patch.
2. [auto] Verify the Agent performs mining for exactly 48 frames upon reaching the Space Crystal Patch.
3. [auto] Verify the Agent picks up 50 Space Crystals after mining completes.
4. [semi] After mining, verify the Agent automatically moves to the nearest own Tunnel's Side B without player input.
5. [auto] Verify the drop-off takes exactly 48 frames at Side B.
6. [auto] Verify 50 Space Crystals are added to the player's crystal count after drop-off completes.
7. [semi] Select an Agent and right-click a Supply Delivery Station. Verify the Agent walks to the station and performs pickup for 48 frames.
8. [auto] Verify the Agent picks up 1 Supply after pickup completes.
9. [semi] After pickup, verify the Agent automatically moves to the nearest own Tunnel's Side C.
10. [auto] Verify the drop-off takes exactly 48 frames at Side C and 1 Supply is added to the player's supply count.
11. [auto] Send two Agents to drop off crystals at the same Tunnel Side B simultaneously. Verify only one Agent drops off at a time.
12. [auto] Send one Agent to drop off crystals (Side B) and another to drop off supplies (Side C) at the same Tunnel. Verify both can drop off simultaneously (separate sides).
13. [semi] Select an Agent carrying crystals. Issue an explicit DropOffResources command targeting an own Tunnel. Verify the Agent walks to Side B and drops off.
14. [semi] Select an Agent carrying supplies. Issue an explicit DropOffResources command targeting an own Tunnel. Verify the Agent walks to Side C and drops off.
15. [auto] Verify the DropOffResources command is unavailable (greyed out) when the Agent is not carrying resources.
16. [auto] Verify that non-Agent units cannot receive the Gather or DropOffResources commands.

---

### 5. Agent Tunnel Building Command & Behavior

**BuildTunnel Behavior execution sequence:**
1. Agent moves to the target build location (MovingToLocation sub-behavior)
2. Construction begins -- a partially-built Tunnel appears at the location, starting at **10% HP** (ConstructionHP Rule: HP = MaxHP x 10% = 60 HP for T1 Tunnel with 600 MaxHP)
3. The Agent **embeds inside** the partially-built Tunnel and becomes **untargetable** for the duration of construction
4. HP increases linearly over 480 frames (30 seconds): HP = MaxHP x (10% + 90% x construction_progress)
5. **If construction completes**: The Tunnel becomes fully operational. The Agent is placed inside the Tunnel Network (not on the surface), available for redeployment from any Tunnel.
6. **If the partially-built Tunnel is destroyed during construction**: The Agent survives and emerges at the Tunnel's location. The Tunnel is lost and any Supplies spent are not refunded.

**Constraints:**
- Only one Agent may construct a given Tunnel (multiple Agents cannot speed up construction)
- The Agent must remain present for the full 480-frame build duration
- Construction cost follows tunnel cost scaling formula: cost = current number of Tunnels owned, in Supplies

**QA Steps:**
1. [semi] Select an Agent, press A (Build Tunnel), place a ghost on a valid location, and left-click. Verify the BuildTunnel command is issued and the Agent begins walking to the target location.
2. [semi] Verify a partially-built Tunnel appears at the build location when the Agent arrives, starting at 10% of MaxHP (60 HP for a T1 Tunnel with 600 MaxHP).
3. [auto] Verify the Agent becomes untargetable once embedded in the partially-built Tunnel.
4. [auto] Verify HP increases linearly during construction. At 50% construction progress (240 frames), HP should be 600 x (0.10 + 0.90 x 0.50) = 330 HP.
5. [auto] Verify construction completes after exactly 480 frames and the Tunnel becomes fully operational.
6. [auto] After construction completes, verify the Agent is inside the Tunnel Network and can be ejected from any Tunnel.
7. [auto] During construction, destroy the partially-built Tunnel. Verify the Agent survives and appears at the Tunnel's former location.
8. [auto] Verify the surviving Agent is targetable again after emerging from a destroyed construction site.
9. [auto] Verify the Tunnel construction cost follows the scaling formula.
10. [auto] Attempt to assign a second Agent to construct the same partially-built Tunnel. Verify this is rejected.
11. [auto] Verify that non-Agent units cannot receive the BuildTunnel command.

---

### 6. Worker-Built Structure Arrival Validation

Worker-built structure placement (e.g., Agent building a Tunnel) follows a two-phase validation model:

**At command time:**
- The build command is accepted regardless of current fog of war visibility state. No visibility check is performed.
- The command is queued and the worker begins pathfinding to the target location.

**On worker arrival:**
- All footprint tiles must be validated: tiles must be **Buildable**, **unoccupied** (no existing structure overlap), and meet any faction-specific constraints.
- **No visibility requirement** on arrival -- the worker is physically present.
- If validation **passes**: construction begins normally (Tunnel appears at 10% HP per ConstructionHP Rule).
- If validation **fails**: the build command is **cancelled**, the worker **stops and idles** at its current position. No error feedback beyond the unit idling.

This validation model is distinct from Direct Placement (which requires Visible tiles and validates immediately). It allows players to speculatively send workers to locations they haven't scouted yet.

**QA Steps:**
1. [auto] Start a game as Syndicate. Produce an Agent from the Headquarters.
2. [human] Order the Agent to build a Tunnel on a valid, visible location with Buildable tiles and no existing structures.
3. [auto] Verify the Agent pathfinds to the location and begins construction successfully.
4. [human] Produce a second Agent. Order it to build a Tunnel on the same tile as the first Tunnel (now occupied).
5. [auto] Wait for the second Agent to arrive. Verify the build command is cancelled -- the Agent stops and idles.
6. [human] Order an Agent to build a Tunnel on a tile covered by fog of war.
7. [auto] Verify the build command is accepted immediately (no rejection at command time).
8. [auto] Wait for the Agent to pathfind to the location. If tiles are Buildable and unoccupied on arrival, verify construction begins. If tiles are invalid, verify the Agent stops and idles.
9. [auto] Order an Agent to build a Tunnel on a non-Buildable tile (e.g., Rock terrain). Verify the Agent walks there and then idles without building.

---

### Key questions for the designer:
- Are all 7 Agent commands (Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel) correct and complete?
- Is the right-click resolution table correct for all target types?
- Should the Agent's command panel layout match any specific grid slot arrangement?
- Is the 480-frame (30 second) tunnel build duration correct?
- Is the resource gathering cycle (48 frames each for mining, pickup, and drop-off) balanced correctly?
- Should failed arrival validation provide any player feedback beyond the Agent idling?
