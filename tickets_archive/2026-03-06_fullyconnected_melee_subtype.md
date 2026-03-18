# Ticket: Add FullyConnected Melee Subtype

## Current State
The FullyConnected attack type exists as a single variant with no subtypes. All FullyConnected attacks use a standard numeric Range value, and ElevationModifier applies uniformly to attack range. There is no distinction between ranged and melee FullyConnected attacks.

## Desired State
FullyConnected has two subtypes — **Ranged** and **Melee** — distinguished by range behavior:

- **Ranged**: Standard FullyConnected behavior. Uses a numeric Range value. ElevationModifier applies to range (benefits from elevation advantage).
- **Melee**: Close-quarters variant. Fixed short range defined as adjacent contact (attacker silhouette touching target). ElevationModifier does NOT apply to range. All other FullyConnected properties remain unchanged (CanMiss=false, UnitTarget only, unified animation+effect in Firing phase).

The subtype should be represented in the attack type system so that:
1. FullyConnected attacks declare whether they are Ranged or Melee.
2. Range calculation logic checks the subtype and skips ElevationModifier for Melee.
3. Melee range uses a fixed short-range value (concrete implementation rule for "adjacent contact" is an open question — use a small fixed numeric range as a placeholder until the design resolves this).

## Justification
Feature update `feature_updates/2026-03-06_combat_system_melee_subtype.md` adds FullyConnected subtypes to `features/combat_system.md` (lines 42-51). The Syndicate Agent unit (per `design_updates/2026-03-06_agent_unit_and_melee_subtype.md`) uses Melee FullyConnected, making this a prerequisite for Syndicate unit implementation.

## QA Steps
1. Inspect the FullyConnected attack type definition — verify it supports a Ranged/Melee subtype distinction.
2. Create or find a unit with a FullyConnected Ranged attack. Verify its range calculation includes ElevationModifier when the attacker is at a higher elevation than the target.
3. Create or find a unit with a FullyConnected Melee attack. Verify its range is a fixed short value (adjacent contact / small constant), NOT modified by elevation.
4. Verify a Melee unit can still attack a target at adjacent range (CanMiss=false, UnitTarget only).
5. Verify a Melee unit cannot attack targets beyond its fixed short range.
6. Run `cargo build` — no compilation errors.
7. Run `cargo test` — all existing tests pass, plus any new tests for subtype behavior.

## Expected Experience
When inspecting the code, FullyConnected attacks clearly declare Ranged or Melee subtype. In gameplay, a Melee unit engages only at very close range without elevation range bonuses, while a Ranged FullyConnected unit benefits from elevation as before. No change to HeadDisjointed, TailDisjointed, or DoublyDisjointed attack types.
