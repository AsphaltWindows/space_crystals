# Q0188 — Mirage: do illusions copy the parent's buffs, and does Shift+F queue?

- **Date**: 2026-11-27T17:52:38-05:00
- **Blocking**: no. M6-CM built Mirage on two assumptions, which a sweep changes if the answer differs:
  - (1) illusions get none of the parent's buffs;
  - (2) Shift does nothing for Mirage: Shift+F casts at once, as F does.
- **Docs**:
  - `factions/cults/units/disciple.md > Mirage` ("Health: ... has its MaxHP and speed (augmentations included)");
  - `> Shroud > What ends the channel`;
  - `systems/controls/commands.md > Unit Command` (Shift-queued orders).

## Question
1. **Does an illusion copy the buffs its parent has when Mirage is cast?** Examples are a Prayer-mobile's Inspire (×1.25 movement speed for infantry) and a Shrine's regeneration. If so, does it keep each one for the rest of that buff's time?
2. **Is Mirage Shift-queueable?** For example, a Disciple channelling Shroud gets Shift+F. Does the Mirage wait until the channel ends and then cast, as a Shift-queued order does? Or does it cast at once and end the channel, as F does?

## Context
- **Point 1:** the doc gives an illusion the parent's current HP, MaxHP and speed, "augmentations included". It says nothing about timed buffs.
  - As built, illusions get none. A Disciple inspired when it casts therefore makes illusions that walk 25% slower than it does.
  - That can give the real one away when the group moves, just as Shroud's mirroring was designed to (the mirroring shows the real one only while it channels).
- **Point 2:** "What ends the channel" lists "casting Mirage" among the cancel commands, beside "any other order **given without Shift**; a Shift-queued order waits and runs when the channel ends".
  - That reads as if a Shift-queued Mirage would wait. But Mirage is an instant spell, not an order, so it doesn't sit in the order queue.
  - As built, Shift is ignored, and Shift+F casts at once.

## Options
**Point 1:**
- **(a)** No buffs (as built). Illusions are plain copies of the parent's stats.
- **(b)** Copy each buff the parent has at casting, for the time it has left. Illusions then match its speed while Inspire lasts.
- **(c)** Copy only movement-speed buffs, so the copies stay convincing while moving.

**Point 2:**
- **(a)** Shift ignored, cast at once (as built).
- **(b)** Shift+F joins the order queue as a cast step: it waits for the channel and any queued orders, then casts if there is energy, else it is dropped with feedback.
