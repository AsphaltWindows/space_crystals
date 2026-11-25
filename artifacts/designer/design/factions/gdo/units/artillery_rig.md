# Artillery Rig
GDO artillery vehicle produced at the Factory; requires Advanced Mechanics. Has two modes: it cannot attack while mobile and must deploy to fire long-range area artillery. (User reference: like the StarCraft 1 Siege Tank, but with no attack in its mobile mode.)

## Entity Type - Unit
## Faction - GlobalDefenseOrganization
## Produced at - Factory
## TechPrerequisite - player owns at least one Advanced Mechanics (see core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites)
## UnitBase - TrackedVehicle

## Modes
- **Mobile:** moves normally; **has no attack**.
- **Deployed:** cannot move; fires its artillery attack.
- **Deploy / Undeploy:** switching modes takes time (values TBD); the unit can neither move nor attack while switching.
- **Two unit types**: **Artillery Rig** and **Artillery Rig (Deployed)**, with separate SelectionGroups.
- **Deploy is an order, not a mode switch** (user, 2026-11-25; unlike Convert and the Riot Shield, which follow the toggle rule):
  - **The difference** (user, 2026-11-25): with a mode switch the unit's type changes **instantly** and the unit catches up in its own time. With deployment, **the deploying process is what actually changes the unit**: it stays the mobile type until the process completes.
  - **Queueable**: Shift-Deploy waits until the earlier orders are done (e.g. move here, then deploy). Without Shift it replaces the current orders and starts at once.
  - **No reversing mid-way**: **Undeploy is only available once deployment is complete**. The unit becomes Artillery Rig (Deployed) (type and SelectionGroup) when deploying finishes; until then it is still an Artillery Rig. (Undeploying is assumed symmetric: Deploy becomes available again only once undeploying is complete.)
  - **Not cancellable** (user, 2026-11-25; replaces the earlier wind-back on Stop): once deploying has started it always runs to completion; Stop doesn't interrupt it. Undeploying likewise always completes, and Deploy is available again only once it has (user, 2026-11-25; Q0071).
  - **Orders given while deploying** (user, 2026-11-25): the **last order given** (without Shift; each new one replaces the previous) is **held**, and it is the first thing the unit tries once deployment finishes. Shift-queued orders line up behind it as usual. At that point each order is checked against the deployed unit: one it can't carry out (e.g. **Move**, since a deployed unit can't move) is **dropped** with the usual unavailable-command feedback (user, 2026-11-25; Q0071), and the next one is tried.
  - **Not the same as the Colonists Drone's Deploy** (user, 2026-11-25): a Drone becomes a Beacon **immediately** and the transformation **can be cancelled** (factions/colonists/economy.md > Beacon Deployment).
  - When the switch makes a group disappear, the group the units joined becomes active (systems/controls/selection_and_hud.md > ActiveGroup > When the ActiveGroup disappears).

## AttackAttributes (Deployed only):
- TargetDomain: Ground
- TargetType: AoE — **friendly fire**: the blast damages friendly units too
- MinRange: yes (enemies closer than MinRange cannot be targeted; value TBD)
- Range: long (value TBD)
- Vision: like every attacker, it can fire at any enemy visible to its owner (systems/combat.md > ValidTarget) — it does not need its own vision of the target.
- AttackType: **TailDisjointed** — the target location is locked at the end of Aiming; the blast lands there at the end of Firing (no tracking projectile). Units that move out of the locked area in time dodge it. The Artillery Rig stays locked during Firing. Can also be fired at a ground location.
- Damage, AoE radius, timings: TBD

## TurretAttributes:
- TurnAngle (Deployed): 360° (full rotation)
- TurnRate: TBD

TBD: Silhouette, MaxHP, armor, SightRange, speed, cost, build time, Unit Control cost, hotkeys, ObjectInterfaceState.
