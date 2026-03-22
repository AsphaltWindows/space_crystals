# extraction_plate_power_penalty

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Extraction Plates now slow down when GDO power is negative, consistent with how buildings already slow down. Two files were updated:

**`artifacts/designer/design/factions.md`** — The Power resource description (GDOResources section) was expanded from only mentioning buildings to explicitly include Extraction Plates and list the affected operations: construction speed, unit production speed, and mining rate.

**`artifacts/designer/design/gdo_objects.md`** — A `PowerPenalty` line was added to the Extraction Plate entity, stating that MiningRate and ResidualMiningRate are reduced proportionally when total GDO power is negative, using the same ratio as the global building power penalty.

The penalty ratio is: available power / required power. For example, if a player generates 20 power but consumes 40, all power-consuming operations run at 50% speed — including Extraction Plate mining.

## QA Instructions

1. **Setup**: Place a Power Plant (+20 power), an Extraction Facility (-15 power), and at least one Extraction Plate (-3 power) on a Space Crystal Patch.
2. **Normal mining**: With positive total power, observe the Extraction Plate mining at its full rate (10 SC per 3 seconds). Note the income rate.
3. **Trigger power deficit**: Destroy or sell the Power Plant so total power goes negative. Alternatively, build enough power-consuming buildings to exceed available power.
4. **Verify slowdown**: Observe that the Extraction Plate's mining rate decreases proportionally. For example, if available power is half of required power, mining should take twice as long per cycle.
5. **Residual mining**: Deplete a Space Crystal Patch and verify the residual mining rate (1 SC per 3 seconds) is also slowed under power deficit.
6. **Recovery**: Restore positive power (e.g., build a new Power Plant) and verify mining returns to full speed.
