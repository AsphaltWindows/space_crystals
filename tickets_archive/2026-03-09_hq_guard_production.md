# Ticket: Headquarters Guard Production

## Current State
The Headquarters production interface (being implemented per existing ticket `syndicate_hq_production_interface`) only supports producing Agents. The production cost function in `HeadquartersState` only has a cost entry for `ObjectEnum::SyndicateAgent`.

## Desired State
Add Guard to the Headquarters production list:

- **Guard production cost**: 125 Space Crystals
- **Guard build time**: 120 frames (7.5 seconds)

### UI Changes
- The Headquarters command panel should show two production buttons:
  - **A (0,0): Produce Agent** — 100 SC, 160 frames (existing)
  - **B (0,1): Produce Guard** — 125 SC, 120 frames (new)
  - **Cancel** button shifts to accommodate both production options
- Both unit types share the same production queue (HeadquartersState.build_queue)

### Production System Changes
- `HeadquartersState::production_cost()` must return cost for Guard variant
- `headquarters_production_tick_system()` must call `spawn_syndicate_guard()` when a Guard completes production
- Produced Guards emerge from the parent Tunnel (same as Agent production)

## Justification
`features/syndicate_objects.md` — Headquarters Produces section lists both Agent and Guard. The original ticket only covered Agent production because the Guard spec was an open question at that time. The 2026-03-09 feature update resolved this: HQ produces both Agent and Guard.

## QA Steps
1. [human] Select a Headquarters — verify both "Produce Agent" and "Produce Guard" buttons are visible
2. [human] Click Produce Guard with sufficient Space Crystals (≥125) — verify a Guard is queued
3. [human] Wait 120 frames (7.5 seconds) — verify a Guard unit emerges from the parent Tunnel
4. [human] Verify 125 SC is deducted when Guard production begins
5. [human] Queue both an Agent and a Guard — verify they produce sequentially from the same queue
6. [human] Verify the Cancel button cancels the last queued item regardless of type

## Expected Experience
The Headquarters now functions as a dual-unit production building. Both Agent and Guard buttons are clearly visible. Producing a Guard is faster (7.5s vs 10s) but costs more (125 vs 100 SC). The queue handles mixed unit types seamlessly — the player can interleave Agent and Guard production orders.
