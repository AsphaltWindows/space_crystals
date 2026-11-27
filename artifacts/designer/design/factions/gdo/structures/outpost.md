# Outpost
GDO expansion structure created when a Prospector deploys. Its only function is to provide a **build radius**: it extends the GDO build area around it so the Deployment Center can place buildings there (including an Extraction Facility for a remote crystal field).

## Entity Type - Structure Type
## Faction - GlobalDefenseOrganization
## Created by - Prospector (Deploy into Outpost); not built by the Deployment Center
## Placement - any Buildable terrain (where the Prospector deploys)
## Power - −10 (placeholder)
## Permanent - cannot pack back up into a Prospector

Provides **only** the build radius — it does not construct anything, produce units, or act as a drop-off.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 3x3 (the Prospector deploys in place); SymmetryType AAAA; MaxHP 600; PointArmor 1, FullArmor 9; SightRange 6 grid units; **Groupable true** (designer placeholder, 2026-11-27; Q0182: it has no commands, so it makes no difference yet); **BuildRadiusExtension 6 grid units**; Power −10.
