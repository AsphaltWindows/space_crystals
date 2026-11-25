# Disciple

(Renamed from "Acolyte", user, 2026-11-26: a name designating a higher rank.)

The Cults' spell caster, trained from a single Recruit at the Cathedral. Ties into the Cults lore of figures who rise by consuming Space Crystals. Infantry — can be augmented at the Altar.

## Entity Type - Unit
## Faction - TheCults
## Trained at - Cathedral (consumes 1 Recruit; Unit Control cost 1)

## Energy
The Disciple has its own energy pool, which it spends on its spells. It regenerates naturally over time (values in Placeholder stats below). The Shrine's energy regeneration ability temporarily increases this rate.

## Spells

### Shroud
(Named by the user, 2026-11-26; briefly "Shared Burden".)
(user, 2026-11-26) A **channeling** spell: while the Disciple channels it, its energy **drains gradually**, and it creates a **cloud**.
- **Damage spreading**: every attack against a unit standing **under the cloud** has its damage **converted to area (AoE) damage**, spread across the **whole area of all clouds contiguously connected** to the cloud the attacked unit stands under. Each object under that combined cloud takes its share under the normal AoE rules (systems/combat.md > AoEDamage: share = damage × its overlap area / the area).
- **The cloud is centered on the Disciple** (user, 2026-11-26). The Disciple **can't move while channeling**.
- **It affects every ground object under it** (user, 2026-11-26): **ground units and buildings**, whoever owns them, **not air units**. **All anti-ground damage** landing on objects under the cloud, single-target and area alike, is **absorbed and redistributed** over the connected cloud (likely through a damage buffer; mechanism being designed).
- **Modifiers** (user, 2026-11-26): each receiver's share from a release is treated like any damage it receives: its own pre-armor reductions, armor, directional armor and minimum damage apply as always (systems/combat.md > DamageModifierOrder).
- **When the cloud changes** (user, 2026-11-26): if the channel ends or the connected cloud splits, the buffer is **released immediately** over the cloud as it was.
- **What ends the channel** (user, 2026-11-26, corrected): only the Disciple's **death**, a **cancel** command (Stop or any other order), or running **out of energy**. Taking damage does **not** end it.
- Contiguous clouds merge for this purpose: **all contiguously connected clouds share one damage buffer** (user, 2026-11-26), and it is spread over their whole combined area.
- **Attackers under the cloud do full damage** (user, 2026-11-26): an attack fired by a unit standing **under the cloud** is not absorbed; it hits normally. Designer's reading: under the same connected cloud as its target, so units fighting inside the cloud (e.g. melee) deal full damage to each other, while fire from outside is spread.
- **The buffer** (user, 2026-11-26): anti-ground damage aimed at objects under the cloud goes into the shared buffer instead of hitting its target. Every **0.5 s (8 frames), placeholder**, the buffer is released as **one area hit over the whole connected cloud** and empties. Each ground object under it takes its share by **overlap area** (systems/combat.md > AoEDamage), with armor applied once per release; the share falling on empty cloud area is **lost**. So sparse clouds absorb much of the damage, and crowded ones little.

Numbers: see Placeholder stats below.

**Placeholder stats** (designer, 2026-11-26, at the user's request: provisional numbers so the Director can implement; tune in the numbers pass):
## UnitBase - LightInfantry
## Silhouette - 24x24 space units (square)
## MaxHP - 40
## PointArmor - 0
## FullArmor - 0
## SightRange - 6 grid units
## Groupable - true
## UnitBaseAttributes[LightInfantry]: as the Militant (TurnRate 180°/frame, infinite acceleration, MaxSpeed 4 space units/frame, RuggedTerrainDefenseBonus 50%)
## TurretAttributes - None
## AttackAttributes - None (placeholder: no weapon; for Augmentation it is **armed**, i.e. it gets only the MaxHP and speed parts of increments. To confirm with the user.)
## Energy - MaxEnergy 200; starts at 50; regenerates 0.5 per second
## Shroud numbers - cloud radius **3 grid units** around the Disciple; drains **5 energy per second** (40 s from full); needs at least **10 energy** to start; hotkey **Q** (Q starts the channel; Stop or another order cancels it)
## Cost - Cathedral: 1 Recruit, 75 Space Crystals, 320 frames
## ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **Q: Shroud**
