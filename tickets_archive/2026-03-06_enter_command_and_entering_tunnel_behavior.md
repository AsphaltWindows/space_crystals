# Ticket: Enter Command and EnteringTunnel Behavior

## Current State
The unit command system defines 8 commands (Move, Attack, AttackGround, AttackMove, Patrol, HoldPosition, Stop, Reverse). There is no mechanism for Syndicate units to enter Tunnel structures. The behavior system has 9 base behaviors with no tunnel-entry behavior.

## Desired State

### Enter Command (9th command)
Add a 9th command to the unit command table:

| Command | Sets CommandType | TargetLocation | TargetObject | Availability |
|---------|-----------------|----------------|--------------|-------------|
| Enter | Enter | None | Tunnel (ObjectInstance) | Syndicate units only, target Tunnel tier sufficient for unit's base category |

- The Enter command is issued when a Syndicate unit is ordered to enter a specific Tunnel.
- Sets `BaseCommandState`: `CommandType=Enter`, `TargetLocation=None`, `TargetObject=Tunnel (ObjectInstance)`.
- **Availability gating**: The command should only be available/issuable when:
  1. The issuing unit belongs to the Syndicate faction.
  2. The target Tunnel's tier meets the transit requirement for the unit's base category (per `features/syndicate_objects.md` tier transit rules).

### EnteringTunnel Behavior (10th base behavior)
Add a 10th base behavior:

**EnteringTunnel**: Unit moves to the target Tunnel's Side A position using `MovingToObject` sub-behavior (targeting Side A). On arrival at Side A:
1. Unit is removed from the map (despawned from the game world).
2. Unit is added to the Tunnel Network's unit pool.
3. The behavior completes.

This behavior has no attack scanning, no leash distance, and no looping — it is a one-way transition from the map to the Tunnel Network.

## Justification
The Enter command and EnteringTunnel behavior are specified in `features/unit_commands_and_behaviors.md` (line 24 for the command, line 73 for the behavior). They are essential for the Syndicate faction's Tunnel Network mechanics defined in `features/syndicate_objects.md`. Without Enter, Syndicate units have no way to return to the underground network after being ejected or produced on the surface. The corresponding control system integration (right-click on own Tunnel issues Enter) is covered by the existing `basic_combat_unit_interface_state` ticket update from `feature_updates/2026-03-06_control_system_enter_command.md`.

## QA Steps
1. Spawn a Syndicate player with at least one Tunnel (T1) and one Agent unit on the surface.
2. Select the Agent and issue an Enter command targeting the Tunnel (right-click on own Tunnel, or via command panel if available).
3. Verify the Agent walks toward the Tunnel's Side A position.
4. Verify the Agent arrives at Side A and is removed from the map (entity despawns, no longer visible or selectable).
5. Verify the Tunnel Network's unit pool now contains the Agent.
6. Attempt to issue Enter on a Tunnel whose tier is insufficient for the unit's base category (e.g., a HeavyVehicle unit on a T1 Tunnel if T1 doesn't support HeavyVehicle).
7. Verify the Enter command is rejected/unavailable — the unit does not move toward the Tunnel.
8. Attempt to issue Enter with a non-Syndicate unit (e.g., a GDO Peacekeeper) targeting a Syndicate Tunnel.
9. Verify the Enter command is rejected/unavailable for non-Syndicate units.
10. Issue Enter on a valid Tunnel while the unit is already moving — verify the unit redirects to the Tunnel's Side A.

## Expected Experience
When a Syndicate player right-clicks their own Tunnel with a unit selected, the unit should begin walking toward the Tunnel entrance (Side A). On reaching the entrance, the unit should visually disappear from the map — it has entered the underground network. If the player tries to Enter with an ineligible unit (wrong faction or insufficient Tunnel tier), nothing should happen — the command should not be issued. The transition should feel clean: the unit walks up to the Tunnel and vanishes, with no lingering entity or visual artifact.
