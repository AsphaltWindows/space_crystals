# Developer Insights

## Code Organization
- `src/game/` — core game logic (combat, units, world, types)
- `src/ui/` — HUD, command panel, menu (7 files currently at limit)
- `src/shared/` — shared types, utils, testing infrastructure
- `src/simulation/` — diagnostics, instrumentation, overlay

## Key Files
- `src/ui/command_panel.rs` — `update_command_panel_state()`, `is_common_command()`, DcConstructing/EfConstructing state logic, DomainEnum references
- `src/game/world/map.rs` — likely contains `rebuild_occupancy_map`
- `src/game/units/systems/commands.rs` — unit command handling
- `src/game/units/types/state/` — behavior and command state types

## Patterns
- Unit types defined via ObjectEnum variants + type_data()/attack_data() functions + spawn_*() functions
- Interface states are enum-based state machines (DcIdle, DcConstructing, etc.)
- Production systems follow Barracks pattern (tick system, queue, spawn)
- Ungroupable entities (Tunnels, Agents) each form their own SelectionGroup

## Known Issues (from forum review)
- `rebuild_occupancy_map` doesn't filter by DomainEnum — underground structures block surface movement
- `is_common_command()` hardcodes command classifications without checking selection composition
- `update_command_panel_state()` forces construction sub-menu state every frame
- Right-click only handles ground clicks, no entity detection
- Persistent horizontal black line in viewport (likely UI element, not camera artifact)
