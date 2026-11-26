# 0328 — Placeholder and cleanup answers: Field Engineer and Career Campus numbers, Shroud on R, Sparrow separation, repair prices, stale lines

- **Date**: 2026-11-26T18:30:00-05:00
- **Answers**: Q0123, Q0125; Q0112 point 1; Q0120 point 1; Q0124 points 5–6 (the other points of Q0112, Q0120 and Q0124 are with the user)
- **Docs**:
  - `factions/gdo/units/field_engineer.md`; `factions/gdo/structures/career_campus.md`;
  - `factions/gdo/structures/barracks.md > Produces`, `> ObjectInterfaceState`; `factions/gdo/structures/deployment_center.md > Constructs`;
  - `factions/cults/units/disciple.md`; `factions/cults/units/sparrow.md`;
  - `factions/cults/units/recruit.md > Repair`, `> Unit Commands`, `> Right-Click Resolution`; `factions/syndicate/units/agent.md > Repair`;
  - `systems/controls/commands.md > Enter`;
  - `factions/cults/structures/{garrison,chop_shop,airstrip,shrine,README}.md`.

## What changed and why
These are under the user's standing permission to set placeholder numbers, plus doc cleanup.

- **Q0125: Field Engineer and Career Campus get placeholders**, so the GDO can repair in M5 (option b; whether to build it is your call).
  - **Field Engineer:**
    - LightInfantry 24x24, HP 50, sight 5, speed 4, 1 Unit Control;
    - Barracks **E**, 75 SC, 160 f;
    - panel: Q Move, **A** Construct (Q Bunker, W First Aid Tent, greyed), **D** Repair, X Stop;
    - right-click rules as for Recruits (Enter wins).
  - **Career Campus:** 2x2, HP 300, armor 1/4, sight 4, Power −20, 150 SC, 320 f, no panel.
- **Q0112.1: Shroud is on R.** 0319's "Q" clashed with Move.
- **Q0120.1: the Sparrow's SeparationRadius** is 32 su.
- **Q0124.5:** a Tunnel's repair rate uses its construction time, 480 f, at every Tier. Repair is free either way.
- **Q0124.6:** a vehicle's repair price counts **crystals only**. A Jackal costs 37.5 SC from 0 to full.
- **Q0123:**
  - The Gunworks and Machine Shop can't be entered until their research exists, so right-click is a Move. This is reflected in the Recruit's lists and in Enter.
  - "Own Garrison → Enter Garrison" is added.
  - Stale TBDs in the Chop Shop, Airstrip, Shrine table and README are replaced by the placeholder values.
  - The Garrison typo is fixed.

## Implementation impact
- New GDO content if you choose to build it.
- Key change for Shroud.
- The rest is doc alignment with 0319.

## Verification
- A Field Engineer from a Barracks (with a Career Campus) repairs a damaged Barracks, spending crystals.
- The Disciple's R channels Shroud, and Q is Move.
