# 0464 — Mirage: illusions copy the parent's buffs; Shift+F queues

- **Date**: 2026-11-27T15:55:00-05:00
- **Answers**: Q0188
- **Docs**: `factions/cults/units/disciple.md > Spells > Mirage` (Health; new "Shift-queueing")

## What changed and why
1. **Option (b).** Every timed buff the parent has at the cast is copied to each illusion for the time it has left. Examples: Inspire and Shrine regeneration. This keeps illusions convincing: an inspired Disciple's illusions move as fast as it does.
2. **Shift+F queues** (user). Mirage joins the order queue as a step. For example, Move then Shift+F casts on arrival. When its turn comes it casts if there is enough energy, and otherwise it is dropped with refusal feedback. The user's note: behind Shroud this doesn't help, since the channel only ends by running out of energy or by an unqueued order. From Shroud, plain F (cast at once, ending the channel) is the way to cast it.

## Implementation impact
- **Changed:** illusions now receive copies of the parent's active timed buffs, each with its remaining duration.
- **Changed:** Shift+F enqueues a Mirage step instead of casting at once. Plain F is unchanged.

## Verification
1. Inspire a Disciple and cast Mirage 4 s later. All five move at the inspired speed, and the illusions' buff ends at the same moment as the parent's.
2. Order a Move, then Shift+F. Mirage casts when the Disciple arrives.
3. With too little energy at that point, nothing is cast and the player gets feedback.
