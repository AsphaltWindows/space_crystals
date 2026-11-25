# 0252 — Syndicate rally points: on a Tunnel stays inside, on an expansion means the spot above it

- **Date**: 2026-11-25T12:00:00-05:00
- **Answers**: Q0064 points 5, 6, 7 (pt 8 pending)
- **Docs**: `factions/syndicate/structures.md > Rally Point Behavior`

## What changed and why
User:
- **Pt 5:** a rally on any own Tunnel keeps the unit inside the network.
- **Pt 6:** a rally on an expansion means the surface spot above it. The unit auto-ejects from the parent Tunnel and walks there (this replaces the Director's "stays in the network").
- **Pt 7:** "moves to the rally point" is the default right-click action at the target (0136).

## Implementation impact
Rally resolution for expansions is a surface location, not an object target.

## Verification
- Rally a Hideout on another Hideout. The produced Guard comes out of the parent Tunnel and stands over the other Hideout.
- Rally on a Tunnel: the Guard stays inside.
