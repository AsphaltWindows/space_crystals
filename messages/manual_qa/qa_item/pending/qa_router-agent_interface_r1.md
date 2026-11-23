# agent_interface_r1

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

**Note from automatic_qa**: This item was routed for automated QA but cannot be verified automatically because all QA steps require interactive gameplay — selecting units, pressing hotkeys (A for Build Tunnel, B for Drop Off), verifying ghost preview overlays, clicking map locations, right-clicking crystal fields and enemies, and observing real-time unit behavior. No cargo tests exist for these interactive scenarios. Returning for manual QA.

## Content

The Agent's ObjectInterfaceState panel shows correctly (Build Tunnel on A, Drop Off Resources on B) and basic movement works (right-click ground, multi-Agent move). However, most contextual commands and the tunnel placement system are broken.

**What passed:**
- Command panel shows Build Tunnel (A) and Drop Off Resources (B)
- Right-click ground issues Move command
- Multi-select Agents right-click ground moves all Agents
- Tab/Shift-Tab cycling between ungroupable Agents works

**What failed:**
1. **Build Tunnel placement broken**: Pressing A does not show a ghost Tunnel preview on the cursor. No placement overlay appears and construction cannot be started.
2. **Right-click Gather broken**: Right-clicking a crystal field does not issue a Gather command — nothing happens.
3. **Right-click Attack broken**: Right-clicking an enemy unit/building does not issue an Attack command — the Agent just stops. (Known issue affecting all units, not Agent-specific.)

**Blocked by failures above:**
- Drop Off Resources (B) — cannot test without first gathering resources
- Right-click own Tunnel while carrying/not carrying resources — depends on gathering and tunnel construction

## QA Instructions

1. Select an Agent and press A (Build Tunnel) — verify a ghost Tunnel preview appears on cursor, snapped to grid, with green/red tinting for valid/invalid placement.
2. Place a Tunnel (left-click valid location) — verify the Agent walks to the site and begins construction.
3. Right-click a crystal field — verify the Agent walks to it and begins gathering crystals.
4. With resources gathered, press B (Drop Off Resources) and click an own Tunnel — verify the Agent walks to the correct side (Side B for crystals).
5. Right-click an own Tunnel while carrying resources — verify Drop Off is issued.
6. Right-click an own Tunnel while NOT carrying resources — verify Enter is issued.
