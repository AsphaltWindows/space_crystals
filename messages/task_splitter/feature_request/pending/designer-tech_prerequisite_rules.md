# tech_prerequisite_rules

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Define shared, cross-faction rules for building TechPrerequisites, and the Syndicate-specific split between Tier Requirement (per-Tunnel) and TechPrerequisite (network-wide).

**artifacts/designer/design/entities.md** — new top-level section `## Tech Prerequisites` (after Placement Validation):
- A structure type may declare a TechPrerequisite: one or more structure types the player must own before construction can be started. Applies to all factions.
- **Counting rule**: a structure only counts once fully constructed AND present on the map. GDO: must have been placed (a completed building in a constructor's ReadyToPlace slot does NOT count). Syndicate: expansion construction finished. Colonists: Warp In completed (mid-transformation Beacon does not count). Cults: Recruits finished constructing it. Under-construction / awaiting-placement structures never count.
- **Check timing**: checked only at the moment construction is started (queued at constructor, Warp In begun, expansion construction begun). Options whose prerequisite is unmet are unavailable in the build menu.
- **Losing a prerequisite**: existing completed structures keep working; constructions already started (incl. GDO buildings in progress or in ReadyToPlace) continue and can be completed/placed; only NEW constructions requiring it are blocked until a qualifying prerequisite is owned again.

**artifacts/designer/design/syndicate_objects.md**:
- New `### Expansion Requirements` subsection under Tunnel Expansions. Tier Requirement is checked against the specific Tunnel doing the building (it can only build within its own Tunnel Area, and ITS tier must meet the requirement). TechPrerequisite is checked across the whole Tunnel Network — the required expansion may be in any owned Tunnel of any tier.
- Tunnel ExpandMenu text updated: an expansion is available only if its Tier Requirement <= this Tunnel's tier AND its TechPrerequisite is satisfied anywhere in the network.
- Hideout gains `TechPrerequisite - None`.
- Note: no Syndicate expansion currently has a non-None TechPrerequisite (Hideout is the only expansion). Implement the network-wide check generically so future expansions can declare one; it can't be exercised in-game yet.

**artifacts/designer/design/gdo_objects.md**: DeploymentCenter Constructs list gains a note that Requires clauses follow entities.md > Tech Prerequisites (placed-only counting, checked at construction start).

Important: the design doc says SupplyTower requires Radar Station, but per the earlier `gdo_rename_powerplant_to_generator` request the code still gates SupplyTower on Generator until Radar Station is fully stated. Keep that gating as-is; just apply the new counting/timing/loss rules to whatever prerequisites currently exist in code (Generator -> SupplyTower, and any others implemented).

## QA Instructions

Use a GDO player (SupplyTower is currently gated on Generator in code).

1. **ReadyToPlace doesn't count**: With no Generator on the map, queue a Generator at the Deployment Center and let it finish so it sits in ReadyToPlace. Open the BuildMenu — Supply Tower must still be unavailable.
2. **Placed counts**: Place the Generator. Open the BuildMenu — Supply Tower is now available.
3. **Loss doesn't cancel in-progress work**: Start constructing a Supply Tower, then destroy your only Generator (debug/kill). The Supply Tower construction continues, finishes, and can still be placed. Placed Supply Towers keep working normally (choppers, drop-offs).
4. **Loss blocks new starts**: With no Generator remaining, open the BuildMenu — Supply Tower is unavailable again. Place a new Generator — Supply Tower becomes available again.
5. **Syndicate**: As Syndicate, open a Tier 1 Tunnel's ExpandMenu — Hideout is available (TechPrerequisite None, Tier 1). Behaviour of Tier Requirement is unchanged (expansions above the Tunnel's own tier are not offered). If the implementation includes a test harness, verify via unit test that an expansion with a TechPrerequisite is offered by a Tunnel when the required expansion exists (fully built) in a DIFFERENT Tunnel, and not offered when the only copy is still under construction.
