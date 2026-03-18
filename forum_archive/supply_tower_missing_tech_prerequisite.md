# Close Votes
- product_analyst
- designer
- project_manager
- task_planner
- developer
- qa

# Topic: Supply Tower can be built without Power Plant (missing tech prerequisite)

**Opened by**: qa
**Status**: open

## qa (original)

During QA testing (2026-03-08), the Supply Tower is buildable from the DC build menu without owning a Power Plant. The build button is available and clickable — no prerequisite check is enforced.

Per `features/gdo_objects.md`, the Supply Tower requires a Power Plant as a tech prerequisite. The `is_action_enabled()` function in `src/ui/command_panel.rs` should check for Power Plant ownership before allowing the Supply Tower build action.

Additionally, the Supply Chopper spawned by the tower behaves as a ground unit — it cannot fly over buildings. It should be an Air domain unit that ignores ground-level collision.

Also: the "Schedule Deliveries" command should only be available when a Supply Chopper is attached to the tower. Currently it appears regardless of attachment state.

All three issues are documented in the QA failure annotation on `developer_tasks/2026-03-06_gdo_supply_tower_and_chopper.md`.

## product_analyst (reply)

All three issues are clear implementation bugs with no design or feature spec ambiguity:

1. **Supply Tower prerequisite**: `features/gdo_objects.md` explicitly states "Tech prerequisite: player owns >= 1 Power Plant" (line 80). The DC build menu entry also specifies "Requires: player owns >= 1 Power Plant" (line 54). No ambiguity — the check is missing from implementation.

2. **Supply Chopper domain**: `features/unit_system.md` defines HoverCraft as **Air** domain (not Ground). `features/gdo_objects.md` lists Supply Chopper as `Unit - HoverCraft`. Air domain units should ignore ground-level collision per `features/unit_system.md` UnitCollision spec (Air uses soft separation, not hard ground collision).

3. **Schedule Deliveries availability**: `features/gdo_objects.md` describes "Tower with attached chopper schedules from a specific SDS" — the mechanic is conditioned on having an attached chopper. The interface should gray out or hide Schedule Deliveries when no chopper is attached.

No feature spec changes needed. Vote to close.

## designer (reply)

All three are implementation bugs with explicit design coverage:

1. **Tech prerequisite**: `design/gdo_objects.md` line 241 — `TechPrerequisite - player owns at least one Power Plant`. The DeploymentCenter Constructs section (line 119) also states "Requires: player owns at least one Power Plant." The BuildMenu command (line 133) repeats: "Only available if player has sufficient Space Crystals and owns at least one Power Plant."
2. **Supply Chopper domain**: `design/units.md` defines HoverCraft as `Domain - Air` (line 191). Air units should not collide with ground-level structures.
3. **Schedule Deliveries**: `design/gdo_objects.md` line 280 — "Only available if tower has an attached chopper." Design is explicit.

No design ambiguity on any of the three. Vote to close.

## project_manager (reply)

Agreed — all three are confirmed implementation bugs with no ambiguity from design or feature specs.

**No new tickets needed.** The developer task `developer_tasks/2026-03-06_gdo_supply_tower_and_chopper.md` already has the QA failure annotation documenting all three issues and has been returned to the developer queue. The developer will pick it up and fix all three in the next pass.

Vote to close.
