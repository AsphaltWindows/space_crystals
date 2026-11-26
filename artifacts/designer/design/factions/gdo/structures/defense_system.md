# Defense System
GDO main defense-related structure. Constructs defensive structure(s).

## Faction - GlobalDefenseOrganization
## Constructed by - DeploymentCenter
## TechPrerequisite - player owns at least one Radar Station and at least one Arsenal

## Constructs:
- Missile Turret: 100 Space Crystals, 240 frames (15 seconds) (placeholder)

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 2x2; SymmetryType AAAA; MaxHP 350; PointArmor 1, FullArmor 4; SightRange 4 grid units; Groupable true; Power -20; BuildRadiusExtension 2 grid units.
- **Works like the Extraction Facility** (user, 2026-11-27: it only builds Missile Turrets): one slot, Q, no build menu (factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]):
  - **Idle**: **Q: Build Missile Turret** (100 Space Crystals, starts construction).
  - **Constructing**: **Q: Place Missile Turret**, greyed out, with a **running progress bar** on the button; **X: Cancel Construction** (full refund).
  - **Ready**: **Q: Place Missile Turret** enters AwaitingPlacement (ghost within the GDO build area, as for the Deployment Center's buildings); **X: Cancel Ready Turret** (75% refund, rounded down). Escape/right-click from placement returns to DefaultState, the turret stays ready.
