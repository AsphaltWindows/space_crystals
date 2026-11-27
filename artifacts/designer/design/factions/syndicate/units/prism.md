# Prism
(user, 2026-11-25; named by the user) The Syndicate's **satellite**: an unmanned orbital unit whose function, **for now, is only laser relay**: deployed Laser Batteries reflect their beams through it (one or more Batteries; Prisms can be chained), and the target Prism of a relay becomes **armed**, attacking ground or air enemies in its range (Laser Battery > Relay).

## Entity Type - Unit
## Faction - TheSyndicate
## UnitBase - HoverCraft (Air domain; factions/syndicate/structures/aerospace_assembly.md)
## Produced at - Aerospace Assembly
## TechPrerequisite - player owns at least one **Orbital Command** (user, 2026-11-25)
## Manned - no (no Null Chamber buff)

## Launch
It leaves the network through a Tier 3 Tunnel's **air exit**, **launched like a rocket straight up**, and **transforms in the air into its orbital form** (user, 2026-11-25). It **can be shot down while launching and unfolding** (user, 2026-11-25).

## One way only
It is the **only unit that can't get back into the Tunnel Network** (user, 2026-11-25). Nothing recharges it inside, Vault or otherwise.

## Movement
- **Very slow** by default.
- **Directional thrust**: a few **charges**, each giving a quick burst of displacement in a chosen direction (user, 2026-11-25).
- **Charges never come back** (user, 2026-11-25): once they are spent it only drifts slowly.

## Sight
It sees through Shadebringer smoke (Shadebringer > Cover).

**Placeholder stats** (designer, 2026-11-27, user's standing permission; tune in the numbers pass):
- Silhouette 40x40 space units; MaxHP 100; PointArmor 0, FullArmor 0; SightRange 6 grid units; TunnelSpaceCost 2.
- Very slow drift: top speed **1 su/frame**. DragMovement (designer placeholder 2026-11-26, Q0154): ForwardAcceleration 0.05; OmniDirectionalAcceleration 0.05; DragRatio 0.1; TurnRate 4°/frame. The thrust charges below are separate bursts, not part of this.
- **Launch** 32 frames (straight up) and **transform** 32 frames (vulnerable during both).
- **Thrust charges: 3**; each moves it **3 grid units** over 1 s in a chosen direction.
- Relay ranges: Laser Battery > Placeholder stats.
- Cost: Aerospace Assembly, 250 Space Crystals, 480 frames.
- **Panel** (designer placeholder, 2026-11-26; Q0168): **Q** Move, **R** Thrust, **X** Stop; while **armed** (a relay target, Laser Battery > Relay) also **A** Attack (A is Attack on every unit that can attack).
- **Thrust** (R, targeted; Q0168): left-click a point; the Prism thrusts **toward it**, always the full 3 grid units whatever the distance. Like any command it **queues with Shift** (systems/controls/commands.md), so thrusts can be chained along a path; each uses a charge when it starts. Greyed with no charges left.
- **Orders during launch and unfolding** (Q0168): **held**, as in the vehicle deploy model (factions/gdo/units/artillery_rig.md): the last order given during the 4 s is carried out once it has unfolded.
