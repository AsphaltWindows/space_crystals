# Laser Turret

Syndicate defensive structure capable of attacking both ground and air units. **Only Tunnels build it** (user, 2026-11-26), from the ExpandMenu inside their Tunnel Area: it is always a hidden **pop-up turret**, and is **only active inside its Tunnel's area**. When its Tunnel is lost it **behaves like every other expansion** (see When its Tunnel is lost). There is no upright form any more.

## Entity Type - Structure Type (pop-up; a Tunnel expansion in all but name)
## Size - 1x1
## Constructed by - Tunnel only (within its own Tunnel Area, via the ExpandMenu; user, 2026-11-26: Agents no longer build it)
## TechPrerequisite - player owns at least one Optics Lab (anywhere in the Tunnel Network)
## AttackAttributes.TargetDomain - Universal

## Pop-up turret
  - Occupies 1 grid unit of that Tunnel's tunnel-region, like an expansion, so it reduces Tunnel Space (¼ Tunnel Space, subject to the rounding rule in Tunnel Space). **A turret site takes its Tunnel Space from the start of construction** (user, 2026-11-26; Q0091).
  - **Built from a Tunnel's ExpandMenu**: on ground that is **Buildable, flat and Drillable** (it must be able to rise), and from **Tier 1** (no Tier Requirement) (user, 2026-11-26; Q0091).
  - While it has no target it stays **underground**, hidden like an expansion (invisible to enemies without detection).
  - It notices enemies by itself (sight 6). When an enemy comes into range it rises to the surface **already facing the target** and fires; it retracts once it has no target. Rise and retract timing: see placeholder stats below.
  - It **can be attacked while underground** (by enemies that can see it), but it has **much higher armor while underground** than when raised (values in the placeholder stats below).

**Pop-up rules** (user, 2026-11-26; Q0084):
- **Hidden only while fully lowered**: that is when it is invisible to enemies without detection and has its underground armor. While rising, raised and retracting it is visible and has its raised armor.
- **Sight 6 grid units**, lowered or raised: a lowered turret notices enemies by itself and rises.
- **Blocking**: a **raised** turret blocks movement like any building. A lowered one never blocks (units walk over it like an expansion).
- **Rising needs a clear footprint**: it can't rise while any unit stands over it. The **owner's own idle units** over it are pushed off by the idle push (systems/units.md > Ground Collision > Idle units push apart). Any other unit over it (an enemy, or an own unit with an order) keeps it **underground** until the spot is clear.

## When its Tunnel is lost
(user, 2026-11-26; replaces the conversion to an upright turret, and Q0091 point 1) The turret **behaves exactly like the Tunnel's other expansions** (factions/syndicate/tunnels.md > Losing Tunnels): it becomes **neutral**, **visible to everyone**, keeps its **high underground armor** (a raised turret retracts at once; it never fires again while neutral), and **slowly decays** (1% of MaxHP per second, placeholder). A new Tunnel whose area covers it **reclaims** it like an expansion (Tier 1 suffices), and it is active again.

## Construction
(user, 2026-11-26: **only Tunnels build turrets**; the Agent-built turret, with its paying, exit, slot and site-visibility rules from Q0084, is removed)
- **By a Tunnel**: from the Tunnel's ExpandMenu, placed within that Tunnel's Tunnel Area like an expansion. It **uses the Tunnel's single operation slot**, like an expansion (user, 2026-11-25; Q0071): while a Tunnel builds a turret it can't build an expansion or upgrade. Paid when placed, like an expansion. **Cancelling** the site refunds **75%** (like an expansion). The **site is hidden** like an expansion under construction.
- **Agents still repair** turrets (factions/syndicate/units/agent.md > Repair).

**Placeholder stats** (designer, 2026-11-25, Q0071; tune later):
- Cost: 100 Space Crystals (no Supplies). Build time: 320 frames (20 s).
- HP: 300. SightRange: 6 grid units.
- Armor (building armor rule, 1x1): raised PointArmor 1, FullArmor 1; underground (pop-up) PointArmor 6, FullArmor 6.
- AttackAttributes: FullyConnected red laser (instant, cannot miss); TargetDomain Universal; SingleTarget; Damage 24; Range 5 grid units; MinRange 0; Aim 4, Firing 1, Cooldown 3, Reload 16 frames (one shot per 24 frames, 16 damage per second).
- Pop-up: rises in 8 frames (already facing its target); retracts after 32 frames with no target, taking 8 frames; **a new target during retracting doesn't interrupt it**: it finishes retracting, then rises (user, 2026-11-26; Q0091). It can't fire while rising or retracting.
- Hotkey: in the Tunnel's ExpandMenu it takes the next slot after the expansions.

**Targeting** (user, 2026-11-26; Q0084): the turret has no commands and targets like a unit holding position: BaseAutoTargeting within Range (systems/controls/behaviors.md), then the closest, ties broken deterministically. It **keeps its target** while the target stays valid and in range.
