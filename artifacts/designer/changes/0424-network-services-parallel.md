# 0424 — Network services run side by side; eject still takes the unit inside longest

- **Date**: 2026-11-26T18:25:00-05:00
- **Answers**: Q0158
- **Docs**: `factions/syndicate/structures/vault.md > Placeholder stats`; `factions/syndicate/structures/null_chamber.md > Placeholder stats`; `factions/syndicate/tunnel_menus.md > EjectMenu`

## What changed and why
- **1 (a)**: a unit's Vault turn, Null Chamber turn (and any repair) run **at the same time** in separate queues: a Specter is ready in 10 s, an empty Shadebringer in 20 s.
- **2 (a)**: the EjectMenu keeps taking the unit **inside the network longest**. User's reason: units may be in for repair too, which also takes time, and "the game doesn't know which [service] you're waiting for".

## Implementation impact
None: both match the placeholders.

## Verification
- An empty Shadebringer entering a network with a free Vault and Null Chamber can be ejected fully refilled and buffed after 20 s.
- Two Shadebringers, the first in empty, the second full: eject one: the first comes out, half-filled if ejected early.
