# Prayer-mobile

(Name: user, 2026-11-27.)

(user, 2026-11-27) A Cults **energy-based caster vehicle**: a wheeled vehicle carrying a **megaphone**, with **no weapon** (a pure caster). It boosts nearby Cults infantry and silences enemies in front of it.

## Entity Type - Unit
## Faction - TheCults
## UnitBase - WheeledVehicle
## Produced at - Chop Shop (consumes **2 Recruits**; Unit Control cost **2**)
## TechPrerequisite - player owns at least one **Temple** (core/tech_and_research.md > Unit Prerequisites)
## AttackAttributes - None (pure caster)
## Energy - has an energy pool for its abilities (values TBD)

## Abilities
- **Inspire**: buffs nearby own infantry with **increased movement speed**, and **Recruits** also with **increased attack speed**. **A cast** (user, 2026-11-27): it spends energy and buffs the infantry around it at that moment for a set duration. Area, size of the buff and duration: placeholders to follow.
  - **Who** (user, 2026-11-27): every own infantry type (Recruit, Militant, Gunner, Disciple) gets the movement speed; Recruits also get the attack speed.
  - **Stacks with augmentation** (e.g. Improved Metabolism): different sources add up. **Recasting** on units that are already inspired **restarts the duration**; Inspire doesn't stack with itself.
- **Suppress**: a screech in an **arc in front of it**. Affected units **temporarily can't attack**. - **Affects every unit in the arc, enemies and own alike** (user, 2026-11-27), like friendly fire. **Not buildings**: attacking structures (turrets, Garrisons) are unaffected. **Air units in the arc are affected** too (user, 2026-11-27).
  - **What it stops** (user, 2026-11-27): **weapon attacks** and **abilities that use a weapon**. A Rocketman's **Target Lock can be maintained**, but no missile can be fired while suppressed. Other abilities (e.g. a Disciple's Shroud) are unaffected.
  - Arc size, duration and cost: placeholders to follow.

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 56x40 space units; MaxHP 150; PointArmor 1, FullArmor 1; SightRange 7 grid units; **Groupable false** (one cast spends one vehicle's energy, as for the Disciple).
- FixedTurnRadiusMovement as the Seeker, but ForwardMaxSpeed **6 su/frame**.
- Energy: MaxEnergy 200; starts at 50; regenerates 0.5 per second.
- **Inspire** (hotkey **R**, instant, no target): 75 energy; every own infantry unit within **4 grid units** of the vehicle gets **+1.5 su/frame MaxSpeed** for **10 s**; Recruits also attack **25% faster** (their attack cycle × 0.75) for the same time.
- **Suppress** (hotkey **F**, instant, no target): 100 energy; an arc of **90° total** in front of the vehicle's current facing, **5 grid units** long; units whose centre is inside can't attack for **4 s**. The driver aims it by pointing the vehicle (user, 2026-11-27: "for now"; the alternative, clicking a direction, may be revisited).
- Cost: Chop Shop, 2 Recruits, **125 Space Crystals, 400 frames (25 s)**; requires a Temple.
- ObjectInterfaceState: BasicCombatUnitInterfaceState without attack commands, plus **R: Inspire** and **F: Suppress**.
