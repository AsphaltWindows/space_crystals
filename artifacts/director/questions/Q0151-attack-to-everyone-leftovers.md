# Q0151 — After "Attack goes to everyone": shielded Peacekeepers, box-selecting Halls, and A on a building's panel

- **Date**: 2026-11-26T15:31:12-05:00
- **Blocking**: no. Slice M5-S5 built 0397–0399 and 0395 on the readings below. Each part is answered on its own.
- **Docs**: `factions/gdo/units/peacekeeper.md` (Riot Shield, the explicit Attack in a mixed selection); `systems/controls/selection.md > BoxSelection`; `systems/controls/interface.md > CommandPanel > Standard Slot Assignments`; design changes 0395 and 0397–0399.

## Question
1. **A Peacekeeper with its shield up, in a mixed selection given an explicit Attack:** is it still left out of the order, or does it move to the target like any unit that can't attack (0398)?
2. **Box-selecting Halls:** now that Recruitment Halls are groupable (0395), should a box select every own Hall in it?
3. **A on a building's panel:** when a building is in focus in a selection that includes units that can attack, should its panel show A: Attack?

## Context
1. **Shielded Peacekeepers.**
   - `peacekeeper.md` (user, 2026-11-24): in a mixed selection's explicit Attack, "Peacekeepers whose shield is up are **left out** of the order and keep their current orders; the units that can attack carry it out. This follows the CommandPanel rule that a command goes only to objects that can carry it out".
   - 0397 and 0398 replaced that rule for Attack: "An Attack is issued to **every selected unit**," and "Non-attackers treat Attack and Attack Move as Move: on ground they move to the spot, on a target they move to it."
   - **As built:** a shielded Peacekeeper now moves to an explicit Attack's target with the rest, and the line in `peacekeeper.md` is stale. Its rules for orders given during the toggle are unchanged.
2. **Halls.**
   - 0395's Verification says "Box-select 2 Halls and set a rally point: both send new Recruits there."
   - `selection.md > BoxSelection` makes own buildings tier 2, "single-select: the own building closest to the box center", and "Only tier 1 (own units) produces a multi-selection."
   - **As built:** a box still selects one Hall. Click one Hall and Shift + box another to select both. The test does it that way.
3. **A on a building's panel.**
   - The rule is written for units: "A is Attack on every unit panel" (0397), shown "whenever at least one selected group can attack, whichever group is in focus" (0399).
   - **As built:** when a building is in focus and its A slot is free, its panel shows the selection's Attack, given to the units as on a unit's panel.

## Options
1. **Shielded Peacekeepers:**
   - (a) As built: they move to the target with the rest, and `peacekeeper.md` is updated to match.
   - (b) They stay left out of an explicit Attack, as an exception to 0397, and keep their orders.
2. **Halls:**
   - (a) As built: a box selects one building, so Halls are selected with clicks and Shift + box.
   - (b) A box selects every own Hall in it, or every own groupable building of one type, as a multi-selection tier between own units and other own buildings.
3. **A on a building's panel:**
   - (a) As built: a building's free A shows the selection's Attack.
   - (b) A shows only on unit panels.
