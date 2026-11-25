# Q0086 — The EjectMenu's tiles, and when Tunnel Areas are drawn

- **Date**: 2026-11-25T06:54:45-05:00
- **Blocking**: no. M4-J2 built option (a) of each point below, marked as a placeholder in the code.
- **Docs**: `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`, `> EjectMenu`, `> Tunnel Network`; `factions/syndicate/units/README.md > Files`; `systems/controls/selection_and_hud.md > CommandIssuingTransition`

## Question
Five small points about the Tunnel's panel, from building it (M4-J2):
1. Which order should the EjectMenu's unit types follow?
2. Does the EjectMenu show tiles for unit types that have no units in the network?
3. After a click ejects a unit, does the panel stay in the EjectMenu?
4. When are Tunnel Areas drawn on the map?
5. Recalling a control group whose units are all inside the network: does that empty the selection, or change nothing?

## Context
1. **The order.** `tunnels.md > Tunnel ObjectInterfaceState` (0257) says: "unit types in the order of `units/README.md` (Agent, Guard, Grenadier, …)". But `units/README.md > Files` lists the Guard, then the Grenadier, then the Agent.
2. **Empty types.** `> EjectMenu` says the grid shows "unit type tiles representing all units currently in the Tunnel Network". 0257 says "entries take the grid slots in order (Q, W, E, R, A, S, …)" and "unavailable entries are greyed".
3. **After a click.** `> EjectMenu` says a click "ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition)". It names no state after the click. The AugmentMenu (`factions/cults/augmentation.md`) says explicitly that it "stays in this menu so further clicks queue more"; AwaitingTarget returns to DefaultState.
4. **Drawing the areas.** The design has rules about Tunnel Areas: non-overlap, expansions within the area, and enemies' areas remembered at their last-seen tier (0237, 0258). But it never says when the player sees an area drawn on the map. The build ghosts for Tunnels (J1) and expansions (F) will show the areas while placing, since red tiles over a known area need explaining.
5. **All-held groups.** `tunnels.md > Tunnel Network` says a unit that enters "stays in its control groups (a recall selects only the members on the map)". With no members on the map, it's unclear whether the recall selects nothing (an empty selection) or does nothing.

## Options
1. **The order:**
   - (a) **Built:** the parenthetical, so Agent Q, Guard W, Grenadier E, then the README's order (Assault Bike, Shadebringer, …). The README's list would need reordering, or the sentence would need to stop naming the README.
   - (b) The README's order, so Guard Q, Grenadier W, Agent E.

   Either way it is one ordered list in the sim's data (`defs::EJECT_MENU_ORDER`).
2. **Empty types:**
   - (a) **Built:** every Syndicate unit type keeps its slot, showing a count of 0 and greyed. The letters never move.
   - (b) Only the types inside get tiles, packed in order. The letters then shift as types come and go.
3. **After a click:**
   - (a) **Built:** the panel stays in the EjectMenu, so repeated clicks eject more, as in the AugmentMenu. The tile's count drops at each click.
   - (b) It returns to DefaultState after each click.
4. **Drawing the areas:**
   - (a) **Built:** areas are drawn while one of your own Tunnels is the ActiveGroup. They are also drawn while placing a Tunnel or an expansion (J1 and F). Your own areas show at their current tier. Known enemy areas show at their live tier when seen, or at their last-seen tier when remembered. A Tunnel the viewer has never seen, unexplored tiles and off-map tiles are never drawn.
   - (b) Always, for your own Tunnels.
   - (c) Only while placing.
   - (d) A toggle key.

   The colours (own violet, enemy red) are placeholders for the art pass.
5. **All-held groups:**
   - (a) **Built:** nothing changes: not the selection, the panel or the double-tap camera. This is what recalling an empty group does today.
   - (b) The selection becomes empty.
