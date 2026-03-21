# tunnel-network-mechanics

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Tunnel Network and Tunnel core mechanics as defined in `artifacts/designer/design/syndicate_objects.md`.

**NOTE: The Tunnel's ObjectInterfaceState was already sent as a prior feature request (designer-tunnel-interface). This request covers the core MECHANICS: tiers, tunnel area, transit rules, cost scaling, and construction.**

**Tunnel Network:**
Collective of all Tunnels owned by a Syndicate player. Units inside can travel between Tunnels freely but can only enter/exit through a Tunnel whose tier is sufficient.

**Transit Tier Requirements:**
- Tier 1+: Infantry (Heavy Infantry)
- Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)
- Tier 3+: Air units (Hover Craft, Glider)

**Tunnel Structure:**
- Size: 4x4, SymmetryType: ABCD
- Side A: Unit entrance/exit
- Side B: Crystal drop-off (Agents deliver Space Crystals)
- Side C: Supply drop-off (Agents deliver Supplies)
- Side D: Back wall (no function)
- Only one Agent may drop off at a side at a time. Crystal and Supply drop-offs on separate sides allow simultaneous delivery.
- Destructible: true, Groupable: false, SightRange: 5

**Tier Table:**
| | Tier 1 | Tier 2 | Tier 3 |
|---|---|---|---|
| HP | 600 | 800 | 1000 |
| PointArmor | 1 | 1 | 1 |
| FullArmor | 16 | 16 | 16 |
| Tunnel Space | 20 | 30 | 40 |
| Tunnel Area Radius | 3 (10x10) | 4 (12x12) | 5 (14x14) |
| Transit | Infantry | +Vehicles | +Air |
| Buildings | T1 | +T2 | +T3 |

**Tunnel Area:**
Square underground build zone centered on the Tunnel. Radius extends from 4x4 footprint, forming (radius+4+radius) per side.
- Underground expansions placed spatially within the area
- **Non-Overlap Rule:** Tunnel Areas cannot overlap. Uses current tier's area (not max potential). Upgrading blocked if expansion would overlap another Tunnel's current area. Players can place T1 Tunnels closer for early aggression but sacrifice upgrade potential.

**Construction/Upgrade Rules:**
One operation at a time: constructing expansion OR upgrading. Cannot do both.

**Cost Scaling (in Supplies):**
- Construction: cost = current Tunnels owned (1st=0 free start, 2nd=1, 3rd=2, etc.)
- Upgrade to T2: 2 + 2 x (T2+ Tunnels owned). 1st=2, 2nd=4, 3rd=6.
- Upgrade to T3: 3 + 3 x (T3 Tunnels owned). 1st=3, 2nd=6, 3rd=9.

**Construction Time:** 480 frames (30 seconds). Agent must be present for duration.

## QA Instructions

1. Start as Syndicate — verify player begins with 1 Tier 1 Tunnel (600 HP, 20 Tunnel Space).
2. Verify Side A functions as unit entry/exit point.
3. Verify Sides B and C function as separate resource drop-off points.
4. Verify only Infantry units can enter/exit a T1 Tunnel. Vehicles and Air should be blocked.
5. Upgrade to T2 — verify HP increases to 800, Tunnel Space to 30, area expands to 12x12. Vehicles can now transit.
6. Upgrade to T3 — verify HP 1000, Space 40, area 14x14. Air units can now transit.
7. Build a 2nd Tunnel — verify cost is 1 Supply. Build a 3rd — verify cost is 2 Supplies.
8. Verify Tunnel Area non-overlap: place two T1 Tunnels close together. Attempt to upgrade one — verify upgrade blocked if expanded area would overlap.
9. Verify only one operation at a time: start an expansion construction, then verify upgrade button is unavailable (and vice versa).
10. Verify construction takes 30 seconds with Agent present.
