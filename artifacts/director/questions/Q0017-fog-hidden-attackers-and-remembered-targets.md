# Q0017: Fog: shots from hidden attackers, and targeting remembered structures

- **Date**: 2026-11-23T20:50:00-05:00
- **Blocking**: no. Work proceeds on the assumptions listed under each point.
- **Docs**: `core/vision.md > Explored`, `> Explored Selectability`; `systems/controls/commands.md` (Attack, right-click)

## Question
1. **What the player sees when a hidden attacker shoots a visible unit.** Today a shot's tracer is drawn only when both ends are on Visible tiles. A unit shot from the fog loses HP and nothing shows where the shots come from.
   - **Assumption:** the hit unit flashes, but no tracer is drawn, so the attacker's position stays secret.
   - **Alternative:** draw the visible part of the tracer, from the fog edge to the target. That hints at the attacker's direction.
2. **Targeting a remembered structure.** In Explored areas, structures and resources show in their last-known state. They are not selectable, except supply stations for GDO.
   - Can the player right-click or Attack such a remembered structure? Units would walk there and attack it if it is still standing.
   - **Assumption:** no. The command gate accepts object targets only when the player currently sees them, plus the GDO station exception. A right-click on a remembered structure is a Move to that spot.
   - **Alternative:** remembered structures can be targeted. The order goes to the last-known position, and attacks once the target is seen again.

## Context
Vision and fog of war are built (M1-V). The display of last-known structures and the target checks in the command gate are being built now.

## Options
- **Point 1:** A, a hit flash only; or B, the visible part of the tracer.
- **Point 2:** A, visible targets only; or B, remembered structures can be targeted.
