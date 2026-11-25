# 0207 — New Syndicate unit: Laser Battery; Syndicate lasers are red

- **Date**: 2026-11-25T04:15:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md` (header: red lasers; `> Laser Battery` new); `factions/syndicate/structures.md > Vehicle Assembly` (Produces)

## What changed and why
User:
- **All Syndicate lasers are red** (Guard, Laser Battery, Laser Turret).
- **Laser Battery**: a TrackedVehicle from the Vehicle Assembly.
  - **Weapon:** a FullyConnected laser, cannot miss, fairly long range, ground only while mobile. Its turret has little mobility.
  - **Battery:** charges while the unit isn't attacking. Normal attacking drains it at ~2× the charge rate.
  - **Attack visual:** a constant thin beam; periodic surges do the damage.
  - **Damage per surge** ∝ (charge used)^(2/3).
  - **Amplify:** spends the whole remaining battery on the next surge (burst at reduced efficiency).
  - **Deploy:** immobile, 360° turret, air targets only, plus **Refract** (to be designed with satellites).

## Implementation impact
Concept only; all numbers and several rules are TBD and being settled with the user. The beam-plus-surge model and the charge-based damage formula are new to the combat system.

## Verification
—
