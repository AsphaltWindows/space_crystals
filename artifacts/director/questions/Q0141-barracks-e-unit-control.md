# Q0141 — Barracks E: does queuing a Field Engineer need free Unit Control?

- **Date**: 2026-11-26T06:20:21-05:00
- **Blocking**: no. M5-N9 built Q's and W's rule for E: queuing needs crystals and a queue slot, and Unit Control is checked when production starts.
- **Docs**: `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]` (Q, W, E); `factions/gdo/overview.md > Unit Control`

## Question
Does E check Unit Control at queue time, as its line says? Or is that line stale, so that Unit Control is checked when production starts, as for Q and W?

## Context
- **E says:** "Only available if the player owns at least one Career Campus, BuildQueue has fewer than 20 entries and the player has sufficient Space Crystals **and Unit Control**."
- **Q and W say:** "(Unit Control is checked when production starts: factions/gdo/overview.md > Unit Control)".
- **The overview says** (2026-11-24, replacing "reserved when queued"): "queuing a unit needs only its Space Crystals (paid when queued) and a free queue slot; it does **not** check or reserve Unit Control."
- E's line was added by 0328, after that rule.

## Options
- **(a) E's line is stale.** Unit Control is checked when production starts, as for Q and W. This is what is built, and the line needs updating.
- **(b) E is an exception** and checks Unit Control at queue time.
