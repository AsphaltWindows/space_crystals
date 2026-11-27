# 0450 — Disciples and their illusions don't fire from a Garrison

- **Date**: 2026-11-27T12:30:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/structures/garrison.md` (intro, Garrisoning > Who can enter, Weapons, Range bonus)
  - `factions/cults/units/disciple.md > Spells > Mirage` (Garrison bullet)
  - `systems/controls/commands.md` (the Cults Garrison line)
  - `factions/cults/structures/README.md` (Garrison row)

## What changed and why
The user decided that neither Disciples nor their illusions fire from inside a Garrison. Before, Disciples fired the Militant's attack from inside. 0448 had let illusions fire for real, which made them a free source of Garrison fire, since they use no Unit Control. Both can still **enter** a Garrison. Inside they are sheltered but don't attack.

## Implementation impact
- **Changed:** a Disciple inside a Garrison no longer attacks. This replaces "Recruits and Disciples fire as Militants". Recruits still fire as Militants.
- **Replaces part of 0448:** illusions inside a Garrison don't fire.
- **Threat scans:** a Garrison holding only Disciples and illusions has no occupant that can attack. Enemy target scans therefore don't treat it as a threat (`garrison.md > Seen as a threat`, Q0139). That is still the existing rule; this change only alters who counts as able to attack.
- No spells from inside, as before.

## Verification
1. Put two Disciples in a Garrison and bring an enemy into range. The Garrison doesn't fire.
2. Add a Recruit. The Garrison now fires the Militant's attack.
3. Put a Disciple's illusions inside instead. The Garrison doesn't fire.
