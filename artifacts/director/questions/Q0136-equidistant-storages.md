# Q0136 — A patch equally near two Storages: which one does a Recruit use?

- **Date**: 2026-11-26T02:51:19-05:00
- **Blocking**: no — as built, the tie goes to the Storage with the lower object id, a technical placeholder.
- **Docs**: `factions/cults/units/recruit.md > Gathering` ("Which Storage"); design change 0345; `factions/cults/structures/storage.md`; design change 0320 (the layout is the player's job)

## Question
0345: "at **every return** the Recruit takes its load to the **nearest own completed Storage, edge to edge from its patch**". When two Storages are equally near the patch, which one does the Recruit use?

## Context
- **When ties happen:** on a field of 8 adjacent patches, with one Storage under each pair of patches (one per face), the patch at the end of a face is a tile from the next face's Storage too, exactly as near as its own.
- **As built,** the tie goes to the lower object id, so some faces' Recruits carry to the neighbouring face's Storage and crowd its lanes.
- **The cost:** in M5-L's throwaway measurement (not a kept test), 16 Recruits on such a field gathered 14.4–15.9 SC/s, against 20.5 SC/s with the faces a free column apart.
- **The oracle's reference field** therefore leaves a free column between patch pairs. A map's real fields may not.
- **0320 makes the layout the player's job,** but a player can't see or control this tie.

## Options
- **(a) As built:** the lower object id. It is deterministic, but arbitrary to the player.
- **(b) Nearest to the Recruit:** on a tie, the Storage nearest the Recruit when its return starts, then the lower id.
- **(c) Keep the last one:** on a tie, the Storage the Recruit delivered to last, so a face's Recruits stay with their face once they have used it.
- **(d) Something else,** such as a Storage serving only the patches it was placed for.
