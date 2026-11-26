# Q0113 — Shroud: what is under the cloud, and the release

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: the proposal below.
- **Docs**: `factions/cults/units/disciple.md > Spells > Shroud`, `> Placeholder stats`; `systems/combat.md > AoEDamage`, `> DirectionalArmor`; `systems/units.md > Underground Collision`

## Question
Shroud is built from the doc's rules. Some points are still open:
1. **Under the cloud.** Is an object under the cloud when its centre is inside a cloud's circle, or when any part of it overlaps one? This decides whose damage is absorbed, and which attackers are "under the cloud" and so do full damage.
2. **Connected.** Do two clouds connect when their circles overlap, or also when they only touch?
3. **The release's direction.** For directional armour, an area hit's zone is "the angle from the unit's center to the AoE center", but a connected cloud has several centres. Which one counts for a unit?
4. **Underground objects** (DrillUnits, Syndicate expansions, a lowered pop-up Laser Turret). "It affects every ground object". Are underground objects under the cloud protected? Do they take a share of a release, as ground area damage hits them (Q0071)?
5. **Regeneration while channelling.** "drains **5 energy per second** (40 s from full)": 40 s means the Disciple's own 0.5/s regeneration stops while it channels. Is that right?
6. **Who sees the cloud:** every player who sees the Disciple, or only its owner?
7. **When releases happen:** every 8 frames on the game's clock, for all clouds together, or 8 frames after a cloud first absorbs damage?

## Context
- "standing **under the cloud**" suggests the centre. The shares already use the overlap area.
- **The numbers:**
  - A cloud's area is about 200 times a Militant's.
  - A lone Militant under a lone cloud, shot for 4, takes 0.5 at the release: the area minimum ("0.5 × overlap fraction").
  - A 3x3 building wholly under one cloud takes about a third of what was absorbed.
- **For 7:** either way, damage comes back at most 0.5 s late. A fixed clock is the simplest to read.

## Options
1. **(a)** the centre; **(b)** any overlap.
2. **(a)** touching joins; **(b)** overlapping only.
3. **(a)** from the nearest Disciple of the connected cloud; **(b)** no directional armour on releases.
4. **(a)** neither protected nor hit by releases, since they aren't ground objects; **(b)** protected, and they share releases as ground area damage does.
5. **(a)** no regeneration while channelling: 40 s from full; **(b)** it regenerates: about 44 s from full.
6. **(a)** every player who sees the Disciple; **(b)** its owner only.
7. **(a)** the game's clock; **(b)** from the first absorbed hit.

Proposed: (a) on all seven.
