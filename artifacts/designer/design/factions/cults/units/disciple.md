# Disciple

(Renamed from "Acolyte", user, 2026-11-26: a name designating a higher rank.)

The Cults' spell caster, trained from a single Recruit at the Cathedral. Ties into the Cults lore of figures who rise by consuming Space Crystals. Infantry — can be augmented at the Altar.

## Entity Type - Unit
## Faction - TheCults
## Trained at - Cathedral (consumes 1 Recruit; Unit Control cost 1)

## Energy
The Disciple has its own energy pool, which it spends on its spells. It regenerates naturally over time (MaxEnergy and regeneration rate TBD). The Shrine's energy regeneration ability temporarily increases this rate.

## Spells

### Cloud spell (name TBD)
(user, 2026-11-26) A **channeling** spell: while the Disciple channels it, its energy **drains gradually**, and it creates a **cloud**.
- **Damage spreading**: every attack against a unit standing **under the cloud** has its damage **converted to area (AoE) damage**, spread across the **whole area of all clouds contiguously connected** to the cloud the attacked unit stands under. Each object under that combined cloud takes its share under the normal AoE rules (systems/combat.md > AoEDamage: share = damage × its overlap area / the area).
- **The cloud is centered on the Disciple** (user, 2026-11-26). The Disciple **can't move while channeling**.
- **It affects every ground object under it** (user, 2026-11-26): **ground units and buildings**, whoever owns them, **not air units**. **All anti-ground damage** landing on objects under the cloud, single-target and area alike, is **absorbed and redistributed** over the connected cloud (likely through a damage buffer; mechanism being designed).
- **Modifiers** (user, 2026-11-26): each receiver's share from a release is treated like any damage it receives: its own pre-armor reductions, armor, directional armor and minimum damage apply as always (systems/combat.md > DamageModifierOrder).
- **When the cloud changes** (user, 2026-11-26): if the channel ends or the connected cloud splits, the buffer is **released immediately** over the cloud as it was.
- **What ends the channel** (user, 2026-11-26): running out of energy, the player ordering Stop or any other order, the Disciple **taking damage**, or its death.
- Contiguous clouds merge for this purpose: **all contiguously connected clouds share one damage buffer** (user, 2026-11-26), and it is spread over their whole combined area.
- **Attackers under the cloud do full damage** (user, 2026-11-26): an attack fired by a unit standing **under the cloud** is not absorbed; it hits normally. Designer's reading: under the same connected cloud as its target, so units fighting inside the cloud (e.g. melee) deal full damage to each other, while fire from outside is spread.
- **The buffer** (user, 2026-11-26): anti-ground damage aimed at objects under the cloud goes into the shared buffer instead of hitting its target. Every **0.5 s (8 frames), placeholder**, the buffer is released as **one area hit over the whole connected cloud** and empties. Each ground object under it takes its share by **overlap area** (systems/combat.md > AoEDamage), with armor applied once per release; the share falling on empty cloud area is **lost**. So sparse clouds absorb much of the damage, and crowded ones little.

TBD: cloud size and drain rate (numbers pass); name; hotkey.

, armor, SightRange, Groupable, movement attributes, AttackAttributes (if any), spells/abilities, and ObjectInterfaceState: TBD. Whether the Disciple counts as an "unarmed" unit for Augmentation increments is TBD.
