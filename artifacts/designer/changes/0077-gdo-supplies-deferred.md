# 0077 — GDO supply loop deferred; Starting Supplies 0

- **Date**: 2026-11-24T01:35:00-05:00
- **Answers**: Q0022
- **Docs**: `factions/gdo/economy.md > Starting Conditions` (Starting Supplies TBD → 0, placeholder)

## What changed and why
The user picked **A**: the GDO supply loop (Supply Tower, Supply Chopper, deliveries) is deferred to the air milestone. GDO Starting Supplies are **0** (placeholder) until GDO Supply costs exist.

The missing design pieces will be filled in during the GDO numbers pass:
- what GDO spends Supplies on;
- the Radar Station's stats;
- the Supply Tower's cost and time;
- the Chopper's cargo.

**Standing instruction from the user:** when the Director is blocked only by missing stats, the Designer may put in reasonable **placeholder** values, clearly marked as placeholders, rather than wait for the user. If missing numbers block you, say so in the question, and the Designer will fill them in.

## Implementation impact
- M2: the HUD shows Supplies 0, and Supply Delivery Stations work as in M1 (0057). Nothing GDO has costs Supplies.
- Nothing about the Tower, Chopper or Radar Station changes. They are simply not built yet.

## Verification
Start a GDO skirmish. Supplies show 0 and nothing in GDO's menus asks for Supplies.
