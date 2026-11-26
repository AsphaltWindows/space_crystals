# 0334 — Shroud rules: centre under the cloud, touching joins, no directional armor, underground affected, regen continues, seen by all, per-cloud release timer, joining releases; the Disciple is Ungroupable

- **Date**: 2026-11-26T19:50:00-05:00
- **Answers**: Q0112 (point 2; point 1 in 0328), Q0113
- **Docs**: `factions/cults/units/disciple.md` (Groupable; Spells > Shroud; Placeholder stats)

## What changed and why
User, point by point:
- **Q0112.2:** the Disciple is **Ungroupable**. One R press starts one Disciple's channel; Tab moves to the next.
- **Q0113.1:** an object is **under** a cloud when its **centre** is inside. Shares are still by overlap area.
- **Q0113.2:** clouds **touching or overlapping** are connected, and a connected cloud is just one cloud.
- **Q0113.3:** releases ignore **directional armor**. Other modifiers apply.
- **Q0113.4:** **underground objects are affected**. They are protected, and take their share of releases.
- **Q0113.5:** the Disciple's **own regeneration continues** while channelling: a net 4.5/s, about 44 s from full.
- **Q0113.6:** the cloud is seen by **every player**, drawn over their Visible tiles, so its edge can be visible when the Disciple isn't.
- **Q0113.7:** **each connected cloud has its own timer**. It releases 8 frames after the first hit it absorbs, then waits for the next absorbed hit.
- **New, when clouds join** (e.g. one forms between two others): each joining cloud releases its buffer **immediately** over its own area. The joined cloud then starts a **fresh joint buffer**. Splits already release immediately.

## Implementation impact
- This replaces the global 8-frame clock assumption and the directional armor on releases.
- Disciple selection becomes ungroupable.

## Verification
- Two Disciples a short gap apart: a third channelling in between makes both clouds release at once, then they buffer together.
- An enemy seeing only the cloud's edge sees that edge.
