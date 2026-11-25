# 0290 — Expansion placement: units don't block; Tunnel ghosts can't conflict; Warp In judged like direct placement

- **Date**: 2026-11-26T09:50:00-05:00
- **Answers**: Q0080
- **Docs**: `core/entities.md > Placement Validation > Direct Placement` (exception); `factions/syndicate/tunnels.md > AwaitingPlacement (Expansion)`; `factions/syndicate/structures/README.md > Tunnel Expansions` (Warp In)

## What changed and why
User:
1. **B:** ground units don't block an expansion's placement; it is underground, and units walk over expansions anyway. This is an exception to Direct Placement's "no ground units on the footprint".
2. **Moot.** A Tunnel ghost can't be placed where another Tunnel could build an expansion: its area would overlap that Tunnel's area. Expansions only go inside their own Tunnel's area, so the two never meet.
3. **A:** a Colonists Warp In over a hidden enemy expansion is judged like direct placement: red at once.

## Implementation impact
- `Method::Expansion` ignores ground units.
- No ghost check is needed for point 2.
- Warp In uses the direct-placement judgement.

## Verification
- Place a Hideout under a standing Guard: it is accepted.
- A Beacon's Warp In ghost over a hidden enemy expansion is red.
