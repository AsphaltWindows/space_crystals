# Q0133 — Can a selected own Hall's claims reveal an enemy Hall the player has never seen?

- **Date**: 2026-11-26T01:42:58-05:00
- **Blocking**: no — M5-J2 shows a selected own Hall's true claims, as its InfoPanel's Effectiveness does.
- **Docs**: `factions/cults/structures/recruitment_hall.md > Recruitment Area display`, `> TileClaiming`, `> Effectiveness`

## Question
A selected own Hall shows its claimed tiles. Those are its real claims, and Halls of different players compete for tiles. So when an enemy Hall the player has never seen took some tiles first, the selected Hall's claims have a gap of that Hall's shape, and its Effectiveness is lower by as much. Is that acceptable, or should the selected Hall's display read the player's knowledge, as the placement preview does?

## Context
- **The preview avoids the leak on purpose:** "Enemy Halls you have never seen are not counted (no information leak), so the preview may then be optimistic."
- **The selected Hall** "shows its claimed tiles", with no such rule. Its InfoPanel already shows "Effectiveness: 72% (72/100 tiles)", the real value, which is what the Hall's capacity and production speed follow (`> Effectiveness`).
- So even before M5-J2, a player could tell from the number that something had taken tiles. The tile display makes it clearer, showing where.

## Options
- **(a) As built:** the selected own Hall shows its true claims and its true Effectiveness. They match what the Hall produces, and the player learns roughly where an unseen enemy Hall is.
- **(b) Both follow knowledge:** the selected Hall's tiles and its InfoPanel's Effectiveness are computed as if only seen or remembered Halls existed, like the preview, while its capacity and production follow the real claims. The production rate still gives the information away, more slowly.
- **(c) Only the tiles follow knowledge:** the tiles are drawn as in (b), and the InfoPanel keeps the true Effectiveness. The number then disagrees with the drawn tiles whenever an unseen Hall competes.
