# 0468 — Shadebringer: one shared tank for Sludge and Cover; Tech Gates clarified

- **Date**: 2026-11-28T09:50:00-05:00
- **Answers**: Q0190 (follow-up)
- **Docs**:
  - `factions/syndicate/units/shadebringer.md` (Both toggles, Trail rules, Charges and recharging, Placeholder stats)
  - `factions/syndicate/structures/vault.md > Throughput`

## What changed and why
- **One shared tank** (user). Sludge and Cover now draw from a single tank instead of two separate charges. The user's reasons: sludge and smoke are related, and sharing creates tension between using the two. If one ability is later gated by a tech, the gate applies to the ability, not to a tank of its own.
- **0467 corrected.** The user's remark about a tech referred to the generic **Tech Gates** sub-mechanic, which is now defined (core/tech_and_research.md > Tech Gates). It did not mean that starting full is gated. The "Tech gate: possible" mark on full tanks is removed. A new Shadebringer starts with a full tank.

## Implementation impact
- **Replaces two tanks of 100 with one tank of 200** (designer placeholder, keeping the old total):
  - it drains **10/s per toggle on**, so 20/s with both on;
  - it refills at **10/s** with a Vault, 20 s from empty as before;
  - an empty tank switches both toggles off.
- **UI:** one tank gauge instead of two.

## Verification
1. With only Sludge on, a full tank lasts 20 s. With both on, it lasts 10 s, and then both switch off.
2. Inside the network with a Vault, an empty tank refills in 20 s.
3. Switching on either toggle with any charge left works.
