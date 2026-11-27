# 0446 — Vehicle buildings start with a default rally point 3 GU out of the exit

- **Date**: 2026-11-26T21:35:00-05:00
- **Answers**: Q0175
- **Docs**: `core/entities.md > Entrance and exit > Exit Doorway` (new point 7)

## What changed and why
The user chose option 4. Idle wheeled/tracked vehicles don't yield (0440), so with no rally point the first vehicle parked outside the exit stalled the building. Every building releasing wheeled or tracked vehicles through an exit side (Chop Shop, Factory) now **starts with a RallyPoint 3 GU straight out from the middle of its exit side** (placeholder). It's an ordinary rally point the player can move; it is set at placement, so it exists during construction. The user's alternative (idle vehicles rolling forward when pushed from directly behind) is noted but not adopted, to avoid relying on it.

## Implementation impact
New: default RallyPoint on Chop Shop and Factory at placement. Colonists portals (centre exits, hover vehicles that yield) and Syndicate Tunnels are unaffected. No "clear rally point" command exists.

## Verification
- Place a Chop Shop: selected, it shows a rally point 3 GU out of side C.
- Build three Seekers without touching the rally: each drives to the rally point area; the doorway never stalls.
- Move the rally point: new vehicles go there.
