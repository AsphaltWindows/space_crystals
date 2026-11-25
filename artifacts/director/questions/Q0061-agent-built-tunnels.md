# Q0061 — Agent-built Tunnels: payment, ownership and the site

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slice M4-E builds.
- **Docs**: `factions/syndicate/units.md > Agent > Building`, `> Tunnel Construction Flow`, `> Cancel Construction`; `factions/syndicate/tunnels.md > Cost Scaling`, `> Losing Tunnels`; `core/entities.md > ConstructionHP Rule`, `> Worker-Built Structures`; `core/game_rules.md > Defeat Condition`.

## Question
1. **When are the Supplies paid?** Silent; the flow mentions only "any Supplies spent" and a 100% refund. **Proposal:** when construction starts, on arrival, at the price then. The order is refused if the player can't pay the current price when giving it.
2. **"Current Tunnels owned".** Does a Tunnel under construction count? **Proposal:** yes. Two Agents building at once pay 1 and 2.
3. **What a Tunnel under construction does.** It "does not count as an exit". **Proposal:** it also gives no Tunnel Space, takes no drop-off and no Enter, and offers neither Expand nor Eject: only X Cancel Construction. This is the rule for a Colonists building in transformation ("While transforming: only Cancel").
4. **ConstructionHP**: "HP during construction = MaxHP x (10% + 90% x construction_progress)". Read literally, this would heal damage as construction goes on. **Proposal:** the formula gives the maximum, and damage stays taken: the current HP rises by the same amount each frame. This is the Warp In rule (`factions/colonists/economy.md > Warp In > HP/Shields Interpolation`).
5. **Where the Agent "emerges at the location"**, on a cancel or a destruction. **Proposal:** at the centre of the footprint, which is now free. It idles.
6. **The price can't be paid on arrival** (under point 1). **Proposal:** the order fails like an invalid site: it is cancelled, the Agent stops and idles, and the feedback line says why.
7. **Defeat.** **Proposal:** a Tunnel under construction counts as a structure. Cancelling it when it is the player's only structure defeats them in that tick, as for a Beacon under deployment.
8. **An Agent carrying a load when ordered to build.** **Proposal:** it keeps the load, in the site and in the network.
9. **What enemies see.** **Proposal:** the site, as a surface structure, but not the Agent inside it.

## Context
The flow says what happens to the Agent and the Supplies in each outcome, but not when the price is taken, nor what a site does before it is finished. Point 1 matters for the cost formula. If the price were paid with the order, several orders given at once could all pay the same low price, unless orders still on their way also counted.

## Options
- **Point 1:**
  - A, at construction start, as proposed;
  - B, with the order, at a price that counts pending orders, refunded in full if the order is cancelled or fails before construction starts. This is how the Cults' Build Command pays.
- **Point 2:** A, yes, as proposed. B, only operational Tunnels count.
- **Point 4:** A, damage stays, as proposed. B, the formula as written: the current HP follows the progress.
- **Other points:** the proposal, or a rule you give.
