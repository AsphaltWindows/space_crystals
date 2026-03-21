# tunnel-network-mechanics-verification

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-tunnel-network-mechanics.md

## Task

Verify the existing Tunnel Network mechanics implementation against the design spec. All core systems appear to be implemented — this task is to confirm correctness and fill any minor gaps.

**Already implemented (verify these match the spec):**

1. **Tunnel Structure** (game/types/objects.rs): ObjectEnum::Tunnel — 4x4, ABCD symmetry, destructible=true, groupable=false, SightRange=5
2. **Tier Table** (game/types/structures.rs): TunnelTier enum with constants:
   - HP: 600/800/1000 (TUNNEL_T1/T2/T3_MAX_HP)
   - Armor: PointArmor=1, FullArmor=16 (TUNNEL_POINT_ARMOR, TUNNEL_FULL_ARMOR)
   - Space: 20/30/40 (TUNNEL_T1/T2/T3_SPACE)
   - Area Radius: 3/4/5 (TUNNEL_T1/T2/T3_AREA_RADIUS)
3. **Transit Tier Requirements** (game/types/structures.rs): TransitTier enum — Tier1=Infantry, Tier2=+Vehicles, Tier3=+Air. TunnelTier::can_transit() delegates to TransitTier::allows_unit_base()
4. **Tunnel Area** (game/types/structures.rs): TunnelArea component with new(), recalculate(), overlaps() methods. Non-overlap validation in validate_tunnel_upgrade()
5. **Side Functions** (game/units/systems/behaviors.rs): drop_off_side_for_carry() — Side B for crystals, Side C for supplies. Side occupancy enforcement (one agent per side)
6. **Cost Scaling** (game/types/structures.rs): tunnel_construction_cost() (0, 1, 2...), tunnel_t2_upgrade_cost() (2, 4, 6...), tunnel_t3_upgrade_cost() (3, 6, 9...)
7. **Construction/Upgrade Rules** (game/types/structures.rs): TunnelOperation enum (Upgrading/BuildingExpansion), one operation at a time via current_operation field
8. **Spawn functions** (game/utils.rs): spawn_tunnel(), spawn_tunnel_under_construction() — both with ABCD labels, SightRange, EjectionQueue
9. **InTunnelNetwork marker** (game/units/types/state/behavior.rs): Units inside tunnel network marked with InTunnelNetwork component
10. **Starting Tunnel** (game/world/faction.rs): Syndicate player starts with 1 Tier 1 Tunnel + Headquarters

**Verification steps:**
- Confirm all constants match the spec values listed above
- Confirm transit tier filtering is correct (run existing tests)
- Confirm tunnel area overlap detection works (run existing tests)
- Confirm cost scaling formulas are correct (run existing tests)
- Run `cargo test` and ensure all tunnel-related tests pass
- If any minor discrepancies are found, fix them

## Technical Context

### Files to Verify (with specific locations)

1. **`src/game/types/objects.rs` line 288**: ObjectEnum::Tunnel definition — size (4,4), destructible true, sight_range 5, groupable false. Line 346: symmetry ABCD. **Tests exist**: lines 964-1019 cover name, size, destructible, sight_range, groupable, symmetry, orientation count, size validation.

2. **`src/game/types/structures.rs`**:
   - **Lines 443-453**: Tier constants — HP 600/800/1000, PointArmor=1, FullArmor=16, Space 20/30/40, AreaRadius 3/4/5. All match spec.
   - **Lines 471-501**: `TransitTier` enum and `allows_unit_base()` — Tier1 allows LightInfantry+HeavyInfantry, Tier2 adds Wheeled/Tracked/Drill/HoverVehicle/Mech, Tier3 allows all (includes HoverCraft+Glider). Design spec says "Tier 1+: Infantry (Heavy Infantry)" meaning the Infantry category (both Light and Heavy). Code is correct.
   - **Lines 545-555**: `TunnelTier::transit_tier()` and `can_transit()` — delegates correctly.
   - **Lines 576-612**: `TunnelState` with `current_operation: Option<TunnelOperation>`, `TunnelOperation` enum (Upgrading/BuildingExpansion).
   - **Lines 621-660**: `TunnelArea` struct with `new()`, `recalculate()`, `overlaps()`, `fits_expansion()` methods.
   - **Lines 692-706**: Cost functions — `tunnel_construction_cost(n) = n`, `tunnel_t2_upgrade_cost(n) = 2 + 2*n`, `tunnel_t3_upgrade_cost(n) = 3 + 3*n`. All match spec pattern (0,1,2... / 2,4,6... / 3,6,9...).
   - **Tests** (65+ tunnel tests): Lines 896-1066 (constants, tier mappings, transit filtering), 1071-1115 (state, operations), 1342-1500 (area cells, overlap, cost formulas), 1545-1595 (upgrade validation), 1648-1660 (expansion marker).

3. **`src/game/units/types/state/behavior.rs` line 324**: `InTunnelNetwork { owner_player: u8 }` component. Tests at lines 630-651.

4. **`src/game/units/systems/behaviors.rs` line 594**: `drop_off_side_for_carry()` — crystals→'B', supplies→'C'. Tests at lines 2293-2314 (crystals side B, supplies side C, both prefers crystals, empty defaults to C).

5. **`src/game/utils.rs`**:
   - **Line 688**: `spawn_tunnel()` — spawns with TunnelTier::Tier1, TunnelState::default_tier1(), TunnelArea, SightRange, EjectionQueue, ABCD side labels.
   - **Line 732**: `spawn_tunnel_under_construction()` — same components but with ConstructionHP and under_construction ObjectInstance.

6. **`src/game/world/faction.rs` line 95**: `setup_syndicate_game_start()` — spawns 1 Tier1 Tunnel at grid (40,40) + HQ at (42,38). Matches spec: "starts with one Tier 1 Tunnel and one pre-built Headquarters expansion."

7. **Ejection system** (`src/game/world/faction.rs` ~line 1847): `ejection_tick_system` — uses `EjectionQueue` with 8-frame cooldown, spawns ejected units at Side A, removes `InTunnelNetwork` marker.

### Design Spec Reference
- **`artifacts/designer/design/syndicate_objects.md`** lines 3-55: Full tunnel network spec including tier table, transit tiers, side functions, interface states.

### How to Verify
- Run `cargo test` from `artifacts/developer/` and filter for tunnel tests: `cargo test tunnel` and `cargo test transit`
- The codebase has 65+ targeted unit tests covering every constant, cost formula, transit filter, area calculation, and state transition
- Cross-reference each constant in the code against the tier table in the design spec (lines 39-47 of syndicate_objects.md)
- Check that `TransitTier::Tier1` allows both `LightInfantry` AND `HeavyInfantry` (spec says "Infantry" category — both bases are Infantry per units.md lines 103-117)

### Known Non-Issues (pre-verified during planning)
- Cost formulas produce correct sequences: construction (0,1,2...), T2 upgrade (2,4,6...), T3 upgrade (3,6,9...)
- Transit tier filtering matches spec exactly — Tier1=Infantry, Tier2=+Vehicles, Tier3=+Air
- Armor is flat across all tiers (PointArmor=1, FullArmor=16) — this matches spec
- Side functions: B=crystals, C=supplies — confirmed in code and tests

## Dependencies

None — this is a standalone verification task. The only task in this feature.
