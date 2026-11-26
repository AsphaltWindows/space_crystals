# 0350 — Recruitment Area display: like the Tunnel Area; visible enemy Halls count in the preview

- **Date**: 2026-11-26T22:55:00-05:00
- **Answers**: Q0107
- **Docs**: `factions/cults/economy.md > RecruitmentHall > Recruitment Area display` (new)

## What changed and why
User chose **(b)**: "If you can see the recruitment hall then you should know which tiles you won't be able to claim, and that should be shown in the preview."
- The placement preview shows the area, the claimable, own-claimed and non-Recruitable tiles, and Effectiveness %.
- The tiles claimed by **enemy Halls the player can see** show as unclaimable.
- Unseen enemy Halls aren't counted.
- A selected own Hall shows its claims.

Whether a **remembered** enemy Hall (seen earlier, now in Explored fog) also counts is being asked. Until then, count only Halls currently Visible.

## Implementation impact
The preview needs the claims of visible enemy Halls.

## Verification
- With an enemy Hall in sight, the preview next to it shows its claimed tiles as unavailable, and the Effectiveness drops accordingly.
