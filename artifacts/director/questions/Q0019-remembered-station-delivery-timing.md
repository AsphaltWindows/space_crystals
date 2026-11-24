# Q0019: Delivery timing on a remembered Supply Delivery Station

- **Date**: 2026-11-23T22:10:00-05:00
- **Blocking**: no. The assumption below is going ahead.
- **Docs**: `core/vision.md > Explored Selectability` (the GDO exception and its rationale), the Supply Delivery Station's InfoPanel

## Question
When a GDO player selects a supply station they can't currently see, it is shown as last seen in an Explored area. What should its InfoPanel show for "Next delivery"?

## Context
The fog now remembers structures and resources as last seen. A GDO player can select a remembered station, as the docs allow, and its InfoPanel currently freezes "Next delivery in N s" at the value it had when last seen. The docs justify the GDO exception with "the SupplyTower scheduled delivery mechanic requires planning around delivery timing even without active vision". A frozen countdown works against that.

**Assumption:** the countdown continues. Deliveries come on a fixed interval, so the next delivery time can be worked out exactly from the last sighting. The supply count stays hidden, because it depends on who has collected.

## Options
- **A (assumption):** the delivery countdown keeps running; the supply count stays unknown.
- **B:** show everything as last seen, with the countdown frozen and a "last seen" note.
- **C:** show the countdown and also estimate the supply count. This would leak information, so it is probably not wanted.
