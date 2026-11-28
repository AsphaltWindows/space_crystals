# 0485 — The Bastion: Cults heavy front-line vehicle (a Garrison on treads)

- **Date**: 2026-11-28T14:45:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/bastion.md` (new)
  - `systems/mechanics/garrisoning.md > Garrison vehicle` (now defined)
  - `factions/cults/structures/chop_shop.md` (Production, Placeholder stats, panel R)
  - `systems/controls/commands.md > Enter`
  - `README.md`, `factions/cults/units/README.md`

## What changed and why
The user designed the Cults' front-line vehicle, the **Bastion**, as a "Garrison on treads".
- It is **tracked**, with directional armour, and **crushes** enemy Light Infantry. It is slow, with high HP and moderate armour.
- It has **no weapon of its own**. Up to **4 infantry** ride inside and fire out, with the Cults Garrison's occupant rules:
  - Recruits fire as Militants;
  - Militants and Gunners fire their own weapons with +1 range;
  - Disciples and illusions shelter but don't fire.
- Occupants **fire while it moves**, and range is measured from its edge.
- They are protected, and they **come out alive where it stood** when it is destroyed.

This is the first unit composed from the mechanics docs. It defines the **Garrison vehicle** archetype: fires on the move, range from the Silhouette edge, Eject All around it, occupants come out where it stood.

## Implementation impact
- **New unit**, composed of the TrackedVehicle base and Garrisoning, Garrison vehicle.
- **Chop Shop:** **R: Build Bastion**.
- **Enter:** Enter targets include the Bastion.
- **Placeholders** (designer):
  - body: 72x56 su, 600 HP, armour 3/3, sight 6;
  - movement: MaxSpeed 3.5, Acceleration 0.3, Deceleration 1, TrackWidth 40, ReverseMaxSpeed 1.75;
  - cost: 4 Recruits, 250 SC, 480 frames; no TechPrerequisite beyond the Chop Shop;
  - panel: Q Move, W Reverse, E Hold Position, X Stop, F Eject All.

## Verification
1. Build a Bastion at the Chop Shop with 4 Recruits.
2. Load 2 Militants, a Gunner and a Recruit, then drive it at enemies. All four fire while it moves. The Militants and the Gunner outrange a lone Militant by 1 GU, and range is measured from the hull's edge.
3. Enemies can't target the occupants.
4. It crushes enemy infantry it drives over.
5. Hits on its front do less damage than hits on its rear.
6. Destroy it. All four occupants appear alive where it stood.
7. Press F. Everyone comes out around it.
