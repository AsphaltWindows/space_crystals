## 2026-03-19 — Non-interactive forum pass

- **Forum topics reviewed**: 6 open topics (all operator-created, directed at designer)
- **Votes to close**: 6/6 — topics 1-4 and 6 are design reviews outside QA domain; topic 5 received a comment before close vote
- **Comment on topic 5** (visual bugs & QA infrastructure): Noted that automated QA re-tagging rules are sound, flagged dependency on `automated_qa_ui_state_queries` for [auto]-tagged UI state checks
- **Pending qa_items**: 0
- **Action taken**: Forum pass only, no QA sessions conducted

## 2026-03-19 — Non-interactive forum pass (second run)

- **Forum topics reviewed**: 6 open topics
- **Comments added**: 5 topics (1-5) — provided QA perspective on QA step coverage, tagging appropriateness, and cross-feature dependencies
- **Votes to close**: 6/6 — all topics reviewed, QA input provided, awaiting designer action
- **Pending qa_items**: 0
- **Action taken**: Forum pass only, no QA sessions conducted

## 2026-03-21T12:20:00Z — factions_resources QA

**QA Item**: `qa_router-factions_resources.md`

**Results**:
- Step 1 (GDO HUD): PASS
- Step 2 (Power system updates): PASS
- Step 3 (Power deficit slowdown): PASS
- Step 4 (Unit Control cap): BLOCKED — Extraction Facility not buildable, can't gather resources
- Step 5 (Syndicate HUD): PASS
- Step 6 (Tunnel Space per tier): PASS
- Step 7 (Cults HUD): BLOCKED — faction not available
- Step 8 (Colonists HUD): BLOCKED — faction not available

**Actions taken**:
- Built QA artifact manually (build_qa_artifact.sh broken due to missing diagnostics feature)
- Forum topic filed: build-qa-artifact-missing-diagnostics-feature
- Forum topic filed: cannot-build-extraction-facility
- Forum topic filed: syndicate-camera-not-centered-on-starting-tunnel
- Rework request sent: factions_resources_r1 (scoped to Unit Control cap, Cults, Colonists)
- QA item moved to done

## 2026-03-21T12:35:00Z — scale_camera_system QA

**QA Item**: `qa_router-scale_camera_system.md`

**Results**:
- Step 1 (Simulation tick rate 16 FPS): PASS (code verified)
- Step 2 (Structure grid snapping): PASS
- Step 3 (SpaceUnit/GridUnit ratio): PASS
- Step 4 (Camera 28 GridUnits horizontal): PASS
- Step 5 (No zoom controls): PASS
- Step 6 (HUD layout top/bottom): PASS
- Step 7 (Window resize behavior): PASS

**All steps passed. QA item moved to done.**

## 2026-03-21T12:45:00Z — tile_terrain_system QA

**QA Item**: `qa_router-tile_terrain_system.md`

**Results**:
- Step 1 (All 5 tile presets visible): PASS
- Step 2 (Distinct visual textures): PASS
- Step 3 (Building placement restrictions): PASS
- Step 4 (Ground unit traversal rules): PASS
- Step 5 (Elevation values 0-16): FAIL — no visible elevation, tiles non-selectable so can't inspect
- Step 6 (Elevation rendering at heights): FAIL — no height differences visible
- Step 7 (Rugged terrain marking): PASS

**Actions taken**:
- Rework request sent: tile_terrain_system_r1 (scoped to elevation rendering only)
- QA item moved to done

## 2026-03-21T12:55:00Z — unit_bases_movement_collision QA

**QA Item**: `qa_router-unit_bases_movement_collision.md`

**Results**:
- Step 1 (LightInfantry rugged + turn in place): PASS
- Steps 2-6 (WheeledVehicle, TrackedVehicle, HoverVehicle, HoverCraft, Glider): BLOCKED — units not available
- Step 7 (Ground hard collision): PASS
- Steps 8-10 (Air separation, turret rotation, directional armor): BLOCKED — units not available

**Actions taken**:
- Only LightInfantry available in-game; 2 steps passed, 8 blocked
- Rework request sent: unit_bases_movement_collision_r1 (scoped to missing unit types and associated mechanics)
- QA item moved to done

## 2026-03-21T13:05:00Z — control_state_selection QA

**QA Item**: `qa_router-control_state_selection.md`

**Results**:
- Step 1 (Click to select): PASS (noted: building selection hitbox too small)
- Step 2 (Drag-box own units): PASS
- Step 3 (Drag-box priority units over buildings): PASS
- Step 4 (Drag-box enemy single-select): PASS
- Step 5 (Enemy selection exclusivity): PASS
- Step 6 (SelectionGroups + Tab cycling): PASS
- Step 7 (Control group assign & recall): PASS
- Step 8 (Control group add, no duplicates): PASS
- Step 9 (Destroyed unit removed from group): PASS
- Step 10 (Ungroupable objects own groups): PASS

**Bugs found during session**:
- Forum topic filed: enemies-dont-attack-by-default
- Forum topic filed: can-control-enemy-units-and-buildings
- Noted: building selection hitbox smaller than visual

**All steps passed. QA item moved to done.**

## 2026-03-21T13:10:00Z — Session end

**Session summary**: QA'd 4 items, completed 4, returned 1 to pending.
- factions_resources: 5 pass, 3 blocked -> rework sent
- scale_camera_system: 7 pass -> complete
- tile_terrain_system: 5 pass, 2 fail -> rework sent (elevation rendering)
- unit_bases_movement_collision: 2 pass, 8 blocked -> rework sent
- control_state_selection: 10 pass -> complete
- pointer_display_types: returned to pending (user ended session)

**Forum topics filed this session**: 5

**Rework requests sent**: 3

**Remaining pending QA items**: ~33
