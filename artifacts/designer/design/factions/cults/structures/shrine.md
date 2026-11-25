# Shrine

Tier 3 of the Mystical/Human Enhancement branch (sibling of the Cathedral). Recruits gather inside the Shrine and take part in a ritual that charges it with energy. The Shrine spends that energy on area abilities that support Cults infantry.

## Faction - TheCults
## Constructed by - Recruit
## TechPrerequisite - player owns at least one Temple

Size, SymmetryType, MaxHP, armor, SightRange, Groupable, Recruit capacity, and ObjectInterfaceState: TBD. Space Crystals cost and construction time are listed under the Recruit's Constructs list (see Recruit > Building).

## Ritual Participation
Recruits enter the Shrine to take part in the ritual (right-click own Shrine with Recruits selected → Enter Shrine). Only Recruits can participate. Participating Recruits are not consumed. Evacuation on Destruction applies.

## Energy
- **MaxEnergy**: TBD
- **Energy regeneration**: proportional to the number of Recruits inside (rate per Recruit TBD). With no Recruits inside, the Shrine does not regenerate energy.

## Abilities
Both abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy.

The effect is a **buff applied to units at the moment of casting**: qualifying units inside the area when the ability is cast receive the buff for its duration and keep it even if they leave the area. Units that enter the area afterward do not receive it. Only the caster's own units are affected.

| Ability | Affects | Effect | Energy cost | Area | Duration |
|---|---|---|---|---|---|
| Health regeneration (name TBD) | Own Cults infantry | Grants health regeneration (rate TBD) for the duration — Cults infantry have no natural regeneration | TBD | TBD | TBD |
| Energy regeneration (name TBD) | Own Acolytes | Increases Acolyte energy regeneration (amount TBD) for the duration | TBD | TBD | TBD |

Cooldowns (if any): TBD.

## ShrineInstanceState:
- ParticipatingRecruits: list of Recruit references (max TBD)
- Energy: number (0 to MaxEnergy)
