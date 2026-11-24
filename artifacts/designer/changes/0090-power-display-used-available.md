# 0090 — Power display: used / available

- **Date**: 2026-11-24T03:35:00-05:00
- **Answers**: Q0025 (point 3; Q0025 is now fully answered)
- **Docs**: `factions/gdo/overview.md > DisplayHud`

## What changed and why
The user chose to show Power as **used / available**:
- **used** = total power consumed;
- **available** = total power generated.

The display reads **21/30** at the start and **51/30** in the all-in Barracks case. It is shown in **red** when used exceeds available, i.e. when power is negative.

The user likes that it matches the other capacity counters: GDO Unit Control, Syndicate Tunnel Space and Colonists Beacon Capacity.

This **replaces** the old definition in the doc ("current = generated − consumed"), which would have shown 9/30 and −21/30.

## Implementation impact
- HUD Power text: consumed / generated, turning red when consumed > generated.

## Verification
1. At the start of a GDO game, the HUD shows Power 21/30 (DC +30; EF −15 and 2 plates −6).
2. Build a Barracks before any Generator. The HUD shows 51/30 in red.
