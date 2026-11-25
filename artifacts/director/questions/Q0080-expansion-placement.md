# Q0080 — Expansion placement: ground units, lingering ghosts, and Warp In over a hidden expansion

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: `core/entities.md > Direct Placement` read literally (ground units block; own ghosts reserve), and Warp In judged like direct placement.
- **Docs**: `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`; `factions/syndicate/structures/README.md > Tunnel Expansions`; `core/entities.md > Placement Validation > Direct Placement`; `factions/colonists/economy.md > Warp In`

## Question
1. Do **ground units** standing on the tiles block an underground expansion's placement?
2. Do the owner's **lingering build ghosts** (Shift-queued Tunnel orders) reserve tiles against an expansion?
3. A Colonists **Warp In** whose building would cover a hidden enemy expansion: is it judged like a direct placement (0240: red at once), since it starts at the command with no walk?

## Context
- `core/entities.md > Direct Placement` lists "Tunnel underground expansions" as direct placement, and says "**No ground units on the footprint** ...: a ground unit of any player ... standing on any footprint tile makes those tiles invalid ... Air and underground units never block placement."
- The adopted plan proposed that units don't block expansions (Q0062, proposal 3c). 0240 answered the layers question without that point, so the proposal fell with it.
- An expansion is underground, so a unit standing above it is never in its way physically.
- 0240: a direct placement over a hidden expansion is red at once ("accepted"); a worker-built one judges by knowledge and fails on arrival, and 0240's verification covers a Drone's deploy. A Warp In is neither: the Beacon transforms in place at the command, so there is no arrival at which to fail.

## Options
1. **Ground units:**
   - **A (proposed, literal).** They block, as for any direct placement. A player may have to move units off the spot first.
   - **B.** They don't block expansions, since they're on another layer. One exception in `Method::Expansion`.
2. **Lingering ghosts:**
   - **A (proposed).** They reserve their tiles against everything, expansions included, as M3 does for other placements.
   - **B.** Expansions ignore them, and the Tunnel's order fails on arrival.
3. **Warp In over a hidden expansion:**
   - **A (proposed).** Judged like direct placement: the ghost is red at once. This is 0240's accepted leak, for the Colonists too.
   - **B.** The ghost judges by knowledge and stays green; the command is refused when given, with feedback. The player learns the same thing one click later, so this only adds a step.
