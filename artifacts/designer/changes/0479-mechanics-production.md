# 0479 — Mechanics doc: Unit Production

- **Date**: 2026-11-28T13:05:00-05:00
- **Answers**: Q0193 (migration, part 7)
- **Docs**:
  - `systems/mechanics/production.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/gdo/overview.md > Unit Control` (the reservation rules are now defined in production.md, for every faction; a summary is kept)
  - `factions/gdo/structures/barracks.md` (intro)
  - `factions/cults/structures/armory.md > Training`
  - `factions/cults/structures/chop_shop.md`, `airstrip.md` (Production)

## What changed and why
The seventh mechanics doc.

**Common rules for every producer:**
- a serial queue;
- paid at queueing, every resource;
- cancel removes the last entry with a full refund;
- at most 20 orders;
- **capacity reserved when production starts**: waiting, a fixed service order, no progress means no start, zero-cost units always pass. This was defined in the GDO overview "for every faction" and now lives in one place;
- a finished unit waits at 100% behind a blocked exit;
- rally points;
- a destroyed producer refunds everything as if cancelled.

**Archetypes:**
- **Queue producer:** the GDO Barracks, Factory and Airfield; Syndicate expansions and Colonists producers via their docs.
- **Pool producer:** the Cults Armory, Chop Shop, Airstrip and Cathedral. Units enter a pool; orders reserve units first in, first used, and the queue is bounded by the pool; they are consumed on completion; what carries over is a parameter; cancel returns the reserved units; Release and Eject All; evacuation on destruction.

**No rule changed.**

## Implementation impact
One production module with two archetypes. No behaviour changes.

## Verification
No gameplay change.
