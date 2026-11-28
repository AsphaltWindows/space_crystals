# 0481 — Mechanics doc: Stocks and Spawned Objects

- **Date**: 2026-11-28T13:40:00-05:00
- **Answers**: Q0193 (migration, part 9)
- **Docs**:
  - `systems/mechanics/spawned_objects.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/gdo/units/drone_operator.md > Drone Stock`
  - `factions/gdo/units/twister.md > Mines`
  - `factions/cults/units/disciple.md > Mirage`

## What changed and why
The ninth mechanics doc.

**Stock and purchase:**
- parameters: maximum, starting stock, cost paid at the press, time;
- purchase either queues up to the free stock (the Twister) or runs only once the previous object has ended (the Drone Operator).

**Common rules for spawned objects:**
- they are the owner's selectable units;
- they use no capacity;
- creating one doesn't change the selection.

**Archetypes:**
- **Piloted object:** the drone. One active at a time; creator restrictions; ends by leash, battery or the creator's death; no recall.
- **Deployed object:** the mine. Not a structure; hidden; a trigger; its effect; removed when destroyed.
- **Copy:** the illusions. What is copied, what they can't do, damage multiplier, owner's tell, when they end.

**No rule changed.** The unit docs keep their parameters and examples, under Uses lines.

## Implementation impact
One module for stocks and spawned objects. No behaviour changes.

## Verification
No gameplay change.
