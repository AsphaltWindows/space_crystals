# Q0067 — Supplies gathering

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slice M4-I builds.
- **Docs**: `factions/syndicate/units.md > Agent > Gathering > Supplies`; `core/map.md > SupplyDeliveryStation`; `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD` (the Supply hatch, 0224); `systems/controls/behaviors.md > Core principle: predictable, not optimal`.

## Question
1. **Several Agents at one station.** **Proposal:** any number pick up at once, as at patches.
2. **When a Supply leaves the station.** **Proposal:** when the pick-up ends. A pick-up starts only if a Supply remains that no running pick-up has claimed.
3. **An empty station.** **Proposal:** the Agent waits there for the next delivery, keeping its job.
4. **An Agent carrying one resource ordered to gather the other.** **Proposal:** it first delivers its load at the right hatch of the Tunnel it delivers to (Q0066 point 2), then goes. The Drone's rule (0128: go to the patch and "skip mining") doesn't fit two resources.

## Context
The station's countdown "begins only once the station's current supply content reaches 0", and each faction collects "using faction-specific collection mechanics". The Agent's figures are given (carry 1, 48 frames each way), but sharing a station and an empty one aren't. Side C is the Supply hatch (0224); Q0066 point 6 asks whether a hatch is the whole side. Following 0223, no proposal sends an Agent to another station or resource by itself.

## Options
- **Point 1:** A, any number, as proposed. B, one Agent at a time.
- **Point 3:** A, wait, as proposed. B, idle.
- **Point 4:** A, deliver first, as proposed. B, a rule you give.
