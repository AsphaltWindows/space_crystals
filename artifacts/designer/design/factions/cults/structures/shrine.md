# Shrine

Tier 3 of the Mystical/Human Enhancement branch (sibling of the Cathedral). Recruits gather inside the Shrine and take part in a ritual that charges it with energy. The Shrine spends that energy on area abilities that support Cults infantry.

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Temple

Size, armor and other stats: see Placeholder stats below. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Ritual Participation
Recruits enter the Shrine to take part in the ritual (right-click own Shrine with Recruits selected → Enter Shrine). Only Recruits can participate. Participating Recruits are not consumed. Evacuation on Destruction applies.

## Energy
- **MaxEnergy**: 200 (placeholder, below)
- **Energy regeneration**: proportional to the number of Recruits inside (0.25 per second per Recruit, placeholder). With no Recruits inside, the Shrine does not regenerate energy.

## Abilities
Both abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy.

The effect is a **buff applied to units at the moment of casting**: qualifying units inside the area when the ability is cast receive the buff for its duration and keep it even if they leave the area. Units that enter the area afterward do not receive it. Only the caster's own units are affected.

| Ability | Affects | Effect | Energy cost | Area | Duration |
|---|---|---|---|---|---|
| Health regeneration (name TBD) | Own Cults infantry | Grants health regeneration (+2 HP/s, placeholder) for the duration — Cults infantry have no natural regeneration | 75 | radius 4 grid units | 15 s |
| Energy regeneration (name TBD) | Own Disciples | Increases Disciple energy regeneration (+1/s, placeholder) for the duration | 50 | radius 4 grid units | 20 s |

Cooldowns: none. Energy costs, areas, durations and rates: see Placeholder stats below.

## ShrineInstanceState:
- ParticipatingRecruits: list of Recruit references (max 8, placeholder)
- Energy: number (0 to MaxEnergy)

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
- Size 2x2; SymmetryType AAAA; MaxHP 300; PointArmor 1, FullArmor 4; SightRange 4 grid units; Groupable **false** (user, 2026-11-26; Q0118: each press casts from one Shrine).
- Cost and time: Recruit > Constructs (200 SC, 480 frames).
- Recruit capacity 8. MaxEnergy 200; starts at 0; regeneration **0.25 energy per second per Recruit inside** (2 per second when full).
- **Health regeneration** (hotkey Q): 75 energy; circle of radius 4 grid units; **+2 HP per second for 15 seconds**.
- **Energy regeneration** (hotkey W): 50 energy; circle of radius 4 grid units; affected Disciples' energy regeneration is **+1 per second for 20 seconds**.
- **In the circle** (user, 2026-11-26; Q0118): a unit is affected when its **centre** is inside. **Recasting** on units that already have the effect **restarts its duration**; effects of the same ability **don't stack**.
- No cooldowns. Targeting: AwaitingTarget, left-click a point in vision. **A: Eject All** (participants leave).
