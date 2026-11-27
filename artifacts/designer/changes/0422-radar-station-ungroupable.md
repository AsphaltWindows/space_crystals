# 0422 — Radar Stations are Ungroupable: each Scan press scans from one station

- **Date**: 2026-11-26T18:10:00-05:00
- **Answers**: Q0156
- **Docs**: `factions/gdo/structures/radar_station.md > Placeholder stats`

## What changed and why
The user chose (b): the Radar Station is **Groupable false**, as the Shrine and caster units, so one press of Scan never spends several stations' 60 s cooldowns on one spot. With several selected, each forms its own SelectionGroup: Q scans from the ActiveGroup's station, Tab moves on, Q scans again.

## Implementation impact
Replaces the placeholder (a). Other Ungroupable rules apply (selection.md > Constraints; GroupCycling).

## Verification
Select two Radar Stations: two groups. Q, click a point: one station scans and cools down; Tab, Q, click another point: the second scans there.
