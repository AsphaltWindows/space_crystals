# Q0084 — Laser Turret: the rules its placeholders leave open

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, marked as placeholders in M4-S1 (upright, Agent-built) and M4-S2 (pop-up, Tunnel-built).
- **Docs**: `factions/syndicate/structures/laser_turret.md` (Upright vs Pop-up, Conversion, Construction, placeholder stats); `factions/syndicate/units/agent.md > Building`; `factions/syndicate/tunnels.md > Expansions under construction`, `> Cost Scaling`; `systems/controls/behaviors.md > BaseAutoTargeting`

## Question
1. **When is a pop-up hidden and at 6/6?** Only while fully lowered, or also while rising and retracting?
2. **Its sight.** 6 in both forms?
3. **Blocking.** Does an upright turret block movement like a building? Does a pop-up, raised or lowered?
4. **Paying.** Is an Agent's turret paid when the order is given (like a Tunnel, 0233), or when construction starts?
5. **Refunds.** Cancelling an Agent's turret site: 100%? A Tunnel-built turret site: 75%, like an expansion (0254)?
6. **What enemies see of a site.** An Agent's site like a Tunnel site (visible, the Agent inside not)? A Tunnel's site hidden like an expansion under construction?
7. **The Agent's exit.** "the Agent emerges on the surface adjacent to the finished structure": on which side, if several are free, and if none is?
8. **Targeting.** The turret has no commands. Which enemy does it pick, and does it keep its target?
9. **The operation slot.** Does an Agent building a turret inside a Tunnel Area use that Tunnel's operation slot?

## Context
- The design: "While it has no target it stays **underground**, hidden like an expansion"; "it rises to the surface **already facing the target** and fires; it retracts once it has no target"; "It can't fire while rising or retracting"; armour "raised PointArmor 1, FullArmor 1; underground (pop-up) PointArmor 6, FullArmor 6".
- "**By an Agent**: built using the same flow as Tunnel construction"; "**By a Tunnel**: ... placed within that Tunnel's Tunnel Area like an expansion. It **uses the Tunnel's single operation slot**".
- No rule names a structure's target choice; `BaseAutoTargeting` is written for units.
- 0267: hidden things "can only be hurt by area damage over them" until detection.

## Options
1. **Hidden:** **A (proposed)** only while fully lowered; rising, raised and retracting it is visible at 1/1. **B** hidden until fully raised: it would fire its first shot unseen, and a rising turret couldn't be shot back.
2. **Sight:** **A (proposed)** 6 in both forms, so a lowered turret can notice enemies ("notices enemies the same way"). **B** none while lowered: it could only rise when an enemy is seen by something else.
3. **Blocking:** **A (proposed)** upright blocks like any building; a pop-up never blocks, since raising under units would trap them. **B** a raised pop-up blocks: units under it must be pushed out, which is new movement work.
4. **Paying:** **A (proposed)** at the order, as 0233 does for Tunnels, with the same refunds. **B** at construction start, like the GDO's direct placement.
5. **Refunds:** **A (proposed)** Agent's site 100% (no price scaling, so the "top price" is the price); Tunnel's site 75% like an expansion. **B** 75% for both.
6. **Sites:** **A (proposed)** as the two flows they follow: an Agent's site visible, a Tunnel's hidden. **B** both visible.
7. **The Agent's exit:** **A (proposed)** the first clear spot next to the footprint, searched in a fixed order starting from the side the Agent came from; if none is clear, it waits inside until one is. **B** always the bottom side, waiting if blocked.
8. **Targeting:** **A (proposed)** as a unit holding position: BaseAutoTargeting within Range, then the closest, ties by id; it keeps its target while the target is valid and in range. **B** the closest enemy every shot.
9. **The slot:** **A (proposed)** no: the Agent builds it, not the Tunnel. **B** yes, if inside the area: the Tunnel can't expand or upgrade meanwhile.
