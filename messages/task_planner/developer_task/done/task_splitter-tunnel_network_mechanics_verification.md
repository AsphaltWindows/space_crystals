# tunnel-network-mechanics-verification

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
