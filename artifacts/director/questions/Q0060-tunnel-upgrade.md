# Q0060 — Tunnel upgrade: duration, HP and payment

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: yes — slice M4-G, for the upgrade's duration (point 1). Nothing else waits for it.
- **Docs**: `factions/syndicate/tunnels.md > Tiers`, `> Tunnel ObjectInterfaceState`, `> Upgrade Costs`, `> Construction and Upgrade Rules`, `> Tunnel Area Non-Overlap Rule`, `> Losing Tunnels`.

## Question
1. **The upgrade's duration**, to Tier 2 and to Tier 3. Silent. **Proposal:** 480 frames (30 s) each, the same as construction.
2. **When the Supplies are paid.** **Proposal:** when the command is given. The full refund on Cancel Upgrade implies they were paid.
3. **"Number of T2+ Tunnels owned".** Does a Tunnel upgrading to Tier 2 count? **Proposal:** yes. Two upgrades started together then cost 2 and 4, rather than 2 each.
4. **HP at completion.** Max HP rises from 600 to 800 (or from 800 to 1000). **Proposal:** the current HP rises by the same 200, so damage taken stays.
5. **A Tunnel destroyed mid-upgrade.** **Proposal:** the Supplies are lost, as for a site.
6. **The target area during the upgrade.** An upgrade is blocked at its start if the larger area would overlap another Tunnel's. But a new Tunnel placed next to it during the upgrade could make the finished area overlap. **Proposal:** from the start of the upgrade, the target area counts against other placements.

## Context
In M4, the upgrade's only effects are HP, a larger area (more Tunnel Space) and a sink for Supplies: no Tier 2 expansion or vehicle exists yet. So slice M4-G comes late, in M4's fifth wave.

## Options
- **Point 1:** A, as proposed. B, durations you give.
- **Point 3:** A, upgrades in progress count, as proposed. B, only finished ones.
- **Point 4:**
  - A, keep the damage, as proposed;
  - B, heal in full;
  - C, keep the HP fraction.
- **Point 6:** A, reserve the target area, as proposed. B, let the upgrade fail, or wait, at completion.
