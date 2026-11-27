# Q0157 — Does low Power slow Scan's cooldown, the Missile Turret's fire and rearming?

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GR (the Radar Station and Scan, the Missile Turret) and GH2 (landing and rearming at the Airfield) build the placeholder in Options.
- **Docs**: `factions/gdo/overview.md > GDOResources > Power`; `factions/gdo/structures/{radar_station,missile_turret,airfield,defense_system}.md > Placeholder stats`; `factions/gdo/units/hornet.md > Placeholder stats`

## Question
At negative Power, do these slow down: the Radar Station's Scan cooldown, the Missile Turret's rate of fire, and the Airfield's rearming?

## Context
- "If total Power is negative, all power-consuming buildings and Extraction Plates operate slower in proportion to total available power versus total required power (affecting construction speed, unit production speed, and mining rate)."
  - "Speed ratio: generated power ÷ consumed power, capped at 100% (e.g. 30 generated, 51 consumed → ≈59%, the opening baseline's all-in case). Applies to every building with negative Power and to Extraction Plates."
  - "Slowed: Extraction Facility (plate construction), Barracks and other consuming producers (production), plates (mining)."
- That list already covers the Factory's and the Airfield's production, and the Defense System's turrets (it "Works like the Extraction Facility").
- It was written before any GDO building had a cooldown, a weapon or rearming. These three are consumers: the Radar Station (Power −20), the Missile Turret (−10) and the Airfield (−30).
- **At ≈59%:** Scan's 60 s cooldown would become about 102 s; a turret's missile every 24 frames, one every 41; a Hornet's 10 s full rearm, about 17 s.

## Options
- **(a) None of them** (the placeholder): only construction, production and mining slow, as listed. Turrets fire and aircraft rearm at full speed whatever the Power.
- **(b) All three,** as "all power-consuming buildings ... operate slower" reads: low Power then weakens defences, and each Missile Turret built past the supply slows every turret.
- **(c) Some of them,** e.g. the cooldown and rearming but not the turret's fire.
