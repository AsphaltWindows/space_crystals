# Null Chamber

A shielded dome that works like a Faraday cage against imprinting, so crystal-based human enhancement can be done without the risk of an operative's identity being imprinted by consumers nearby (see core/lore.md > Space Crystals > Properties and core/lore.md > Syndicate > Paranoia about imprinting).

Distinct from the Syndicate's physical augmentation: Syndicate infantry are Heavy Infantry because of mechanical exoskeleton-style assistance, which involves no crystals. The Null Chamber is about **consumption**: its effects are subtle and mental (heightened awareness), unlike the Cults' morphological Altar augmentation and unlike Colonists psionics.

## Null Chamber Exit Buff
**Throughput limits** (user, 2026-11-25, to be designed): the re-buff should probably be **limited**: only so many units buffed at a time (per Null Chamber), and each buff takes time to apply. How that combines with "every unit that exits receives the buff": placeholder below (to confirm).

While the player owns at least one Null Chamber anywhere in the Tunnel Network, every **manned** Syndicate unit that exits the Tunnel Network through any Tunnel receives a temporary buff that wears off over time.
- **Manned units**: all Syndicate units except satellites (infantry, vehicles, and crewed aircraft).
- **Effects**:
  - **Detection** (stealth/detection mechanic not yet designed — see loose ends)
  - **Increased SightRange**
  - **Percentage damage-taken reduction**, applied **before armor**: damage taken = Attack Damage × (1 − reduction) − armor
- **Refresh**: exiting the Tunnel Network again instantly resets the buff to full duration (no cooldown, for now).
- **Lore**: occupants consume crystals inside the Null Chamber, shielded from imprinting; the effect fades once they are on the surface.

Placeholders below for: duration; SightRange bonus; damage-reduction percentage; whether the buff ends abruptly or decays gradually; whether the buff (or which of its effects) must first be unlocked by research at the Null Chamber, and any further Null Chamber upgrades to the buff.

## Entity Type - Structure Type (Underground)
## Tier Requirement - 2
## TechPrerequisite - player owns at least one Hideout (anywhere in the Tunnel Network)

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Size 2x2; MaxHP 400; PointArmor 6, FullArmor 24 (underground expansion, n = 6); Cost 200 Space Crystals; Build Time 480 frames (30 seconds).
- Exit buff: duration **20 s**; **+2 grid units SightRange**; **20%** damage-taken reduction; ends abruptly (to confirm); no research needed (to confirm). Throughput (to confirm): each Null Chamber buffs **4 units at a time**, each buff taking **2 s** inside before the unit can leave buffed; a unit that leaves before its turn leaves unbuffed.
