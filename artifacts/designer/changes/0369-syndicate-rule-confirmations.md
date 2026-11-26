# 0369 — Syndicate rule confirmations: Amplify marks the next attack, Shadebringer toggles on with any charge, Transport holds 4; Hornet dodgeable confirmed

- **Date**: 2026-11-27T03:15:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/syndicate/units/{laser_battery,shadebringer,subterranean_transport,specter}.md`;
  - `factions/syndicate/structures/{vault,null_chamber}.md`;
  - `factions/gdo/units/hornet.md`.

## What changed and why
User, on 0367's "to confirm" readings:
- **Hornet:** DoublyDisjointed (dodgeable), confirmed.
- **Amplify marks the next attack:** pressing it makes the next surge use the whole battery. It does not fire at once.
- **A Shadebringer toggle can be switched on with any charge left.** This replaces "at least 10".
- **Subterranean Transport capacity is 4 infantry** (4 heavy infantry, e.g. 4 Guards), replacing 8 Tunnel Space.

Not objected to, so they stand:
- the Vault serves 2 at a time;
- the Null Chamber buffs 4 at a time for 2 s each, the buff ends abruptly, and no research is needed;
- Amplify works mobile and deployed;
- the sludge slow refreshes while standing in it;
- toggles follow the mode-switch rule;
- the Transport loads by right-click and F unloads all;
- the Specter can be hit during lift-off and landing.

## Implementation impact
- Amplify is a flag on the next surge.
- Transport capacity is 4.

## Verification
- Press Amplify on a battery that is between surges: the next surge empties it in one big hit.
