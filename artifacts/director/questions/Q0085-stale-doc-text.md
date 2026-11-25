# Q0085 — Stale lines the answers left in the docs

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: the newer answer wins in each case below; the slices build it and cite the answer.
- **Docs**: listed per item

## Question
May these lines be corrected to match the later answers? Each contradicts a newer rule in another section.

## Context
1. `factions/syndicate/units/agent.md > Building > Tunnel Construction Flow`, step 5 ("If construction is cancelled by the player"): "100% of the Supplies cost is refunded", and `> Cancel Construction`: "Full refund of Supplies cost". 0233 and `tunnels.md > Upgrade Costs` say construction refunds the **current top price** ("Tunnel construction refunds the same way").
2. `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState`, the older commands: "**X: Cancel Upgrade** — ... Full refund of Supplies cost". 0232 says the current top price.
3. `factions/syndicate/units/agent.md > DefaultState commands`, the older letters that 0257 "replace[s] ... where they differ": "**B: Drop Off Resources**" (now S), and "**Build Laser Turret** (hotkey TBD)" (now D).
4. `factions/syndicate/structures/laser_turret.md > Upright vs Pop-up`: "Rise and retract timing: TBD" and "Values TBD", while the placeholder stats below give 8/32/8 frames and 6/6.
5. `core/entities.md > Structure Type > Building armor rule`: the example "e.g. a 2x2 Hideout: 1 / 4". By "High armor underground" the Hideout is 6/24.
6. The expansions' stat blocks (`hideout.md`, `optics_lab.md`, `chemical_plant.md`): "PointArmor - 1", "FullArmor - 4", which "High armor underground" says it "replaces".
7. `systems/combat.md > AoEDamage`, first sentence: "Ground AoE hits Ground and surfaced Underground units", followed by "**Underground objects are hit by area damage too**".

## Options
- **A (proposed).** Correct each line to the newer rule, or replace it with a pointer to it. No design change: the slices already build the newer rules.
- **B.** Leave them, with a note at the top of each file that later answers override older lines. Agents reading a section alone will keep tripping on them.
