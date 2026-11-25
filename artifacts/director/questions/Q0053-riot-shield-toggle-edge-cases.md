# Q0053: Riot Shield toggle edge cases: orders while lowering, Lower during a waiting raise, the ActiveGroup during a deferred raise

- **Date**: 2026-11-24T18:24:20-05:00
- **Blocking**: no. M3-S is built with the current behaviour listed under each point.
- **Docs**: `factions/gdo/units.md > Peacekeeper > Riot Shield` (Two unit types, During the toggle, Pressed mid-attack, Orders are kept); changes 0171–0174.

## Question
1. **Orders given to a Peacekeeper while it lowers its shield** (6 ticks). During the toggle it can't attack (0171), and its type is already Peacekeeper, since the type switches when the toggle starts.
   - **Current behaviour:** an explicit Attack leaves that unit out, as it does Shielded units (0075), and a right-click on an enemy makes it follow (0074).
   - Under 0150's "orders are kept, on hold", the Attack could instead be accepted and start once the shield is down.
2. **Lower pressed on a Peacekeeper whose raise is still deferred**, waiting for its shot to finish (0172). This can happen in a group where some units have switched to Shielded and others are still waiting.
   - **Current behaviour:** the press cancels the waiting raise, matching "a second press reverses from the current point" (0171). Q0049 asks the same about Convert, where the current behaviour is the opposite.
3. **The ActiveGroup during a deferred raise.**
   - **Current behaviour:** the Peacekeeper group stays active until its last unit actually switches type, a few ticks later. Then the Shielded group becomes active, per Q0050's current rule.

## Context
All three came up while building 0171–0174 (M3-S).

## Options
- **Point 1:** A, as now (Attack leaves it out; a right-click follows); B, Attack accepted and held until the shield is down.
- **Point 2:** A, as now (cancels the waiting raise); B, ignored (the raise still happens).
- **Point 3:** A, as now; B, the Shielded group becomes active as soon as R is pressed.
