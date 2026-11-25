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
- Contiguous clouds merge for this purpose: **all contiguously connected clouds share one damage buffer** (user, 2026-11-26), and it is spread over their whole combined area.

TBD: cloud size; what ends or interrupts the channel; whether damage spread over empty cloud area is lost or shared among the units under it; drain rate; name; hotkey.

, armor, SightRange, Groupable, movement attributes, AttackAttributes (if any), spells/abilities, and ObjectInterfaceState: TBD. Whether the Disciple counts as an "unarmed" unit for Augmentation increments is TBD.
