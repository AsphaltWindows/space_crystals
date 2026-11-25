# 0249 — Inside the Tunnel Network: orders cleared, groups kept, loads kept, slow regeneration; upgrading Tunnels keep their old tier

- **Date**: 2026-11-25T11:25:00-05:00
- **Answers**: Q0063 points 2–6 (Q0063 complete: 0245–0249)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Network` (Regeneration inside, Walking in)

## What changed and why
User:
- **Pt 3:** entering clears the unit's orders, queued ones included.
- **Pt 4:** units stay in their control groups.
- **Pt 5:** Agents keep their load.
- **Pt 6, new: all units inside the network slowly regenerate HP.** The Syndicate otherwise has no healing. The rate is a placeholder, **1% of MaxHP per second**. My assumption: passengers inside a Subterranean Transport don't regenerate, but the transport itself does.
- **Pt 2:**
  - Until an upgrade has fully finished, a Tunnel only accepts **and lets out** units supported by its tier from before the upgrade began.
  - If the target Tunnel is destroyed or unsuitable, the Enter order ends.

## Implementation impact
- Add HP regeneration for units in the network (fractional HP accumulation as needed).
- Exit and entry tier checks use the pre-upgrade tier until completion.

## Verification
- A Guard at 50% HP enters a Tunnel. After 10 s inside, ejected, it shows about 60%.
- A Tunnel upgrading from Tier 1 to Tier 2 won't let out a vehicle until the upgrade completes.
