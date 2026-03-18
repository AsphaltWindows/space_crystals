# Ticket: Faction DisplayHud Configuration

## Current State
No faction-specific HUD display exists. Players have no visibility into their resource state.

## Desired State
Implement a per-faction DisplayHud that shows the active player's resources:

**GDO DisplayHud**:
- Space Crystals: current amount
- Supplies: current amount
- Power: current / total (where current = total generated - total consumed)
- Unit Control: used / 200

**Syndicate DisplayHud**:
- Space Crystals: current amount
- Supplies: current amount
- Tunnel Space: used / available (max 200)

**Cults DisplayHud**:
- Space Crystals: current amount
- Unit Control: used / available

**Colonists DisplayHud**:
- Space Crystals: current amount
- Alloys: current amount
- Essence: current amount
- Conduits: current amount
- Beacon Capacity: used / available (max 200)

The HUD must:
- Automatically display the correct resource layout based on the local player's faction.
- Update resource values in real time as they change.
- Distinguish between stockpile display (single number) and capacity display (used/available or current/total).

## Justification
Required by `features/factions_and_resources.md` (DisplayHud sections for each faction). The resource HUD is the player's primary feedback mechanism for economic decision-making.

## QA Steps
1. Start a game as GDO. Verify the HUD shows exactly: Space Crystals, Supplies, Power (current/total), Unit Control (used/200).
2. Start a game as Syndicate. Verify the HUD shows exactly: Space Crystals, Supplies, Tunnel Space (used/available).
3. Start a game as Cults. Verify the HUD shows exactly: Space Crystals, Unit Control (used/available).
4. Start a game as Colonists. Verify the HUD shows exactly: Space Crystals, Alloys, Essence, Conduits, Beacon Capacity (used/available).
5. Gather resources during gameplay and verify stockpile values update in real time.
6. Build/destroy cap-providing structures and verify capacity values update in real time.
7. For GDO, place generators and consumers and verify Power current/total reflects the net sum correctly.
8. Verify no faction shows resources belonging to another faction.

## Expected Experience
Upon starting a game, the player immediately sees their faction's resource display in the HUD. Resource values change smoothly as the player gathers, spends, and manages resources. Stockpile resources show a single number (e.g., "Space Crystals: 1500"). Capacity resources show a ratio (e.g., "Unit Control: 45/200"). The display is always accurate and updates without delay.
