# 0352 — Hall placement preview counts remembered enemy Halls

- **Date**: 2026-11-26T23:05:00-05:00
- **Answers**: — (settles the open point of 0350)
- **Docs**: `factions/cults/economy.md > RecruitmentHall > Recruitment Area display`

## What changed and why
User: "If the Recruitment Hall is now in fog, your estimate should assume it's there, although it could be wrong."
- Enemy Halls the player **remembers** (last-known state in Explored fog) count in the preview, like visible ones.
- This uses the remembered state only, so nothing leaks. The estimate can be wrong if the Hall has since been destroyed.

## Implementation impact
- The preview reads the player's memory of enemy Halls, not the true state.

## Verification
- Scout an enemy Hall, leave, and have it destroyed unseen: the preview still treats its tiles as taken until you see the spot again.
