# 0321 — Repair: Recruits repair buildings and vehicles; Agents repair Tunnels (free) and Laser Turrets; repairers stack linearly

- **Date**: 2026-11-26T16:40:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/recruit.md > Repair` (new); `factions/syndicate/units/agent.md > Repair` (new); `factions/gdo/units/field_engineer.md > Abilities` (Repair)

## What changed and why
User:
- **Recruits** can repair own buildings and vehicles, paying **Space Crystals**.
- **Agents** can repair own Tunnels and Laser Turrets. Turret repair costs crystals; **Tunnel repair is free**.
- **Several repairers scale linearly**, as construction does. This applies to all repairers, the GDO Field Engineer included.

Placeholders (designer):
- repairing from 0 to full HP takes half the target's build time with one repairer;
- it costs half the target's price in the resource it was bought with, paid gradually.

## Implementation impact
- A new Repair command for Recruits and Agents.
- The Field Engineer's repair gets numbers.
- The Recruit's panel letter is D, pending Q0095.

## Verification
- Two Recruits repair a damaged Armory twice as fast as one, and crystals drop as HP rises.
- An Agent repairing a Tunnel spends nothing.
