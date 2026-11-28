# Bastion

(Named by the user, 2026-11-28.)

(user, 2026-11-28) The Cults' **heavy front-line vehicle**: a **Garrison on treads**. It has **no weapon of its own**; up to 4 infantry ride inside and **fire out of it, even while it moves**. Slow, with high HP and moderate armor, it carries the Cults' infantry into the fight protected.

## Entity Type - Unit
## Faction - TheCults
## UnitBase - TrackedVehicle (turns in place; **directional armor** and **crushes enemy Light Infantry**, as every tracked vehicle: systems/units.md; user, 2026-11-28)
## Produced at - Chop Shop (consumes **4 Recruits**, designer placeholder; Unit Control cost 4, one per Recruit)
## TurretAttributes - None
## AttackAttributes - None (only its occupants fire)

## Garrisoning
Uses **Garrisoning, Garrison vehicle** (systems/mechanics/garrisoning.md), with the Cults Garrison's occupant rules (user, 2026-11-28):
- **Capacity 4**. **Who**: Recruits, Militants, Gunners, Disciples, and Disciples' illusions (as the Garrison).
- **Weapons inside**: a **Recruit** fires the **Militant's** attack; **Militants and Gunners** fire their own, with **+1 grid unit of Range**; **Disciples and illusions don't fire** (sheltered only).
- **Occupants fire while it moves** (user, 2026-11-28).
- **Range** is measured from the Bastion's **edge** (its Silhouette).
- **Protection**: occupants can't be targeted and take no damage while it stands; no abilities from inside.
- **Entering**: right-click the Bastion with eligible infantry (Enter); they walk to it and board.
- **Leaving**: **F: Eject All**: everyone comes out at once around it.
- **Destroyed** (user, 2026-11-28): **everyone inside comes out alive**, all at once, **where the vehicle stood** (spread over the area it occupied).
- **Sight**: occupants see with the Bastion's SightRange.

**Placeholder stats** (designer, 2026-11-28, user's standing permission; tune in the numbers pass):
- Silhouette **72x56 su** (length x width); MaxHP **600**; PointArmor **3**, FullArmor **3**; SightRange **6 grid units**; Groupable true.
- SpeedTurnRadiusMovement: MaxSpeed **3.5 su/frame** (Hand of Heaven: 5); Acceleration 0.3 su/frame²; Deceleration 1 su/frame²; **TrackWidth 40 su**; ReverseMaxSpeed 1.75 (spins about 10° a frame when stopped; can't turn at full speed).
- Cost: Chop Shop, **4 Recruits, 250 Space Crystals, 480 frames (30 s)**; hotkey **R** at the Chop Shop. No TechPrerequisite beyond the Chop Shop (placeholder).
- Panel: BasicCombatUnitInterfaceState without attack commands (Q Move, W Reverse, E Hold Position, X Stop), plus **F: Eject All**. Right-click: as a vehicle without an attack; own eligible infantry right-clicking it → Enter.
