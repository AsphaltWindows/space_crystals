# Designer Review: GDO Structure Interfaces & Guard Unit

## Metadata
- **Created by**: operator
- **Created**: 2026-03-19T10:03:00Z
- **Status**: open

## Close Votes

## Discussion

### [operator] 2026-03-19T10:03:00Z

The following items cover GDO Deployment Center cancel commands, Supply Tower interface completion, and the new Syndicate Guard unit. None have been implemented yet. **Designer**: please review and produce `feature_request` messages.

---

### 1. DC DefaultState Cancel Commands

DC DefaultState (DcIdle) should show a conditional Cancel (X) command at slot (2,1) when the DC has an active construction or a ready-to-place building. Currently the player must enter the BuildMenu to find Cancel -- this adds unnecessary friction.

**Behavior:**
- Cancel visible at (2,1) only when `current_construction.is_some()` or `ready_to_place.is_some()`
- During construction: full refund on cancel
- When ready-to-place: 75% refund on cancel (rounded down)
- Build (Q) remains at (0,0) -- both buttons visible simultaneously during construction
- The existing DcCancel handler already handles both cases with correct refunds -- no handler changes needed, just slot visibility

**Dependency:** Requires dc_ef_no_auto_enter_construction_submenu to be completed first. Currently `update_command_panel_state()` forces DcConstructing state whenever current_construction.is_some(), so the player can never stay in DcIdle while constructing.

**QA Steps:**
1. [auto] Select a DC that is not constructing -- verify only Build (Q) appears, no Cancel (X).
2. [auto] Start a construction and return to DefaultState -- verify Cancel (X) now appears alongside Build (Q).
3. [auto] Press X during construction from DefaultState -- verify full refund.
4. [auto] Let construction complete to ready-to-place, return to DefaultState -- verify Cancel (X) appears.
5. [auto] Press X when ready to place -- verify 75% refund.
6. [auto] Enter BuildMenu (Q) while constructing -- verify Cancel (X) is still available inside BuildMenu.
7. [auto] Verify the X slot position is consistent at (2,1) in both DefaultState and BuildMenu.

---

### 2. Supply Tower ObjectInterfaceState

The Supply Tower needs a complete ObjectInterfaceState with 4 commands:

**Command Layout:**
- **(0,0) Q: Build Supply Chopper** -- queues a Supply Chopper for production (100 SC, 160 frames). Max queue size 5.
- **(1,0) S: Schedule Deliveries** -- enters AwaitingTarget[ScheduleDeliveries]. Left-click an SDS (Supply Delivery Station) to assign it. Only available when the tower has an attached chopper.
- **(2,1) X: Cancel Production** -- cancels the last queued item with full refund. Only visible when queue is non-empty.
- **(2,2) C: Set Rally Point** -- enters AwaitingTarget[SetRallyPoint]. Left-click a ground location to set rally.

**Additional behaviors:**
- Right-click a ground location while Supply Tower is selected sets rally point directly (no need to press C first)
- Produced Supply Choppers should move to the rally point after spawning
- Schedule Deliveries targets SDS objects specifically; clicking non-SDS entities should be rejected

**QA Steps:**
1. [auto] Select a Supply Tower -- verify Q, X, C, S in correct grid positions
2. [human] Press Q with sufficient SC -- verify Supply Chopper queued and 100 SC deducted
3. [auto] Press Q when queue is full (5) -- verify rejected
4. [auto] Press X with non-empty queue -- verify last entry removed and fully refunded
5. [auto] Press X with empty queue -- verify nothing happens
6. [auto] Press C -- verify state changes to AwaitingTarget[SetRallyPoint]
7. [human] In AwaitingTarget[SetRallyPoint], left-click ground -- verify rally point set, state returns to DefaultState
8. [human] Right-click ground while Supply Tower selected -- verify rally point set directly
9. [auto] Press S with attached chopper -- verify state changes to AwaitingTarget[ScheduleDeliveries]
10. [human] In AwaitingTarget[ScheduleDeliveries], left-click an SDS -- verify deliveries scheduled, state returns to DefaultState
11. [auto] Press S without attached chopper -- verify command unavailable
12. [human] Produce a Supply Chopper with rally point set -- verify it moves to rally point after spawning

---

### 3. Guard Unit Implementation

New Syndicate combat unit. The Guard is a ranged infantry unit produced from the Headquarters.

**Stats:**
| Property | Value |
|---|---|
| Faction | TheSyndicate |
| UnitBase | HeavyInfantry |
| Silhouette | 36x36 space units |
| MaxHP | 80 |
| PointArmor | 1 |
| FullArmor | 1 |
| SightRange | 5 |
| TunnelSpaceCost | 2 |
| Groupable | true (unlike Agent) |
| Movement | TurnRate -- MaxSpeed 5 su/frame, Acceleration infinite, Deceleration infinite, TurnRate 180 deg/frame |
| Attack | FullyConnected Ranged, Ground, SingleTarget -- Damage 6, Range 3 GU, MinRange 0, Aim 2 frames, Fire 1 frame, Cooldown 1 frame, Reload 4 frames |
| ObjectInterfaceState | BasicCombatUnitInterfaceState (Move, Stop, Attack) |
| Production | From Headquarters, 100 SC, 160 frames |

**Key differences from Agent:**
- Groupable (Agents are ungroupable)
- Ranged attack at 3 GU (Agent is melee)
- HeavyInfantry base (Agent is LightInfantry)
- No resource gathering capability
- Uses standard BasicCombatUnitInterfaceState (Agent has custom AgentMenuState)

**Implementation needs:** ObjectEnum::SyndicateGuard variant, guard_type_data()/guard_attack_data() functions, spawn_syndicate_guard() function, integration with HQ production system.

**QA Steps:**
1. [auto] Verify SyndicateGuard ObjectEnum variant exists with correct properties (name "Guard", size 36x36, destructible true, sight_range 5, groupable true).
2. [auto] Verify guard_type_data() returns correct stats (HeavyInfantry, 80 HP, 1/1 armor).
3. [auto] Verify guard_attack_data() returns correct attack (FullyConnected Ranged, 6 damage, 3 range).
4. [auto] Spawn a Guard -- verify all expected components attached (movement, combat, tunnel space cost).
5. [auto] Select a Guard -- verify BasicCombatUnitInterfaceState commands (Move, Stop, Attack, HoldPosition).
6. [auto] Verify Guard is groupable: box-select two Guards, verify they form a single merged SelectionGroup.
7. [auto] Verify Guard attack range is 3 GU (ranged), not melee.
8. [human] Produce a Guard from Headquarters -- verify it emerges from parent Tunnel's Side A.
9. [human] Order the Guard to attack an enemy unit -- verify ranged attack at 3 GU distance.

---

### Key questions for the designer:
- DC Cancel at slot (2,1) -- is this the right position? Should it be consistent across all production structures?
- Supply Tower Schedule Deliveries -- should clicking a non-SDS entity provide feedback (error sound/message)?
- Guard unit stats: is 6 damage at 3 GU range with the given attack timings balanced?
- Should the Guard have any special abilities beyond basic combat, or is pure ranged combat the intent?
- Guard is Groupable (unlike Agent) -- confirm this is intentional? Guards can be merged into SelectionGroups while Agents cannot?
