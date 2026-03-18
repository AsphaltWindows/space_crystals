# Ticket: Rally Point Behavior for Production Expansions

## Current State
Production expansions (Headquarters) do not have rally point behavior. When a unit finishes production, there is no system to determine whether it should auto-eject from the parent Tunnel or remain in the Tunnel Network.

## Desired State
Implement rally point behavior for production expansions:

### Rules
- **Rally point set on the surface**: When a unit finishes production, it auto-ejects from the parent Tunnel's Side A and moves to the rally point location.
- **No rally point set**: Unit stays in the Tunnel Network, available for ejection from any sufficiently-tiered Tunnel.
- **Rally point set on the parent Tunnel**: Same as no rally point — unit stays in the Tunnel Network.

### Implementation Requirements
- Each production expansion needs a rally point field (target position or entity)
- Right-clicking a surface location while a production expansion is selected should set the rally point
- Right-clicking the parent Tunnel (or no right-click target) should clear/reset the rally point to "stay in network"
- The production tick system must check the rally point after spawning a unit and either:
  - Eject the unit from Side A and issue a move command to the rally point, OR
  - Place the unit in the Tunnel Network without ejecting
- A visual indicator (rally point flag/marker) should appear at the rally point location when set

## Justification
`features/syndicate_objects.md` — Rally Point Behavior section. This is a core Syndicate production mechanic that differentiates it from GDO. Without rally points, every produced unit would need manual ejection, making Syndicate production tedious. The surface vs. Tunnel rally point distinction is a meaningful strategic choice: surface rally points for immediate deployment, Tunnel rally points for staging forces underground.

## QA Steps
1. [human] Select a Headquarters and right-click a surface location — verify a rally point marker appears at that location
2. [human] Produce a unit with the surface rally point set — verify the unit auto-ejects from the parent Tunnel's Side A and moves to the rally point
3. [human] Right-click the parent Tunnel while the Headquarters is selected — verify the rally point is cleared (no marker visible)
4. [human] Produce a unit with no rally point set — verify the unit stays in the Tunnel Network (does not eject, appears in Eject menu)
5. [human] Set a rally point, then right-click a new surface location — verify the rally point moves to the new location
6. [human] Produce multiple units with a surface rally point — verify each auto-ejects sequentially from Side A

## Expected Experience
Setting a rally point works like standard RTS production rally points: right-click a location, see a visual marker, and produced units automatically move there. The Syndicate twist is that units emerge from the Tunnel entrance (not the building itself, since it's underground). When no rally point is set, units silently enter the Tunnel Network — the player manages them later via the Eject menu. The system feels intuitive to RTS veterans while supporting the Syndicate's underground staging gameplay.
