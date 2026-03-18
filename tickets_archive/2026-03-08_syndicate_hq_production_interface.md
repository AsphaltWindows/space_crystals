# Ticket: Syndicate Headquarters Must Show Agent Production Commands

## Current State
When selecting the Syndicate Headquarters (underground expansion), the command panel shows unit commands (Move, Attack, etc.) instead of Agent production commands. This prevents the player from producing Agents, breaking the core Syndicate production loop.

## Desired State
The Headquarters expansion must have an ObjectInterfaceState that shows Agent production in its command panel. When selected:

### DefaultState commands:
- **A: Produce Agent** — CommandIssuingTransition. Costs 100 SC, takes 160 frames (10 seconds). Greyed out if insufficient Supply Credits or if the production queue is full. Queues an Agent for production; produced Agent emerges from the parent Tunnel or enters the Tunnel Network depending on rally point.

The Headquarters should NOT show Move, Attack, or other unit commands — it is a structure, not a unit.

## Justification
Discovered during QA session 2026-03-08 (forum topic `qa_session_2026_03_08_issues.md`, issue #3). `features/syndicate_objects.md` specifies "Headquarters produces Agent (100 SC, 160 frames / 10 seconds)" but no explicit ObjectInterfaceState was defined for the Headquarters expansion. The intent is clear: the HQ is the Syndicate's primary unit-producing structure, analogous to GDO Barracks. Without this interface, Syndicate faction gameplay is non-functional — the player cannot produce any units.

Note: The feature spec may need a formal Headquarters ObjectInterfaceState section added by the product analyst. This ticket covers the implementation of the production interface based on existing spec intent.

## QA Steps
1. [human] Start a game as Syndicate — select the Headquarters expansion inside the starting Tunnel
2. [human] Verify the command panel shows "Produce Agent" (A) — NOT Move/Attack/unit commands
3. [human] Click Produce Agent with sufficient Supply Credits — verify an Agent is queued for production
4. [human] Wait 160 frames (10 seconds) — verify the Agent is produced and emerges from the parent Tunnel
5. [human] Verify the cost (100 SC) is deducted when production begins
6. [human] Reduce Supply Credits below 100 — verify Produce Agent is greyed out / unavailable
7. [human] Select the Headquarters again while an Agent is in production — verify production progress is visible

## Expected Experience
Selecting the Headquarters shows a clean production interface with a single "Produce Agent" button. Clicking it begins visible production. After 10 seconds, an Agent unit appears from the parent Tunnel. The interface feels like a standard RTS production building — queue, wait, unit emerges. No unit movement or attack commands are shown.
