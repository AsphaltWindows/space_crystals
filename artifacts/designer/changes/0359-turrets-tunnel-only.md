# 0359 — Only Tunnels build Laser Turrets; the Agent's panel reverts to A: Build Tunnel

- **Date**: 2026-11-27T00:30:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/syndicate/structures/laser_turret.md` (intro, Constructed by, Upright vs Pop-up, Construction, hotkey);
  - `factions/syndicate/units/agent.md > Agent ObjectInterfaceState`;
  - `factions/syndicate/structures/README.md` (table; Surface Defensive Structures).

## What changed and why
User, after weighing whether Agents should build all expansions (decided **no**: the Tunnel keeps building them, since its build time gates tech):
- **Agents no longer build Laser Turrets.** Only Tunnels build them, from the ExpandMenu inside their Tunnel Area, so every turret starts as a pop-up. They are still unlocked by owning an Optics Lab.
- **The Agent's panel reverts to A: Build Tunnel.** This undoes 0326's Construct menu. The panel is now Q Move, A Build Tunnel, S Drop Off, D Repair, X Stop.
- **Removed:** every rule for Agent-built turrets (Q0084: paying when ordered, Agent exit, no slot use, visible site). Q0091's "Agent's site takes Tunnel Space at the start" is moot.

A follow-up decision on what happens to a turret when its Tunnel is lost is pending with the user. The user says turrets should only be active inside a Tunnel Area. This entry does not yet change the Conversion rule.

## Implementation impact
- Remove turret placement from the Agent.
- The ExpandMenu turret stays.
- Agent repair of turrets stays.

## Verification
- The Agent panel shows A: Build Tunnel with no turret option.
- Turrets can be ordered only from a Tunnel's ExpandMenu.
