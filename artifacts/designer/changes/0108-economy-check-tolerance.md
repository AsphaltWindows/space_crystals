# 0108 — Economy reference check: tolerance, and benchmark status

- **Date**: 2026-11-24T12:20:00-05:00
- **Answers**: Q0038 (point 3; with 0106 and 0107, Q0038 is fully answered)
- **Docs**: `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)` (paragraphs "A benchmark, not a rule" and "Matching the table in a simulated check")

## What changed and why
Option A. On the reference layout, targets 1–4 must hold and each income figure must be within ±15% of the Economy Targets table. A larger gap comes back to the designer as a question with the measured table; the Director does not change game numbers to force a match.

The user stressed the reference layout is only a benchmark: maps and players place patches and Egress Nodes freely, and different incomes on other layouts are intended.

## Implementation impact
Test/oracle criteria only. Nothing in gameplay may enforce or assume the reference geometry.

## Verification
The Colonists oracle (K1) checks targets 1–4 and ±15% per table row on the reference layout (0107); failures are reported as a Director question with the measured table.
