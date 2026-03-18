# Ticket: Entity Hierarchy and Invisible Entities

## Current State
No entity type hierarchy exists in the codebase. There are no types representing the base Entity concept, Factions, or Players.

## Desired State
Define the foundational entity hierarchy as Bevy ECS components and marker types:

- **Entity base**: A marker or trait distinguishing game entities, with a `Visible` boolean property.
- **Invisible Entity** (Visible=false): Marker for abstract/non-visual entities.
  - **Faction**: Component with `FactionEnum` value, `Name` (string), and `DisplayHud` (keyed by FactionEnum).
  - **Player**: Component with `Name` (string), `Faction` (FactionEnum), `PlayerNumber` (number), and `DisplayHudInfo` (keyed by Faction).
- **Visible Entity** (Visible=true): Marker for entities with on-screen representation, with a `Selectable` boolean property.
- **FactionEnum**: Enum type (variants to be defined by future faction-specific features; for now, define as an extensible enum placeholder).

The hierarchy relationships (Entity -> Invisible/Visible, Invisible -> Faction/Player) should be expressed through Bevy component composition, not Rust struct inheritance.

## Justification
Required by `features/entity_system.md` (Entity Hierarchy section). Every other entity type in the game builds on this hierarchy. Factions and Players are invisible entities that own and control visible entities.

## QA Steps
1. Verify that `FactionEnum` exists and can be instantiated.
2. Verify that a Faction entity can be spawned with `FactionEnum`, `Name`, and `DisplayHud` components.
3. Verify that a Player entity can be spawned with `Name`, `Faction`, `PlayerNumber`, and `DisplayHudInfo` components.
4. Verify that marker components exist to distinguish Visible from Invisible entities.
5. Write a unit test that spawns a Faction and a Player, then queries for all invisible entities and confirms both are returned.
6. Write a unit test that confirms a Visible entity marker is distinct from an Invisible entity marker.

## Expected Experience
All unit tests pass. Spawning Faction and Player entities with their required fields compiles and runs without errors. Querying by visibility markers correctly filters entities.
