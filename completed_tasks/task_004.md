# Task 004: Implement Faction and Player Data Model

## Status
**Completed** - 2026-02-22

Refactored Owner from enum to struct wrapping Option<u8>. Created Player component, GdoPlayerResources with power ratio and unit control cap (200). Added placeholder resources for Syndicate/Cults/Colonists. Removed old FactionResources unified enum and PlayerResources wrapper. Updated all 14 referencing files.

## Description
Implement the Faction and Player entity types matching the design specification. Define GDO-specific resource tracking (Space Crystals, Supplies, Power, Unit Control) and create the Player entity with its faction association and resource state.

## Why Needed
The design defines Faction as an Invisible Entity with a Name, Value (FactionEnum), and a faction-specific DisplayHud. Player is an Invisible Entity with Name, Faction, PlayerNumber, and DisplayHudInfo. The current code has a `FactionMember` component and `PlayerResources` / `FactionResources` enums — these need to be aligned with the formal design, especially the GDO resource model (Space Crystals, Supplies, Power as flat capacity, Unit Control hard cap of 200).

## Acceptance Criteria
- `Player` component with: `name` (String), `faction` (FactionEnum), `player_number` (u8)
- `GdoPlayerResources` struct/component with:
  - `space_crystals: i32` (current amount)
  - `supplies: i32` (current amount)
  - `power_generated: i32` (sum of positive Power from owned buildings)
  - `power_consumed: i32` (sum of negative Power from owned buildings, stored as positive)
  - `unit_control_used: u32` (current units' total control cost)
  - `unit_control_cap: u32` (always 200)
- A method or function to compute `current_power() -> i32` (generated - consumed)
- A method or function to compute `power_ratio() -> f32` (for proportional slowdown: if power is negative, ratio = generated / consumed; if positive, ratio = 1.0)
- `Owner` component aligned with design: stores `Option<u8>` (PlayerNumber) where `None` means unowned/neutral
- Replace existing `PlayerResources` and `FactionResources` with the new design-aligned types
- Existing systems that reference player resources updated
- Project compiles and runs

## Relevant Files/Components
- Current faction module — has `Faction`, `FactionMember`, `PlayerResources`, `FactionResources`, `GdoResources`
- Current units module — has `Owner` enum

## Technical Considerations
- The current `Owner` enum has `Player(u8)` and `Neutral` variants with color mapping. The design uses `PlayerNumber | None`. Keep the color mapping utility but align the core type with the design.
- The current `GdoResources` struct has `space_crystals`, `supplies`, `power_generated`, `power_consumed` — this is close to what's needed. Add `unit_control_used` and `unit_control_cap`.
- Power is a flat capacity system: each building has a static Power value (positive=generator, negative=consumer). Total Power = sum across all owned buildings. If negative, all consumers operate slower proportionally.
- Unit Control is a hard cap of 200, always fully available (no buildings needed to unlock it).
- For now, only GDO resources need full implementation. Other faction resources can be defined as placeholder structs.
- The `setup_player_resources` system currently creates resources for Player 0 (GDO) and Player 1 (Syndicate) — update to use new types.

## Prerequisites
- [ ] `task_001.md` — Directory structure must be in place
- [ ] `task_002.md` — FactionEnum must be defined

## Complexity
Medium
