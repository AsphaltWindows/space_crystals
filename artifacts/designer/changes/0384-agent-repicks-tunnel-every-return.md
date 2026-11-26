# 0384 — Agents choose their Tunnel at every return

- **Date**: 2026-11-27T11:00:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units/agent.md > Gathering` ("Which Tunnel", crystals)

## What changed and why
User: yes, Agents should re-pick like Recruits. At **every return**, an Agent delivers to the own operational Tunnel whose crystal hatch is nearest to it when it finishes mining. A tie goes to the earlier id (0375). This **replaces** "chosen at the Gather's first return and kept for that Gather", so a newly finished Tunnel is used at once.

## Implementation impact
- Agents re-evaluate their Tunnel at the start of each return instead of caching it for the Gather.
- No load balancing, as before.

## Verification
- Agents gather to Tunnel A. Finish Tunnel B nearer their patch.
- From their next return on, they deliver to B without a new order.
