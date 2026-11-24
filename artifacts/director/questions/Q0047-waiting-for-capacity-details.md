# Q0047: Waiting for capacity: several waiting buildings, no power, zero-cost units

- **Date**: 2026-11-24T13:54:02-05:00
- **Blocking**: no. M3-P is built with the current behaviour listed under each point.
- **Docs**: `factions/gdo/overview.md > Unit Control` (Reserved when production starts); `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts`; `systems/controls/selection_and_hud.md > Group production with limited resources`; changes 0143 and 0083.

## Question
1. **Several buildings wait, and room appears for only one unit.** Which starts?
   - **Current behaviour:** the building with the lowest id. That's effectively the oldest building, and the player can't see or control it.
   - 0083's "least remaining work pays first" was read as a queue-time rule for resources only.
2. **GDO with power at 0.** A Barracks whose front unit passes the capacity check makes no progress while there's no power.
   - **Current behaviour:** the unit doesn't start and doesn't take the room until progress can begin, so another building can take it.
3. **Over the cap, a unit that costs 0 capacity.** "Free capacity ≥ its cost" is false when used > available, even for a cost of 0.
   - **Current behaviour:** it waits like any other.
   - No such unit exists yet. This decides whether a free unit, a future summon for example, can be built while over the cap.

## Context
All three came up while building 0143 (M3-P).

## Options
- **Point 1:**
  - A, as now: lowest id.
  - B, the building that has waited longest.
  - C, 0083's rule applied here: least remaining work first, then the selection order of a group.
- **Point 2:**
  - A, as now: no start without power.
  - B, the unit starts and takes the room even at power 0, then progresses when power returns.
- **Point 3:**
  - A, as now: it waits.
  - B, a cost of 0 always passes.
