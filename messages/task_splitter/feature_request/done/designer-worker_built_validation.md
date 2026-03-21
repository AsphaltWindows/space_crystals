# worker-built-validation

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement two-phase placement validation for worker-built structures as defined in `artifacts/designer/design/entities.md` under 'Placement Validation > Worker-Built Structures'.

Worker-built structures (e.g., Agent building a Tunnel) use a different validation model than direct-placement structures:

**Phase 1 — Command Acceptance:**
- The player queues a build command targeting a map location
- The command is accepted regardless of current visibility — NO visibility check at command time
- The worker begins pathfinding to the location

**Phase 2 — Arrival Validation:**
- When the worker arrives at the location, validation occurs:
  - All tiles under the footprint must be Buildable
  - All tiles must be unoccupied (no existing structure overlap)
  - Faction-specific constraints must be met (e.g., Tunnel Area non-overlap)
- No visibility requirement on arrival (the worker is physically present)
- If validation fails on arrival, the command is cancelled and the worker stops and idles

This contrasts with Direct Placement (GDO buildings, Tunnel expansions) which requires all footprint tiles to be Visible at the moment of placement confirmation.

## QA Instructions

1. Order an Agent to build a Tunnel on a location that is currently in the Explored (fog) state.
2. Verify the command is accepted — the Agent begins walking toward the location.
3. If the location is still valid on arrival: verify construction begins normally.
4. Order an Agent to build a Tunnel on a location. Before the Agent arrives, place another structure on those tiles (e.g., from another player or via a second Tunnel).
5. Verify the Agent arrives, fails validation, and stops/idles without building.
6. For comparison: attempt to place a GDO building (Direct Placement) on a non-Visible tile.
7. Verify the ghost shows red and placement cannot be confirmed — validating the Direct Placement visibility check still works.
