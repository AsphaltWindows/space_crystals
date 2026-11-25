# Laser Turret

Syndicate defensive structure capable of attacking both ground and air units. Its form depends on where it stands: inside one of the player's Tunnel Areas it is a hidden **pop-up turret**; outside every Tunnel Area it is an **upright turret**.

## Entity Type - Structure Type (Surface / pop-up)
## Size - 1x1
## Constructed by - Agent (anywhere) or Tunnel (within its own Tunnel Area, via the ExpandMenu)
## TechPrerequisite - player owns at least one Optics Lab (anywhere in the Tunnel Network)
## AttackAttributes.TargetDomain - Universal

## Upright vs Pop-up (by location)
- **Upright turret** (outside every own Tunnel Area): a normal surface structure, always visible. Costs no Tunnel Space.
- **Pop-up turret** (inside an own Tunnel Area, whoever built it):
  - Occupies 1 grid unit of that Tunnel's tunnel-region, like an expansion, so it reduces Tunnel Space (¼ Tunnel Space, subject to the rounding rule in Tunnel Space).
  - While it has no target it stays **underground**, hidden like an expansion (invisible to enemies without detection).
  - It notices enemies the same way an upright turret does. When an enemy comes into range it rises to the surface **already facing the target** and fires; it retracts once it has no target. Rise and retract timing: see placeholder stats below.
  - It **can be attacked while underground** (by enemies that can see it), but it has **much higher armor while underground** than when raised (values in the placeholder stats below).

**Pop-up rules** (user, 2026-11-26; Q0084):
- **Hidden only while fully lowered**: that is when it is invisible to enemies without detection and has its underground armor. While rising, raised and retracting it is visible and has its raised armor.
- **Sight 6 grid units in both forms**: a lowered turret notices enemies by itself and rises.
- **Blocking**: an **upright** turret blocks movement like any building, and so does a **raised** pop-up. A lowered pop-up never blocks (units walk over it like an expansion).
- **Rising needs a clear footprint**: it can't rise while any unit stands over it. The **owner's own idle units** over it are pushed off by the idle push (systems/units.md > Ground Collision > Idle units push apart). Any other unit over it (an enemy, or an own unit with an order) keeps it **underground** until the spot is clear.

## Conversion
The form updates automatically whenever Tunnel Areas change:
- A new Tunnel, or a Tunnel upgrade, whose Tunnel Area now covers an upright turret converts it into a pop-up turret (it starts occupying that Tunnel's tunnel-region).
- When a Tunnel is destroyed, the pop-up turrets in its Tunnel Area become upright turrets (their tunnel-region is lost along with the Tunnel).
- If a conversion pushes Tunnel Space below the player's current usage, the over-capacity rule applies (existing units unaffected; production waits — see Tunnel Space).

## Construction
- **By an Agent**: built using the same flow as Tunnel construction (see Agent > Building > Tunnel Construction Flow): the Agent enters the partially-built structure and is untargetable for the duration; if the structure is destroyed or cancelled mid-construction the Agent survives and emerges at the location. On successful completion, the Agent emerges on the surface adjacent to the finished structure (unlike Tunnels, where the Agent ends up inside the Tunnel Network). If built inside a Tunnel Area, it is a pop-up turret on completion.
- **By a Tunnel**: from the Tunnel's ExpandMenu, placed within that Tunnel's Tunnel Area like an expansion. It **uses the Tunnel's single operation slot**, like an expansion (user, 2026-11-25; Q0071): while a Tunnel builds a turret it can't build an expansion or upgrade.
- **Paying** (user, 2026-11-26; Q0084): an Agent's turret is paid **when the order is given** (ghost placed), like an Agent's Tunnel (factions/syndicate/tunnels.md > Construction Cost), with the same refunds if the order is cancelled, replaced, fails, or the Agent is killed before construction starts. A Tunnel's turret is paid when placed, like an expansion.
- **Cancelling a site** (user, 2026-11-26; Q0084): **75% refund** for both, Agent-built or Tunnel-built (like an expansion).
- **What enemies see of a site** (user, 2026-11-26; Q0084): each follows its flow. An **Agent's site is visible** on the surface like a Tunnel site (the Agent inside is not). A **Tunnel's site is hidden** like an expansion under construction.

**Placeholder stats** (designer, 2026-11-25, Q0071; tune later):
- Cost: 100 Space Crystals (no Supplies). Build time: 320 frames (20 s).
- HP: 300. SightRange: 6 grid units.
- Armor (building armor rule, 1x1): raised PointArmor 1, FullArmor 1; underground (pop-up) PointArmor 6, FullArmor 6.
- AttackAttributes: FullyConnected red laser (instant, cannot miss); TargetDomain Universal; SingleTarget; Damage 24; Range 5 grid units; MinRange 0; Aim 4, Firing 1, Cooldown 3, Reload 16 frames (one shot per 24 frames, 16 damage per second).
- Pop-up: rises in 8 frames (already facing its target); retracts after 32 frames with no target, taking 8 frames. It can't fire while rising or retracting.
- Hotkey: Agent **D** (Q0068); in the Tunnel's ExpandMenu it takes the next slot after the expansions.
