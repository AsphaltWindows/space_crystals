# 0339 — Cults group construction: closest footprint part and wall-hug, first arrival starts it, own Recruits block, whole order fails, Recruit-frames

- **Date**: 2026-11-26T21:20:00-05:00
- **Answers**: Q0097
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics > Group construction rules` (new)

## What changed and why
User, point by point:
1. Each Recruit goes to the **closest part of the footprint**. If that part is blocked, it **wall-hugs** around the footprint until it finds a free part, and works from there within reach (8 su).
2. The site starts **when the first Recruit arrives**.
3. Recruits of the order standing on the footprint **block it**.
4. A blocked site makes **the whole order fail**, refunded once. The user notes this is a special case.
5–8. A building needs a fixed number of **Recruit-frames**. Each frame, every Recruit **actively building** adds 1. Those walking or waiting add nothing, and there is no other cap.
6. Builders **keep building** under attack.
7. **Assist** targets only buildings already under construction.

## Implementation impact
- Spot search changes from a fixed order to closest point plus wall-hug.
- Own Recruits on the footprint fail the order.
- Construction progress accumulates per active builder per frame.

## Verification
- Five Recruits from one side: the first builds at once, and the others spread along the wall.
- With two builders the progress rate doubles while both work.
- A Recruit of the order standing on the ghost at the first arrival fails the order with a full refund.
