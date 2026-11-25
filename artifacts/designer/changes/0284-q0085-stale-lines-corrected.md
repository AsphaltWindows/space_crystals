# 0284 — Stale lines corrected to the newer rules

- **Date**: 2026-11-26T08:30:00-05:00
- **Answers**: Q0085
- **Docs**: `factions/syndicate/units/agent.md` (Tunnel Construction Flow step 5, Cancel Construction, DefaultState letters S/D), `factions/syndicate/tunnels.md > Tunnel ObjectInterfaceState` (Cancel Upgrade), `factions/syndicate/structures/laser_turret.md > Upright vs Pop-up`, `core/entities.md > Building armor rule` (example), `factions/syndicate/structures/{hideout,optics_lab,chemical_plant}.md` (armor 6/24), `systems/combat.md > AoEDamage` (first sentence)

## What changed and why
Option **A**. All seven lines are corrected to the newer rules, with no design change:
- **Cancel refunds:** construction and upgrade cancels refund the current top price.
- **Agent panel letters:** S is Drop Off, D is Build Laser Turret.
- **Laser Turret:** its timing and armor now point to the placeholder stats.
- **Building armor example:** fixed.
- **Expansion stat blocks:** 6/24.
- **AoE:** the sentence says Ground AoE hits underground objects too.

## Implementation impact
None beyond what the slices already build.

## Verification
—
