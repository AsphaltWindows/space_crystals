# 0337 — Choosing Recruits by kind, the free Release order, per-kind release at tech buildings; Eject All stays; HoverCraft dropped from the sidestep list

- **Date**: 2026-11-26T20:55:00-05:00
- **Answers**: Q0128 (point 1; point 2 is with the user)
- **Docs**:
  - `factions/cults/structures/README.md > Choosing Recruits, Release and Eject All` (new);
  - `factions/cults/structures/{armory,cathedral,chop_shop,airstrip,temple,shrine,gunworks,machine_shop}.md` (panels);
  - `factions/cults/augmentation.md` (AugmentMenu: Release; Fully augmented enabled);
  - `systems/units.md > Ground Collision`.

## What changed and why
User: a way to get units out unchanged. "First select the unit, then among the train/augment options a no-op that is free with a very fast training time, so you can press it n times to get n recruits out. For tech buildings, just buttons with recruit types (by augment); pressing one makes a recruit of that type exit; the icon shows the augment. Eject All still exists."

- **Kinds:** Plain, Constitution, Metabolism and Both. Each button shows an augment icon and the count inside.
- **Pool buildings** (Armory, Cathedral, Chop Shop, Airstrip):
  - DefaultState: **Q/W/E/R** pick a kind, **A** Eject All, **C** Rally, **X** Cancel.
  - KindMenu: the production options with Recruits of that kind, plus **A: Release**; Z goes back.
  - You pick a kind **at the Chop Shop and Airstrip too** (user), and a Jackal takes 2 Recruits of the chosen kind.
- **Release:**
  - free, **4 frames** (placeholder), queued like any order;
  - the unit walks out unchanged to the RallyPoint;
  - first in, first used within a kind.
- **Altar:** Release is added in every state menu. The "Fully augmented" tile is now enabled, with Release as its only option.
- **Tech buildings** (Temple, Gunworks and Machine Shop once enterable, Shrine):
  - **A/S/D/F** each release one Recruit of a kind at once, with no queue;
  - **V** Eject All (letters are designer placeholders).
- **Eject All** stays everywhere.
- **Q0128.1:** HoverCraft in 0323's list was a slip, since it is Domain Air. It is removed.

## Implementation impact
- The Armory's direct Q/W become a two-step pick (kind, then order). This changes M5's Armory panel.
- Release is a new queue order type.
- Augment icons are needed on unit buttons.

## Verification
- An Armory with 2 Plain and 1 Constitution Recruit: W shows 1.
- Pressing W, A, A queues one Release; the second press is greyed.
- The Constitution Recruit walks out after ¼ s.
- At a Temple, S sends out one Constitution Recruit immediately.
