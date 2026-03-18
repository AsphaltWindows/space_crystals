# Feature Update: vision_system (2026-03-06)

## Modified Feature File
`features/vision_system.md` (NEW)

## Relevant Design Files
- `design/entities.md`

## Summary
Initial feature specification created from formal design content. Defines fog of war with three visibility states (Unexplored, Explored, Visible), vision provided by owned objects via SightRange, and ElevationModifier (+1/-1 binary modifier to sight/attack range, air units exempt, underground units use terrain elevation).
