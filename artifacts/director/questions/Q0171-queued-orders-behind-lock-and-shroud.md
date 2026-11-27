# Q0171 — Does a Shift-queued order end a Target Lock or a Shroud channel, or wait behind it?

- **Date**: 2026-11-26T20:08:20-05:00
- **Blocking**: no — proceeding on assumption: a Shift-queued order waits behind the lock or the channel and runs when it ends; any order given without Shift ends it at once (the Director's reading, T76; built for Shroud in M5-N4b, and being built for Target Lock in M5-K).
- **Docs**: `factions/gdo/units/rocketman.md > Target Lock` ("breaks when ... the Rocketman is given **any** order (move, stop, attack something else, etc.)"); `factions/cults/units/disciple.md > Spells > Shroud` ("What ends the channel ... a **cancel** command (Stop or any other order)"); `systems/controls/commands.md > Unit Command` (the order queue); `systems/controls/interface.md > AwaitingTarget[CommandType]` ("Shift while targeting").

## Question
When a player Shift-queues an order, for example Shift+right-click to Move, for a Rocketman that is locking or locked, or for a Disciple channelling Shroud, does it:
- end the lock or the channel at once, as an order given without Shift does; or
- wait in the queue, and run when the lock or the channel ends?

## Context
- Both docs say "any order" ends the lock or the channel. Their examples (move, stop, attack something else) are orders given without Shift, which replace what the unit is doing.
- **Elsewhere, a Shift-queued order never interrupts** the current one: it runs when the current one is done (`commands.md`'s queue).
- **How the lock and the channel end without an order:**
  - A lock ends when its target dies, or leaves LockRange or the Rocketman's vision. A queued order behind a lock on a building therefore runs once the building is destroyed: "kill that, then move".
  - A channel ends when the energy runs out.
- **As built,** Target Lock and Shroud are orders in the unit's queue (T76), so queued orders wait behind them. A Shift+R (Target Lock) after a Move queues the lock to start when the Move is done.

## Options
1. **Queued orders wait** (as built): any order without Shift ends the lock or the channel, and a Shift-queued one runs after it. This is consistent with every other order.
2. **Every order ends it, queued or not:** Shift would add the order and start it at once. This is a small change at the gate. It would make Shift behave differently for these two units than for any other.
