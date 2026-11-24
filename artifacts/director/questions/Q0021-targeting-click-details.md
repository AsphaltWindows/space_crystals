# Q0021: Targeting click details

- **Date**: 2026-11-23T21:40:31-05:00
- **Blocking**: no. Targeting is built with the behaviour listed under each point.
- **Docs**: changes 0065, 0067, 0068, 0069; `factions/gdo/units.md > Peacekeeper > Riot Shield`; Q0020

## Question
1. **A targeting click where the mouse moves a little.**
   - 0067 says dragging while targeting does nothing. 0068 says the target is taken where the button is pressed.
   - Today, a mouse that moves more than 4 pixels between press and release counts as a drag, so the click is ignored.
   - A slightly sloppy click on a moving enemy is therefore lost.
   - **Proposal:** box selection isn't possible while targeting, so any press and release counts as a click at the press point, however far the mouse moves. Alternatively, keep the drag rule with a larger allowance.
2. **Shielded units in Attack targeting.** The Peacekeeper doc only covers right-click: with the shield up, a right-click on an enemy moves the unit there. When the player instead presses A and clicks an enemy, shielded units move to the clicked spot and the other units attack. Is that right? Should shielded units ignore the order instead?
3. **Recalling a control group while targeting.** By 0069, changing the selection cancels targeting. Recalling a group that gives exactly the same selection changes nothing, so targeting stays active. Should any group recall cancel targeting?

## Context
These came up while building the targeting updates (M1-H2). Point 1 is the most noticeable. Players click fast, and the mouse often moves a few pixels during a click.

## Options
- **Point 1:** A, any press counts as a click at the press point; B, the drag rule with a larger allowance; C, the current 4-pixel rule.
- **Point 2:** A, shielded units move (current); B, shielded units are left out of the order.
- **Point 3:** A, only an actual change cancels (current); B, any recall cancels.

## Update (2026-11-23)
- **Point 1** is answered by 0073.
- **Point 2.** After 0074, shielded units that are right-clicked onto an enemy unit follow it. Attack targeting uses the same rule, so after M1-H3, pressing A and clicking an enemy unit makes shielded units follow it while the others attack. That replaces "move to the clicked spot". The options for point 2 are now: A, shielded units follow (current); B, shielded units are left out of the order.
- **Point 3** is unchanged.
