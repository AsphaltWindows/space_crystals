# 0007 — Patch and Supply station values are set per object in the map editor; defaults

- **Date**: 2026-11-23T13:23:28-05:00
- **Answers**: Q0003
- **Docs**: `core/map.md > SpaceCrystalsPatch` (StartingAmount), `core/map.md > SupplyDeliveryStation` (StartingSupplies, DeliverySize, DeliveryInterval)

## What changed and why
The user wants map makers to control resource values per object:
- **Space Crystals patch:** `StartingAmount` is set per patch in the map editor. The default for newly placed patches is **1500** (placeholder).
- **Supply Delivery Station:** fully customizable per station: **StartingSupplies** (default 0), **DeliverySize** (default 4) and **DeliveryInterval** (default 60 s = 960 frames). All defaults are placeholders.
- A station that starts at 0 begins counting down at game start (first delivery one interval in). A stocked station starts counting down once it's emptied, per the existing rule that the countdown runs only while CurrentSupplies = 0.
- **Reference map (point 4):** test maps stay the Director's concern. For the test map, put the start locations in the **corners** (user). Map design proper comes later.

## Implementation impact
- Matches the Director's placeholder constants, except that the values are per-object map data with editor defaults, not global constants. Map files store StartingAmount per patch and the three values per station.
- The editor needs fields to edit these on a selected patch or station.

## Verification
- In the editor, place a patch: it shows 1500. Change it to 800, save, load in game: the InfoPanel shows 800.
- Place a station with StartingSupplies 0, size 4, interval 60 s: the first delivery of 4 arrives at 1:00.
- Set StartingSupplies 6: the station starts with 6 and only begins its 60 s countdown once emptied.
- Two stations on one map can have different sizes and intervals.
