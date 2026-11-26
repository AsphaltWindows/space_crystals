# 0364 — New Cults caster vehicle, the Prayer-mobile (Inspire, Suppress); the Jackal is renamed Seeker

- **Date**: 2026-11-27T01:50:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/prayer_mobile.md` (new);
  - `factions/cults/units/seeker.md` (renamed from `jackal.md`), and every reference to the Jackal;
  - `factions/cults/units/README.md`; `README.md`.

## What changed and why
**Prayer-mobile** (user):
- A **wheeled caster vehicle with a megaphone**, with **no weapon**.
- Built at the **Chop Shop** from **2 Recruits**; needs a **Temple**.
- It has an energy pool.
- **Inspire**, a **cast**:
  - for a duration, own infantry around it gain movement speed, and Recruits also gain attack speed;
  - it stacks with augmentation, and a recast restarts the duration.
- **Suppress**, a screech in a **frontal arc**:
  - **every unit** in it, enemy or own, **air included**, temporarily **can't use weapon attacks or weapon-based abilities**;
  - a Rocketman keeps its Target Lock but can't fire;
  - buildings and non-weapon abilities are unaffected.
- Numbers, area and arc are placeholders, to follow.

**The Jackal is renamed Seeker** (user), in the same religious vein. Nothing else changes.

## Implementation impact
- Rename the Jackal to Seeker in code and UI.
- The Prayer-mobile is new: concept, with numbers coming.

## Verification
—
