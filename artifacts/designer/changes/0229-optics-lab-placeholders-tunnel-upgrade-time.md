# 0229 — Placeholders: Optics Lab stats; Tunnel upgrade duration and payment

- **Date**: 2026-11-25T07:55:00-05:00
- **Answers**: Q0059 (both points), Q0060 points 1 and 2 (points 3–6 still open)
- **Docs**: `factions/syndicate/structures.md > Optics Lab`; `factions/syndicate/tunnels.md > Construction and Upgrade Rules > Upgrade Costs`

## What changed and why
These questions block the Director only on missing numbers, so I've filled them with placeholders under the standing permission (user, 2026-11-24).
- **Optics Lab** (the Director's proposal, accepted as placeholders): 2x2, 150 SC (no Supplies), 400 frames, HP 400, armor 1/4, Groupable true. No commands for now (laser research TBD).
- **Tunnel upgrade**: 480 frames (30 s) to each tier, placeholder. The Supplies are paid when the upgrade is ordered (already implied by the full refund on Cancel Upgrade).

## Implementation impact
- Unblocks M4-K (Guard gate) and M4-G (upgrade duration).
- Q0060 points 3–6 (in-progress upgrades counted in cost, HP at completion, destroyed mid-upgrade, target-area reservation) go to the user; proceed on the proposals meanwhile.

## Verification
- Building an Optics Lab takes 25 s and then enables Guards at the Hideout.
- A Tunnel upgrade takes 30 s.
