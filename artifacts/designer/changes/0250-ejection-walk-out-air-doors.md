# 0250 — Ejection: ground units walk out single file; air doors have their own rate

- **Date**: 2026-11-25T11:40:00-05:00
- **Answers**: Q0064 points 1, 2, 4 (rate), 10, 11 (rate)
- **Docs**: `factions/syndicate/tunnels.md > EjectMenu`, `> Tunnel > Side A`

## What changed and why
User:
- **Ground exit mirrors entering** (0246): units walk out of the exit path single file at their normal speed and spacing. The exit rate comes from unit speed and size. This **replaces the "every 8 frames minimum"**.
- Each unit appears at the exit spot just outside Side A. If the spot is blocked by any unit or structure, the next unit waits; no other spot is tried.
- With no rally point, a unit walks about 1 GU straight out and idles.
- **Air units** use the Tier 3 air exit door, in parallel with the ground path. Since they don't collide they need an explicit rate: **one aircraft per 8 frames, placeholder**. The air entry door uses the same rate.

## Implementation impact
- The ground exit is a single-file walk-out, gated on the spot being clear.
- Air doors are throttled at 8 frames.
- Still open from Q0064: door positions (proposal: on the roof, over the matching path halves), which unit comes out (pt 3), click reservation and queue details (pt 4), rallies (pts 5–7), auto-eject (pt 8).

## Verification
- Ejecting 10 Guards: they stream out continuously.
- Two Phantoms and two Guards ejected together come out simultaneously through their separate exits, the Phantoms 0.5 s apart.
