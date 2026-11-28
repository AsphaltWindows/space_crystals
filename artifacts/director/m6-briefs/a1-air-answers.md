# Task A1: A queued order starts when the aircraft lands (0466); a shorter Aim (0470); the settled Airfield and transport placeholders (0465, 0469)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, and your budget is 200 tool calls.**

## The architecture
The core, then the Airfield paragraphs of `docs/architecture/production.md` only: `grep -n -i 'airfield\|rearm\|landing' docs/architecture/production.md`.

## The design
- `factions/gdo/structures/airfield.md > Returning and rearming` and `> Landing slots`, as they stand now.
- The change entries in `artifacts/designer/changes/`:
  - `0465-airfield-landing-details-rearm-command.md`;
  - `0466-airfield-queue-after-landing-name-kept.md`, which replaces 0465's part 2 and its rename (the command keeps the name Return to Base);
  - `0469-transport-unloading-rules.md`.

## Goal
1. **0466: an order queued behind a return starts on landing.** An order Shift-queued behind a Return to Base, or behind the automatic return when the ammo runs out, starts right after the aircraft lands on its slot. It works like an order given during rearming: the aircraft takes off with the ammo rearmed so far. With nothing queued, the aircraft stays landed and rearms fully, as now.
   - **Where.** GH2 made the return end only once the aircraft is rearmed. The mechanics audit confirms that 0466 isn't built yet: `run` in `orders/air_base.rs` (about lines 35–42) ends the flight only after the rearm. The places:
     - `crates/sim/src/orders/air_base.rs`: its module doc (lines 10–12) and `run`;
     - `crates/sim/src/orders/run.rs`, near lines 266 and 341;
     - `crates/sim/src/sim/aircraft.rs`: the automatic return, near line 41;
     - `crates/sim/src/landing.rs`: `rearm`, and the module doc's lines on an order during rearming (36–40).
   - **Keep the rearm going** for a landed aircraft with no order, however the code drives it now.
   - **Tests:**
     - Change GH2's test that a queued order waits for the full rearm to 0466's rule. Find it with `grep -rn -i 'queue\|shift' crates/sim/tests/landing crates/sim/tests/airfield`.
     - Add 0466's Verification:
       1. F, then a Shift-queued Attack Move: the aircraft lands and takes off again at once, with almost no ammo rearmed.
       2. F alone: after 10 s it is fully rearmed and still landed.
       3. A right-click on a different own Airfield with a free slot: the aircraft is reassigned there and lands there. This is GH1's Rebase; reuse a test that covers it if one does.
   - **The client** changes only if one of its tests asserts the old wait.
2. **0465 and 0469: the settled placeholders.** Remove the `Placeholder (not in the design: ...)` marks these answers settle, and cite the design section in their place.
   - **From 0465:**
     - an aircraft on its slot holds fire (`crates/sim/src/aircraft.rs` near line 81, `crates/sim/src/landing.rs` near 48);
     - a landed aircraft is out of the air separation (`landing.rs` near 20);
     - a new aircraft with no free slot waits inside (find its mark);
     - the queued orders behind a Return to Base (`air_base.rs` near 12), which now follow 0466. The automatic return's mark (`sim/aircraft.rs` near 41: the orders queued behind the one the aircraft ran out in) stays, as `Placeholder (Q0195)`: 0466 doesn't settle it. (Corrected at A1's question.)
   - **The slots' layout keeps its mark** (`landing.rs` near 9): it is still a placeholder, now the designer's (`airfield.md > Landing slots`). Cite that section.
   - **From 0469:** where each passenger comes out, and that those with no free spot stay aboard (`crates/sim/src/carry.rs` near 40); who sees the pips; and F on the move. Find their marks with `grep -rn 'Placeholder (not in the design' crates | grep -i 'pip\|unload\|aboard\|move'`.
   - Leave every other mark as it is.
   - **Check that each Verification item below has a test,** and add one where it's missing:
     - 0465 (1): a landed Hornet with enemies in range doesn't fire;
     - 0465 (4): a Hornet flying over a landed one doesn't displace it;
     - 0469 (1): a packed unload leaves the rest aboard, and they come out after a move and another F;
     - 0469 (2): an opponent sees no pips (a client test, if the pips are drawn by the client alone);
     - 0469 (3): F while driving stops the transport, then unloads it.
3. **0470 (answers Q0192): limited-ammo aircraft fire from their base, facing the target.** Read `artifacts/designer/changes/0470-aircraft-fire-from-base-facing-short-aim.md`.
   - **Aim is 2 frames** for the Dragonfly (from 8) and the Hornet (from 4): `aim_ticks` in `crates/sim/src/defs/gdo/units/{dragonfly,hornet}.rs`. These are the designer's placeholders, so mark them as such.
   - There is no launch cone, as built. The landing approach is as built too, now the designer's placeholder (`factions/gdo/structures/airfield.md > Returning and rearming`): GL's marks on it cite that section.
   - **Tests** for 0470's Verification (the Specter's points belong to slice SA):
     1. Hornets and Dragonflies on a Move past enemy ground units and aircraft fire nothing.
     2. On Attack Move they engage.
     3. A Dragonfly attacking a circling Sparrow flies out, comes back head-on and launches within a pass or two. If it can't launch, don't tune it: report how close it came (its closest angle to the target, and over how many passes).
   - Commit this apart from 0466's change, with its own golden re-bless.
4. **Goldens.** A golden with an order queued behind a return moves, and so does one whose Hornets or Dragonflies fire. Re-bless only those that fail, in the commit of the change, and say in your report why each moved (an aircraft that took off earlier). No other golden may move.

## Don't
- Don't rename the command: it stays Return to Base (`PanelCommand::ReturnToBase`, `OrderKind::ReturnToBase`).
- Don't change the other landing rules: the slots, the approach, the Dragonfly's vertical landing.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff**:
- the change and its commit;
- the marks removed;
- the tests added;
- the goldens re-blessed, and why each moved;
- the gate's results.
