# Ticket: Unit Control / Army Cap Systems

## Current State
No unit cap enforcement exists. Units can be produced without limit.

## Desired State
Implement four distinct unit cap systems, one per faction:

**GDO — Fixed Unit Control (200)**:
- Cap is always 200, no infrastructure required.
- Each unit has a UnitControl cost.
- Cannot build a unit if doing so would exceed 200.

**Syndicate — Tunnel Space (cap 200)**:
- Each Tunnel building provides TunnelSpace based on its upgrade level.
- Total TunnelSpace = sum across all Tunnels, capped at 200.
- Each unit has a TunnelSpace cost.
- Cannot build a unit if doing so would exceed available TunnelSpace.

**Cults — Territory-Based Unit Control (no hard cap)**:
- Each RecruitmentCenter provides UnitControl proportional to the number of Recruitable tiles it is actively recruiting from.
- Total UnitControl = sum across all RecruitmentCenters.
- No maximum cap — army size is bounded only by territorial control.
- Each unit has a UnitControl cost.
- Cannot build a unit if doing so would exceed available UnitControl.

**Colonists — Beacon Capacity (cap 200)**:
- Each Beacon provides BeaconCapacity.
- Total BeaconCapacity = sum across all Beacons, capped at 200.
- Each unit has a BeaconCapacity cost.
- Cannot build a unit if doing so would exceed available BeaconCapacity.

All systems must:
- Recalculate available capacity when the providing structure is built, destroyed, or upgraded.
- Prevent unit construction that would exceed the cap.
- Handle the case where a structure is destroyed and used > available (existing units remain but no new units can be built until used drops below available).

## Justification
Required by `features/factions_and_resources.md`. Army size management is a core strategic mechanic for all factions, with each faction's system creating distinct gameplay dynamics (GDO is static, Syndicate/Colonists must build infrastructure, Cults must control territory).

## QA Steps
1. **GDO**: Start with 0 units. Build units until UnitControl reaches 200. Attempt to build one more unit — verify it is blocked. Lose a unit, verify you can build again.
2. **Syndicate**: Start with no Tunnels. Verify TunnelSpace is 0 and unit production is blocked. Build a Tunnel. Verify TunnelSpace increases. Build units up to the new limit. Upgrade the Tunnel and verify TunnelSpace increases (up to 200 max). Destroy the Tunnel — verify existing units persist but new production is blocked.
3. **Cults**: Start with a RecruitmentCenter near Recruitable tiles. Verify UnitControl is proportional to the number of tiles. Build units up to the limit. Expand to more Recruitable tiles — verify UnitControl increases with no cap. Lose the RecruitmentCenter — verify existing units persist but new production is blocked.
4. **Colonists**: Start with no Beacons. Verify BeaconCapacity is 0 and unit production is blocked. Place a Beacon and verify capacity increases. Build units up to the cap. Verify total BeaconCapacity cannot exceed 200 even with more Beacons.
5. For all factions, verify the "used / available" values update correctly as units are built, destroyed, or cap-providing structures change.

## Expected Experience
Players see their faction's unit cap resource in the HUD (e.g., "Unit Control: 45/200" for GDO). When they try to queue a unit that would exceed the cap, the build command is rejected. When cap-providing structures are built or destroyed, the available value updates immediately. If available drops below used (structure destroyed), no new units can be built but existing ones continue to function.
