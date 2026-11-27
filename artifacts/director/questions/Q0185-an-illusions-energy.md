# Q0185 — What energy does an illusion show?

- **Date**: 2026-11-27T12:03:20-05:00
- **Blocking**: no. The Mirage slice (M6-CM) will proceed on the assumption: an illusion shows its parent's current energy, live.
- **Docs**:
  - `factions/cults/units/disciple.md > Spells > Mirage` ("Seen by enemies: exactly like a Disciple, on the map and when selected; enemies can't tell them apart");
  - `factions/cults/units/disciple.md > Energy`;
  - design change 0448.

## Question
An illusion can't cast and spends no energy. What energy does it show on its bar and in its InfoPanel?

## Context
- **What is built (M5-N4a):** a Disciple shows a violet energy bar under its HP bar, and its InfoPanel reads "Energy N / 200". Everyone who sees the Disciple sees both, not just its owner. This was a placeholder the user didn't change.
- **What 0448 says:**
  - an illusion starts with its parent's current HP, MaxHP and speed;
  - to an enemy it looks exactly like a Disciple.
- **The gap:** 0448 says nothing about energy. If an illusion showed no energy bar, or a different number, an enemy could tell it from the real Disciple at a glance.

## Options
- **(a) The parent's current energy, live** (the assumption). Illusions always show the same number as their parent, so energy never gives them away.
  - When the parent channels Shroud, its energy drains and the illusions' does too. The cloud gives the real one away anyway, as the doc intends.
  - No new state: the number is read from the parent.
- **(b) A copy taken at the cast,** which then regenerates at the Disciple's rate (0.5/s) but is never spent.
  - The copy matches the parent's until the parent spends energy: on Shroud, or on its next Mirage, which replaces the set anyway.
  - Each illusion keeps its own number.
- **(c) No energy:** an illusion shows no energy bar and no energy line. Enemies can then tell them apart, against the doc's intent, unless energy becomes visible to its owner only.
- **(d) Energy visible to the owner only,** for every Disciple. This changes N4a's placeholder: an enemy sees no energy on any Disciple, so illusions need none.
