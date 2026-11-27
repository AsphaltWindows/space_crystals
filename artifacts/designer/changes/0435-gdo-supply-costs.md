# 0435 — GDO Supply costs (first set); the Chopper repairs on its Tower at the repairers' rate

- **Date**: 2026-11-26T20:00:00-05:00
- **Answers**: Q0169
- **Docs**: `factions/gdo/overview.md > GDOResources > Supplies`; Cost lines of `factions/gdo/units/{field_engineer,drone_operator,hornet,dragonfly,artillery_rig,prospector}.md`; `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`; `factions/gdo/economy.md > SupplyTower > Landing and Attachment`

## What changed and why
**1 (b)**, the user's costs, in Supplies on top of Space Crystals: **Field Engineer 1, Drone Operator 2, Hornet 2, Dragonfly 4, Artillery Rig 5, Prospector 8**. Paid with the Space Crystals when queued (both required), refunded with them on cancel. No GDO building or research costs Supplies yet.

**2 (a)** (designer placeholder): a Chopper landed on its Tower is repaired free at the repairers' rate, 0 to full in 80 frames (≈30 HP/s).

## Implementation impact
New: a second cost on six GDO units; production buttons need both resources (greyed otherwise); refunds return both. Supplies now matter, so the Supply Tower/Chopper loop is needed for these units. Starting Supplies are still 0 (see the open follow-up with the user).

## Verification
- With 0 Supplies, Field Engineer, Drone Operator, Hornet, Dragonfly, Artillery Rig and Prospector are greyed; Peacekeepers aren't.
- Queue an Artillery Rig: 250 SC and 5 Supplies leave the bank; cancel it: both come back.
- A damaged Chopper landing on its Tower: back to full within ~5 s, no cost.
