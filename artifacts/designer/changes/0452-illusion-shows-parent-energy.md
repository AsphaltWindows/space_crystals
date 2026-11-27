# 0452 — An illusion shows its parent's energy, live

- **Date**: 2026-11-27T12:45:00-05:00
- **Answers**: Q0185
- **Docs**: `factions/cults/units/disciple.md > Spells > Mirage` (new "Energy shown" bullet)

## What changed and why
The user chose option (a). An illusion's energy bar and InfoPanel line always show the **parent Disciple's current energy**, read live. Illusions have no energy of their own. Energy therefore never gives an illusion away. When the parent channels Shroud its energy drains and the illusions' bars drain with it; the cloud is what gives the real one away (0448).

## Implementation impact
As the Director assumed: no new state, the number is read from the parent. Every viewer still sees Disciple energy, as before (M5-N4a).

## Verification
1. Cast Mirage. All five Disciples show the same energy.
2. Channel Shroud on the parent. All five bars drain together.
3. Stop the channel. All five regenerate together.
