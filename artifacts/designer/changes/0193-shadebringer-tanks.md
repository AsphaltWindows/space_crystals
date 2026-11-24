# 0193 — Shadebringer charges are draining tanks

- **Date**: 2026-11-25T01:35:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Shadebringer > Charges and recharging`

## What changed and why
User: each ability's charge (Sludge and Cover, separate) is a **tank** that drains while its toggle is on. When a tank runs empty, its ability switches off and can't be switched back on until the tank holds some charge again. Tanks refill gradually inside the Tunnel Network while the player owns a Vault, partly used tanks included. Refines 0190.

## Implementation impact
Concept only. Tank size, drain rate, refill rate and any minimum charge to switch on are TBD. The InfoPanel will need two tank gauges (detail TBD).

## Verification
—
