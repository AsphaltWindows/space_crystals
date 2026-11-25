# Laser Battery
(user, 2026-11-25) Syndicate laser vehicle running on a **battery**. Treaded. Fires a **red laser** (FullyConnected: instant, cannot miss) at **fairly long range**, **ground only** while mobile. Its turret has **relatively little mobility**. It can **deploy**, becoming an immobile **anti-air** emplacement with a full-rotation turret.

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - TrackedVehicle (exits only through Tier 2+ Tunnels)
## Produced at - Vehicle Assembly
## TechPrerequisite - player owns at least one **Optics Lab** (user, 2026-11-25)
## Groupable - true (assumed)

## Battery
- The Laser Battery has a **battery charge** that is **always charging** at its charge rate, attacking or not (user, 2026-11-25).
- **A normal surge draws twice what charges in one surge interval**, so while attacking with charge left the battery drains at a net **1× the charge rate**. Resting refills it.
- **Empty battery** (user, 2026-11-25): it keeps firing **just as often**, but each surge uses only what charged since the last one (half a normal surge's charge), so by the ^(2/3) rule it deals about **63%** ((½)^(2/3)) of a normal surge's damage. An exhausted Laser Battery still fights at about two-thirds strength.
- More generally, a surge uses the normal surge charge if the battery holds it, otherwise whatever is left; its damage follows the ^(2/3) rule.
- TBD: battery size, charge rate (numbers); whether it charges while switching modes (assumed yes: always charging).

## Attack: constant beam with surges
- **Animation**: while attacking, a **constant thin red laser** connects the Laser Battery to its target. Periodic **surges** along the beam are what **actually do the damage**.
- **Damage per surge** is proportional to the **charge used^(2/3)** (the square of the cube root of the charge spent on that surge). Spending charge in many small surges deals more total damage than spending the same charge in one big surge, so big surges trade efficiency for burst.
- AttackType: FullyConnected (cannot miss)
- TargetDomain: Ground (mobile); Air (deployed, below)
- TargetType: SingleTarget (assumed)
- Range: fairly long
- TBD: surge interval, charge used per normal surge, damage constant, range, aiming time.

## Amplify (ability)
Converts the **entire remaining battery** into the **next surge**. A single big hit for burst damage, at the cost of efficiency (per the ^(2/3) rule) and of an empty battery afterwards.

**The trade is a tuning dial** (user, 2026-11-25: a reasonable trade; its size depends on the numbers). If a full battery holds N normal surges, an Amplified full battery deals N^(2/3) normal surges' worth of damage instead of N, i.e. N^(−1/3) of the total: N = 4 → ~63%, N = 8 → 50%, N = 27 → ~33%. So the share of the battery a normal surge uses sets how costly Amplify is; choose it in the numbers pass.
- TBD: hotkey; whether it can be used while deployed; whether it is queueable or a mode-like toggle.

## TurretAttributes (mobile):
- TurnAngle: **narrow arc** (user, 2026-11-25): it can only aim within a cone ahead of the hull, so the tank must roughly face its target (like the Assault Bike). Value TBD.

## Deploy (ability)
- **Deployed**: **cannot move**. The turret gets **360° rotation**. It can **only target air units** (no ground targets while deployed).
- **Relay** (ability, deployed only; user, 2026-11-25; renamed from "Refract"): the deployed Laser Battery reflects its beam through the player's **Prisms** to hit far-off **ground or air** targets. See **Relay** below.
- **Switching modes works like the GDO Artillery Rig** (user, 2026-11-25; factions/gdo/units/artillery_rig.md > Modes): deploying and undeploying take time, during which it can **neither move nor attack**.
- **Two unit types**: **Laser Battery** and **Laser Battery (Deployed)**, with separate SelectionGroups.
- **Deploy is an order, not a mode switch** (user, 2026-11-25; unlike Convert and the Riot Shield, which follow the toggle rule):
  - **The difference** (user, 2026-11-25): with a mode switch the unit's type changes **instantly** and the unit catches up in its own time. With deployment, **the deploying process is what actually changes the unit**: it stays the mobile type until the process completes.
  - **Queueable**: Shift-Deploy waits until the earlier orders are done (e.g. move here, then deploy). Without Shift it replaces the current orders and starts at once.
  - **No reversing mid-way**: **Undeploy is only available once deployment is complete**. The unit becomes Laser Battery (Deployed) (type and SelectionGroup) when deploying finishes; until then it is still a Laser Battery. (Undeploying is assumed symmetric: Deploy becomes available again only once undeploying is complete.)
  - **Not cancellable** (user, 2026-11-25; replaces the earlier wind-back on Stop): once deploying has started it always runs to completion; Stop doesn't interrupt it. Undeploying likewise always completes, and Deploy is available again only once it has (user, 2026-11-25; Q0071).
  - **Orders given while deploying** (user, 2026-11-25): the **last order given** (without Shift; each new one replaces the previous) is **held**, and it is the first thing the unit tries once deployment finishes. Shift-queued orders line up behind it as usual. At that point each order is checked against the deployed unit: one it can't carry out (e.g. **Move**, since a deployed unit can't move) is **dropped** with the usual unavailable-command feedback (user, 2026-11-25; Q0071), and the next one is tried.
  - **Not the same as the Colonists Drone's Deploy** (user, 2026-11-25): a Drone becomes a Beacon **immediately** and the transformation **can be cancelled** (factions/colonists/economy.md > Beacon Deployment).
  - When the switch makes a group disappear, the group the units joined becomes active (systems/controls/selection_and_hud.md > ActiveGroup > When the ActiveGroup disappears).
