# syndicate_tunnel_space_from_unused_area

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Reworked how the Syndicate's Tunnel Space (population cap resource) is calculated — replacing the flat per-Tunnel-tier value with a value derived from unused underground space. Changed `artifacts/designer/design/syndicate_objects.md` and `artifacts/designer/design/factions.md`.

## Old behavior
Each Tunnel provided a flat Tunnel Space value based on its tier: Tier 1 = 20, Tier 2 = 30, Tier 3 = 40. This was a table row on the Tunnel's Tiers table, totally independent of what was built inside the Tunnel's underground area.

## New behavior
Note: this change also coincides with a reduction in Tunnel Area sizes (separate decision, same session) — Tunnel Area is now 8x8/10x10/12x12 grid units by tier (previously 10x10/12x12/14x14). All numbers below reflect the new, smaller sizes.

Tunnel Space is now derived dynamically:
- **Tunnel-region**: a Tunnel's Tunnel Area (the underground build zone — 8x8/10x10/12x12 grid units by tier) MINUS the 4x4 grid units occupied by the Tunnel's own footprint. E.g. a Tier 1 Tunnel's tunnel-region = 64 - 16 = 48 grid units.
- **Unused tunnel-region**: whatever part of a Tunnel's tunnel-region does NOT currently have an expansion built on it.
- **Tunnel Space = (sum of unused tunnel-region grid units across ALL owned Tunnels) / 4**, still capped at a hard maximum of 200 (this cap is unchanged from before).

This means building an expansion (e.g. a 2x2 Hideout) permanently converts 4 grid units of unused tunnel-region into occupied space, directly reducing total Tunnel Space by 1. The only way to grow army capacity AND production capacity together is building more Tunnels or upgrading existing ones (bigger Tunnel Area = bigger tunnel-region). This is an intentional trade-off: production/tech buildings cost you population cap, not just resources.

**Overflow handling**: if a newly-placed expansion drops total Tunnel Space below the player's already-committed usage, existing units are NOT affected/removed. The HUD simply displays usage over capacity (e.g. `12/10`), and no new units can be produced until usage drops back within capacity — this mirrors how GDO's Power system allows going negative rather than forcibly destroying/removing things.

## Files changed
- `syndicate_objects.md`: Removed the "Tunnel Space" row from the Tunnel's Tiers table (no longer a flat per-tier value). Changed the "Tunnel Area Radius" row from 3/4/5 (10x10/12x12/14x14) to 2/3/4 (8x8/10x10/12x12) — every tier's Tunnel Area is now smaller than before. Added a new "Tunnel Space" subsection under Tunnel Area with the full formula, tunnel-region/unused definitions, and the overflow-handling rule.
- `factions.md`: Updated the Syndicate faction blurb and the `SyndicateResources > Tunnel Space` description to reference the new formula instead of "each Tunnel provides Tunnel Space based on its upgrade level." The HUD display format (`current used / current available (max 200)`) is unchanged.

## QA Instructions

1. Start a Syndicate game with the starting Tier 1 Tunnel + pre-built Hideout. Confirm the starting Tunnel Space reflects: Tier 1 tunnel-region (48 grid units) minus the Hideout' 2x2 footprint (4 grid units) = 44 unused grid units / 4 = 11 Tunnel Space available.
2. Build a second Hideout (or any future expansion) in the same Tunnel's Tunnel Area. Confirm Tunnel Space drops by 1 for every 4 grid units the new expansion occupies (a 2x2 expansion should drop capacity by 1).
3. Build a second Tunnel. Confirm total Tunnel Space increases to include that Tunnel's own unused tunnel-region (Tier 1 = up to 48 grid units / 4 = 12 more, before any expansions are built in it).
4. Upgrade a Tunnel to Tier 2. Confirm its tunnel-region grows (84 grid units total tunnel-region at Tier 2 minus whatever's already occupied by existing expansions), increasing total Tunnel Space accordingly — NOT a flat +10 like the old table implied.
5. With Tunnel Space near its cap, build enough units to exceed capacity via a scenario where an expansion is placed after units are already trained (e.g., train units to 10/11, then build an expansion that drops capacity to 8). Confirm existing units remain alive and controllable, the HUD shows something like `10/8`, and new unit production is blocked until usage drops back at or under capacity (e.g., by losing units or building more Tunnels).
6. Confirm total Tunnel Space never displays/allows above 200 regardless of how much unused tunnel-region exists across all Tunnels.
7. Confirm Tunnel Area footprints themselves are now 8x8 (Tier 1), 10x10 (Tier 2), 12x12 (Tier 3) — smaller than the previous 10x10/12x12/14x14 — affecting how much room is available to place expansions and the Tunnel Area Non-Overlap Rule's spacing.


> Note (designer, 2026-10-18): the Syndicate "Headquarters" expansion has been renamed to "Hideout". All references in this message were updated accordingly — implement under the name Hideout.
