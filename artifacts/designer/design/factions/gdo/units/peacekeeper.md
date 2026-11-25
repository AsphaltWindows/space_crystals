# Peacekeeper
GDO basic infantry unit. A light infantry soldier armed with a fully connected ground attack.

Targets ground only. This is intended: there are no air units in the early game, and GDO's anti-air comes later (Arsenal missile infantry, Missile Turret).

Lore: "Peacekeeper" is the official name for a sanctioned external operator — hired security working under GDO license beyond the borders (see core/lore.md > Factions > GDO).

Look: corporate private security — polished body armor, branded, standardized gear.

Role intent: in the early game the Peacekeeper should feel like a great all-rounder. Later in the game it may become slightly redundant as specialized units take over.

## Faction - GlobalDefenseOrganization
## Silhouette - 24x24 space units (square)
## MaxHP - 50
## PointArmor - 1
## FullArmor - 1
## SightRange - 5 grid units
## Destructible - true
## Groupable - true
## UnitBase - LightInfantry
## UnitControlCost - 1

## UnitBaseAttributes[LightInfantry]:
- TurnRate: 180 degrees/frame
- Acceleration: infinite
- Deceleration: infinite
- MaxSpeed: 4 space units/frame
- RuggedTerrainDefenseBonus: 50%

## TurretAttributes - None

## AttackAttributes:
- AttackType: FullyConnected
- TargetDomain: Ground
- TargetType: SingleTarget
- Damage: 10
- Range: 4 grid units
- MinRange: 0
- AimDuration: 2 frames
- FiringDuration: 1 frame
- CooldownDuration: 2 frames
- ReloadDuration: 12 frames

## Riot Shield (toggled ability)
The Peacekeeper carries a riot shield that can be raised or lowered (toggle).

**Two unit types** (user, 2026-11-24): shield down and shield up are **two unit types**, **Peacekeeper** and **Peacekeeper (Shielded)**, like the Sabre and Rifle Legionnaire (factions/colonists/units/weapon_conversion.md). They share everything except what the shield changes (below). Raising or lowering the shield switches the unit's type.
- In a selection, the two types form **separate SelectionGroups**; the shield command goes to the ActiveGroup. Example: 6 shielded + 4 unshielded selected → two groups; Tab to the 4, raise → all 10 are one type and form one group.
- The Peacekeeper's panel shows **Raise Shield**; Peacekeeper (Shielded)'s shows **Lower Shield** and no Attack.
- **A toggle sets a target state** (user, 2026-11-25; Q0049/Q0053, same rule as factions/colonists/units/weapon_conversion.md > A toggle sets a target form): pressing R flips the Peacekeeper's target (shield up or down), and the unit works to reach it. The type, and so the SelectionGroup, switches **at the press**, including when the raise is deferred behind a shot. The pressed units join the other type's group, and **that merged group becomes the ActiveGroup**, so a second press reaches all of them and everyone ends in the same state.
- The Barracks produces the unshielded Peacekeeper. Cost and Unit Control are the same for both types.

While the shield is **up**:
- MaxSpeed is halved (4 → 2 space units/frame).
- The Peacekeeper **cannot attack**.
- Incoming damage is halved — a 50% **pre-armor** reduction (step 1 of systems/combat.md > DamageModifierOrder), multiplying with other pre-armor reductions. The reduction applies to damage from **all directions**.
- Attack orders are not possible: the player must lower the shield before ordering an attack. The Peacekeeper does not auto-target enemies while the shield is up.
- **Explicit Attack (A + click) in a mixed selection** (user, 2026-11-24): Peacekeepers whose shield is up are **left out** of the order and keep their current orders; the units that can attack carry it out. This follows the CommandPanel rule that a command goes only to objects that can carry it out (systems/controls/selection_and_hud.md > CommandPanel). It applies per unit, so in a Peacekeeper group with some shields up and some down, only the lowered ones attack. Note the difference from right-click (below), where shielded Peacekeepers follow. A + click on **ground** (AttackMove) is different: shielded Peacekeepers take part and simply move, never stopping to fight (systems/controls/commands.md > AttackMove > Units that can't attack).

While the shield is **down**, the Peacekeeper behaves normally.

**Toggle time:** raising and lowering the shield takes a short time, so it cannot be flicked on and off between enemy shots. **6 frames (0.375 s), placeholder** (user, 2026-11-24; should stay under half a second).

**During the toggle** (user, 2026-11-24):
- While raising **or** lowering, the Peacekeeper moves at half speed (as with the shield up) and **cannot attack**.
- The 50% damage reduction applies **only once the shield is fully up**; a half-raised shield gives none.
- Pressing the toggle again mid-way **reverses from the current point** (halfway up → lowering takes only the half it already moved).
- **Pressed mid-attack** (user, 2026-11-24): pressed during the Firing or Cooldown phase (not interruptible), the raise is **deferred**: the shot finishes, then the shield starts rising immediately. Same as the Legionnaires' Convert (factions/colonists/units/weapon_conversion.md > Eligibility), so one press raises a whole fighting group.
- **Lower pressed while a raise is still deferred** (user, 2026-11-25; Q0053): the target flips back to shield down, so the waiting raise is dropped; the shield never starts rising.
- **Orders given during the toggle** (user, 2026-11-25; Q0053): **which orders the unit accepts follows its new type from the press; carrying them out waits for the toggle to finish.** While **lowering**, it is already a Peacekeeper: an explicit Attack or a right-click on an enemy is **accepted and held**, and the attack starts as soon as the shield is fully down. While **raising**, it is already Peacekeeper (Shielded): no Attack, and a right-click on an enemy follows (as with the shield up).

**Right-click on an enemy while the shield is up** (user, 2026-11-24): the Peacekeeper can't attack, so it does what any unit that can't carry out a right-click does (systems/controls/selection_and_hud.md > RightClickResolution): it **follows** an enemy unit (Move to that unit) and moves to an enemy structure. In a mixed group, the other units attack while the shielded Peacekeepers stay with them.

**Orders are kept** (user, 2026-11-24): raising or lowering the shield is a mode switch, not an order; the Peacekeeper's orders stay. An order it can't carry out while shielded is **on hold**, not dropped:
- an explicit **Attack** order: while the shield is up it follows the target (as with a right-click while shielded); when the shield is lowered it **resumes attacking** that target;
- an **AttackMove** or Patrol: while the shield is up it only moves (systems/controls/commands.md > AttackMove > Units that can't attack); once lowered it engages along the way again.
Same principle as the Colonists' Weapon Conversion (factions/colonists/units/weapon_conversion.md > Orders are kept).

**Not queueable** (user, 2026-11-24): the toggle always happens immediately; holding Shift changes nothing.

**Hotkey: R** (Raise Shield on Peacekeeper, Lower Shield on Peacekeeper (Shielded)), placeholder (user, 2026-11-24).

## ObjectInterfaceState: BasicCombatUnitInterfaceState