- The battery keeps charging in every mode (Battery: always charging).
- TBD: deploy/undeploy time.

## Relay (ability, deployed only)
(user, 2026-11-25) Only a **deployed** Laser Battery can relay; no other laser uses Prisms.

**Giving the order**
1. With deployed Laser Batteries selected, choose **Relay**. This **highlights every Prism they can relay to**: each Prism **in range of the Batteries themselves**, plus every Prism connected to one of those by a **contiguous chain** of Prisms.
2. Click one of the highlighted Prisms: the **target Prism**.

**The relay**
- Each selected Battery **locks onto its nearest Prism** with its normal thin beam.
- The beams are **reflected as one beam** along the chain of Prisms to the target Prism.
- The target Prism becomes **armed**. It **targets enemies in its own range at will**, ground or air, like an attacking unit.
- **Several Batteries** can feed the same relay.

**Firing**
- The armed Prism attacks at roughly the **Batteries' own attack frequency**. Each surge **discharges the Batteries as normal** (Battery).
- **Animation**, at every link:
  - surges do the damage;
  - between surges, the thin, harmless (0 damage) beam stays visible along the whole path: Battery → Prism(s) → the armed Prism → its current target.

**While relaying**
- The Batteries are **locked in**: they do nothing else.
- Their commands are:
  - **Stop Relaying**;
  - **Relay**, which picks another target Prism;
  - **Amplify**: available while relaying **if the target Prism has an active target**. Every selected Battery spends its **entire battery** on the next surge (Amplify).

**Every selected Battery must reach the chain** (user, 2026-11-25): the selected deployed Batteries act as one group, so **each** of them must have at least one Prism in its own range. If any doesn't, **no relay targets are highlighted**. Highlighted targets are the Prisms reachable from **every** selected Battery (designer's reading).

**Sight** (user, 2026-11-25): the armed Prism can attack **anything the player sees** within its range (any own vision, not only its own).

**Directing the armed Prism** (user, 2026-11-25): besides auto-targeting, the player can **select the armed Prism and give it a specific target** (Attack).

**Several Batteries add up** (user, 2026-11-25): each surge of the armed Prism deals the **sum** of each relaying Battery's own surge damage (each computed from its own charge by the ^(2/3) rule). Four Batteries deal four times one Battery's damage.

**Chain breaks** (user, 2026-11-25): if the chain breaks (a Prism on it is destroyed or a link goes out of range), the Batteries **stop relaying automatically** and go back to ordinary deployed behavior.

TBD:
- link range between Prisms, and Battery-to-Prism range;

All numbers (HP, armor, sight, speed, TunnelSpaceCost, battery, damage, range, timings, cost, build time): TBD.
