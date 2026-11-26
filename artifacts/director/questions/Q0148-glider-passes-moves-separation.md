# Q0148 — Gliders: a lone Sparrow makes one pass, a Move never ends, and SeparationRadius is under the Silhouette

- **Date**: 2026-11-26T13:38:18-05:00
- **Blocking**: no. Slice M5-H1 (gliders) built the literal readings below. Each part is answered on its own.
- **Docs**: `factions/cults/units/sparrow.md` (Placeholder stats, Flight rules); `systems/units.md` (SeparationRadius: "Must be larger than the unit's Silhouette"); `factions/gdo/units/dragonfly.md > UnitBase - Glider`; design change 0200 (a unit that loses its target stops); Attack Move's 6 GU chase leash.

## Question
Three things H1 found while building the Sparrow, the first glider:
1. **A lone Sparrow loses its target after one strafing pass.** Is that intended?
2. **A glider's Move never completes**, so orders queued after it never start. Should a Move end when the glider reaches its point?
3. **SeparationRadius (32 su) is smaller than the Silhouette (40 su),** which the units doc forbids. Which number changes?

## Context
1. **One pass.** The Sparrow's full-speed turn radius is 200 su (MaxCentripetalAcceleration 0.5).
   - After a pass, its loop carries it about 470–500 su from the target. Its SightRange is 6 GU (384 su).
   - **On Attack:** a lone Sparrow loses sight of the target mid-loop, and its Attack ends (0200).
   - **On Attack Move:**
     - alone, it loses sight at tick 136;
     - with a spotter keeping the target seen, Attack Move's 6 GU leash drops it at tick 140.

   Either way it gets one pass. The strafing tests use spotters for this reason. The Dragonfly's 330 su turn radius would be worse.
2. **Moves.**
   - A glider can't stop, so at its Move's point it circles, and the Move stays its current order.
   - An order queued after it with Shift never starts, for example Move A then Move B, or Move then Attack Move.
   - Stop and Hold circle the stop point, as 0354/0355 say.
3. **Separation.**
   - `systems/units.md`: SeparationRadius is "the distance at which air-to-air repulsion activates. Must be larger than the unit's Silhouette".
   - `sparrow.md` gives Silhouette 40x40 and SeparationRadius 32 (Q0120).
   - As built, six Sparrows sent to one point keep about 32 su apart, closer than their 40 su silhouettes.

## Options
1. **One pass:**
   - (a) As built: a glider needs a spotter and a re-order for each pass.
   - (b) A glider keeps its attack target through its loop: the target stays while it is within some distance of the glider (for example 2× SightRange) or for some seconds after a pass, sight or no sight.
   - (c) Gliders are exempt from Attack Move's leash, and (b) covers sight.
   - (d) Tighter loops or longer sight, through the numbers (MaxCentripetalAcceleration or SightRange), with the rules as they are.
2. **Moves:**
   - (a) As built: a Move never completes.
   - (b) A glider's Move completes when it first reaches its point. It then circles there as when idle, and the next queued order starts.
3. **Separation:**
   - (a) SeparationRadius goes above 40, for example 48.
   - (b) The Silhouette shrinks below 32.
   - (c) The rule doesn't apply to gliders.
