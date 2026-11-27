# 0448 — Disciple: Mirage spell; the Shroud cloud lingers 1.5 s

- **Date**: 2026-11-27T12:00:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/disciple.md > Spells > Mirage` (new)
  - `factions/cults/units/disciple.md > Spells > Shroud` (new "The cloud lingers" bullet; "When the cloud changes" and "What ends the channel" changed)
  - `factions/cults/units/disciple.md > Placeholder stats` (Mirage numbers, panel F)
  - `factions/cults/structures/garrison.md > Garrisoning` (illusions may enter)
  - `README.md`, `factions/cults/units/README.md` (index lines)

## What changed and why
The Disciple gets a second spell, **Mirage** (user's design). It is instant: it creates **4 illusions** of the Disciple, one each to the north, east, south and west, 1 GU away (placeholder). The Disciple has no weapon, so a fake Disciple can't give itself away by not shooting. That makes it a decoy that enemies have to guess at.

The illusions:
- start with the parent's current HP, MaxHP and speed;
- take **double damage from all sources**;
- look exactly like a Disciple to enemies, including in the InfoPanel;
- look different to their owner, and are named "Disciple (illusion)";
- are normal controllable units that use **no Unit Control**;
- have the Disciple's sight;
- can't cast spells.

**Shroud and Garrison:**
- When the parent starts channelling Shroud, its illusions play the channelling animation **without a cloud**. Each can be cancelled individually, and all stop when the parent's channel ends.
- Illusions can enter a Garrison and fire **for real** as Militants.

**How illusions end:** when they die, when the parent dies, or when the parent recasts Mirage. A recast replaces the old set.

**Casting Mirage cancels Shroud.**

**The Shroud cloud now lingers** (user) for **1.5 s** (placeholder) after its channel ends for any reason, death included. It stays where it was and keeps absorbing. Its buffer is released when it vanishes, no longer at the moment the channel ends.

## Implementation impact
- **Changed (Shroud):** the "released immediately when the channel ends" rule is replaced. Release now happens when the lingering cloud vanishes, or when a connected cloud splits. A lingering cloud is a normal cloud for connection, joining and timers. If the Disciple starts a new channel next to its lingering cloud, the two join under the existing rules.
- **New:** the Mirage spell. Its parameters, all placeholders: 50 energy, 4 illusions, 1 GU offset centre to centre.
  - Placement: the nearest free spot when a compass spot is blocked.
  - Hotkey: **F** in the Disciple's combat panel.
  - Disciple's panel: R Shroud, F Mirage.
- **New:** the illusion kind, a Disciple variant:
  - takes ×2 damage;
  - no Unit Control;
  - can't cast spells;
  - can't be augmented, but copies the parent's augmented stats when created;
  - owner-only visual difference and name suffix;
  - linked to its parent: it vanishes when the parent dies or recasts.
- The Shrine's health regeneration affects illusions.
- **Designer's readings, flagged for the user:**
  - the linger keeps the cloud in place;
  - an illusion that is busy when the parent starts channelling stops and plays the animation.

## Verification
1. Select a Disciple with at least 50 energy and press F. Four illusions appear north, east, south and west of it, and energy drops by 50. For the owner they look different and are named "Disciple (illusion)". An enemy sees five identical Disciples, and their InfoPanels show "Disciple".
2. Unit Control is unchanged. Move the illusions around: they reveal the map like a Disciple.
3. Shoot an illusion: it loses twice the HP a real Disciple would.
4. Press R on the parent. The illusions play the channelling animation, but only the parent has a cloud. Stop one illusion: it alone stops. Stop the parent: all of them stop.
5. Press F while the parent channels. The channel ends, the old illusions vanish and four new ones appear.
6. Kill the parent: all its illusions vanish.
7. Put an illusion in a Garrison: it fires the Militant attack and deals real damage.
8. Shroud linger: channel, then Stop. The cloud stays 1.5 s where it was, and a hit fired into it during that time is still absorbed. The buffer releases when the cloud vanishes. The same happens when the Disciple dies while channelling.
