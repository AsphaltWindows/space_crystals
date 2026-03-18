# Feature Update: gdo_objects (2026-03-06)

## Modified Feature File
`features/gdo_objects.md` (NEW)

## Relevant Design Files
- `design/gdo_objects.md`
- `design/factions.md`

## Summary
Initial feature specification created from formal design content. Defines GDOBuildArea mechanics, all 9 concrete GDO objects: Peacekeeper (LightInfantry combat unit), Power Plant (+20 power), Barracks (produces Peacekeepers, rally point), Deployment Center (constructs buildings, multi-step place flow), Extraction Facility (constructs Extraction Plates), Extraction Plate (mines SC, residual rate), Supply Tower (scheduled deliveries, chopper attach), Supply Chopper (unarmed HoverCraft, supply transport). Full production chain: DC->PowerPlant/Barracks/SupplyTower, Barracks->Peacekeeper, ExFacility->ExPlate, SupplyTower->SupplyChopper.
