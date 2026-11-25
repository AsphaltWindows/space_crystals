# Q0079 — Agent build orders: an Agent killed before the start, and the area of a pending Tunnel ghost

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, marked as placeholders in M4-E.
- **Docs**: `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, `> Tunnel Area Non-Overlap Rule`; `factions/syndicate/units/agent.md > Building`; `core/entities.md > Placement Validation > Lingering build ghosts`

## Question
1. An Agent's Tunnel order was paid when given (0233). If the Agent is **killed** on its way, before construction starts, is the price refunded?
2. Does an Agent's **pending Tunnel ghost** reserve its whole Tier 1 area against its owner's further Tunnel placements, or only its footprint?
3. Does a Tunnel **site** (under construction) count for the non-overlap rule with its area?

## Context
- 0233: "If the order is cancelled, replaced, or fails before construction starts, it is refunded at the current top price". A killed Agent's order neither is cancelled nor fails at the site, so the text doesn't say.
- 0233 also counts lingering ghosts and sites toward the price, so they already act as Tunnels for cost.
- Without an area reservation, two Shift-queued ghosts can be placed with overlapping areas; the second then fails on arrival, after its Agent walked there.
- M4-E builds the proposals; each is one rule in the gate.

## Options
1. **Killed Agent:**
   - **A (proposed).** Refund at the top price, as for an order that fails. The books stay exact ("the Supplies paid for what the player still has always equal the formula's total").
   - **B.** The Supplies are lost, like a destroyed site. Losing an Agent on the way then costs a Supply too.
2. **Pending ghost's area:**
   - **A (proposed).** The ghost reserves its whole Tier 1 area against its owner's further Tunnel ghosts; the ghost shows red over it.
   - **B.** Only the footprint is reserved; the overlap is found on arrival, and that order fails with a refund.
3. **A site's area:**
   - **A (proposed).** It counts like any Tunnel's, for every player, from the moment the site appears.
   - **B.** Only operational Tunnels count. Two sites could then complete with overlapping areas, which breaks the invariant that areas are disjoint.
