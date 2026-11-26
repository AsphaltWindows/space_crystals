# 0387 — A Recruitment Hall can be placed at the map edge; off-map tiles count as non-Recruitable

- **Date**: 2026-11-27T11:50:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/structures/recruitment_hall.md > RecruitmentArea`, `> Effectiveness`

## What changed and why
User: "Recruitment Hall should be buildable on the edge, but you shouldn't get the recruitable tiles from the edge of the map, so it's suboptimal but should be possible."
- **This replaces Q0099 / 0341's "the whole area must lie on the map".** A Hall can be placed wherever its 4x4 footprint fits on the map, right up to the edge.
- The part of its 10x10 area beyond the map edge yields **no Recruitable tiles**. Effectiveness still divides by the full 100, so off-map tiles count as non-Recruitable.
- Example: a Hall against the edge has 3 of its 10 rows off the map, so at most 70%.
- Tunnel Areas keep their own on-map rule (0242).

## Implementation impact
- Remove the placement refusal for Hall areas leaving the map, from M5-J1 and from 0341's Verification.
- The placement preview and the selected-Hall display show only the on-map part. The preview's Effectiveness counts off-map tiles as 0.

## Verification
- Place a Hall flush against the map edge: it is accepted.
- The preview shows Effectiveness ≤ 70%, and the built Hall's capacity and production follow it.
