# 0301 — No transit delay for a Tunnel's builder or for produced units rallied inside

- **Date**: 2026-11-26T12:20:00-05:00
- **Answers**: Q0087
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Network` (Transit delay)

## What changed and why
User: **(a)**, as built. The Agent that completes a Tunnel can be ejected at once. The user adds that produced units rallied into the network have no delay either; this was already true for produced units and is now stated explicitly.

## Implementation impact
None; as built.

## Verification
Finish a Tunnel and eject its builder immediately: it walks out with no delay.
