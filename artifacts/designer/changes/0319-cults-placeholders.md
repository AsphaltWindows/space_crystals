# 0319 — Cults placeholder stats for every designed building and unit

- **Date**: 2026-11-26T16:00:00-05:00
- **Answers**: Q0092, Q0093, Q0094, Q0108 (point 1; point 2 waits for Q0095)
- **Docs**:
  - `factions/cults/units/{disciple,jackal,sparrow,recruit}.md` (Constructs);
  - `factions/cults/structures/{gunworks,machine_shop,scrap_hangar,garrison,chop_shop,airstrip,temple,cathedral,shrine}.md`;
  - `factions/cults/augmentation.md` (Altar, Metabolism increment);
  - `factions/cults/economy.md > Storage` (armor);
  - `systems/controls/commands.md > Enter`.

## What changed and why
User: "You should fill in provisional stats for buildings and units so that they could be implemented."

Every designed Cults building and unit now has **placeholder** numbers, marked as such. They follow the building armor rule (n = 1, with the Garrison at n = 2) and are sized against the Armory and Storage.

**Buildings** (cost / build time with 1 Recruit):

| Building | Cost / time | Size | HP | Armor | Other |
|---|---|---|---|---|---|
| Gunworks | 150 / 480 | 2x2 | 300 | 1/4 | — |
| Machine Shop | 150 / 480 | 2x2 | 300 | 1/4 | — |
| Scrap Hangar | 200 / 640 | 3x3 | 450 | 1/9 | — |
| Garrison | 150 / 400 | 2x2 | 400 | 2/8 | sight 6; units leave any side; A Eject All |
| Chop Shop | 200 / 640 | 3x3 | 450 | 1/9 | sides B in / C out; capacity 10; queue 10; Q Build Jackal |
| Airstrip | 250 / 640 | 2x4 runway | 400 | 1/8 | capacity 10; queue 10; Q Build Sparrow |
| Temple | 200 / 640 | 3x3 | 450 | 1/9 | staff 6; queue 5; L1 research 100 SC / 960 f; L2 150 SC / 1440 f |
| Cathedral | 250 / 640 | 2x3 | 400 | 1/6 | capacity 10; Disciple 75 SC / 320 f |
| Shrine | 200 / 480 | 2x2 | 300 | 1/4 | 8 Recruits; MaxEnergy 200 at 0.25/s per Recruit |
| Altar | 150 / 480 | 2x3 | 300 | 1/6 | capacity 10; queue 20 |

- **Gunworks and Machine Shop:** no research is designed yet, so they have **no commands and can't be entered** (Q0093 option a).
- **Shrine abilities:**
  - Health regeneration: 75 energy, radius 4 GU, +2 HP/s for 15 s;
  - Energy regeneration: 50 energy, radius 4 GU, +1 energy/s for 20 s.
- **Altar orders:** 25 SC / 160 f each, and Both costs the sum.
- **Metabolism increment:** +0.5 su/frame MaxSpeed; for unarmed units, −1 frame of FiringDuration (minimum 2).
- **Storage armor 1/6 → 1/4** (Q0094 a): the 1/6 was a leftover from its old 3x2 size.

**Units:**
- **Disciple:**
  - LightInfantry 24x24, HP 40, sight 6, speed 4;
  - **no weapon** (placeholder, to confirm with the user), so it counts as armed for Augmentation;
  - energy: max 200, starts at 50, regenerates 0.5/s;
  - Shroud: radius 3 GU, drain 5/s, needs at least 10 to start, hotkey Q;
  - cost: 75 SC, 1 Recruit, 320 f.
- **Jackal:**
  - 56x40, HP 120, armor 1/1, sight 6;
  - turn radius 1 GU, speed 8 su/f;
  - turret 360° at 12°/f;
  - weapon: Damage 3, range 4, one shot every 4 f;
  - cost: 2 Recruits, 75 SC, 320 f.
- **Sparrow:**
  - 40x40, HP 50, sight 6;
  - glider: speed 10 su/f, idle 4, centripetal 1;
  - turret 45° at 8°/f;
  - weapon: Damage 3, range 3, one shot every 4 f;
  - cost: 1 Recruit, 75 SC, 320 f.
- **Enter (Q0108.1):** its TargetObject is now "the structure being entered".

**Q0092 scope:** every designed Cults building and unit can now be implemented on these placeholders:
- Gunworks and Machine Shop without research;
- the Scrap Hangar with nothing to unlock yet.

The order of slices is the Director's call.

## Implementation impact
New content to implement as listed. These values supersede the Director's own interim Gunworks numbers (the same as proposed).

## Verification
- A Garrison with 4 Militants fires at range 5.
- The Chop Shop needs 2 free Recruits inside to queue a Jackal.
