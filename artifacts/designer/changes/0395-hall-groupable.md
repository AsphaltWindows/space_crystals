# 0395 — Recruitment Halls are groupable

- **Date**: 2026-11-27T12:45:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/structures/recruitment_hall.md` (Groupable)

## What changed and why
User: "Recruitment hall should be groupable". Selected Halls form one SelectionGroup. Their panel commands, including Set Rally Point, apply to all of them. The other Cults production buildings stay ungroupable (Q0105).

## Implementation impact
- Groupable false → true for the RecruitmentHall.

## Verification
- Box-select 2 Halls and set a rally point: both send new Recruits there.
