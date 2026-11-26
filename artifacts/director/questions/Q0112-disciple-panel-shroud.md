# Q0112 — The Disciple's panel: Shroud's key, and casting with a group

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: (a) and (c): R casts Shroud, Q stays Move; the Disciple stays Groupable and every selected Disciple with 10 energy channels.
- **Docs**: `factions/cults/units/disciple.md > ObjectInterfaceState`, `> Placeholder stats`; `systems/controls/selection_and_hud.md > BasicCombatUnitInterfaceState`, `> SelectionGroup`; `core/entities.md > Groupable`; `factions/gdo/units/peacekeeper.md` (Hotkey: R); `factions/colonists/units/sabre_legionnaire.md` (R: Convert)

## Question
Two points of the Disciple's panel:
1. 0319 gives Shroud "hotkey **Q**", and the panel is "BasicCombatUnitInterfaceState, plus **Q: Shroud**". That template already has "[Q] Move". Which key should Shroud use?
2. The Disciple is "Groupable - true", but caster-type units are Ungroupable ("one cast should spend one unit's energy, not the whole group's"). Which applies to the Disciple?

## Context
- The units that add an ability to the combat template use R: the Peacekeeper's shield ("Hotkey: R ... placeholder") and the Legionnaires' "R: Convert". R is empty in BasicCombatUnitInterfaceState.
- Shroud has no target: it is a channel centred on the Disciple. With Groupable Disciples, one press starts every selected Disciple's cloud, and clouds that touch join ("all contiguously connected clouds share one damage buffer"). To keep some Disciples back, the player selects fewer.

## Options
Key:
- **(a) R: Shroud**, and Q stays Move, as for the Peacekeeper and the Legionnaires.
- **(b) Q: Shroud**, replacing Move on the Disciple's panel. Move is then by right-click only.

Grouping:
- **(c) Groupable, and all channel.** One press starts every selected Disciple that has 10 energy. A large cloud is quick to make.
- **(d) Ungroupable,** by the caster rule. One press, one Disciple, then Tab to the next. Precise, but a large cloud is slower to make.

Proposed: (a) and (c).
