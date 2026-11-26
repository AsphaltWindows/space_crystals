# Designer Session Log

## 2026-08-06T02:27:30Z — Interactive: Colonists faction deep dive (Beacon, Warp In, Egress Node, Roboport)

**Trigger**: User session, no open forum topics or pending inbox messages at start (verified via forum pass before wrap-up).
**Loose end picked up**: Colonists faction (previously mostly open — only Prospector was fully defined).

**Work done**:
1. Confirmed with user: scrap the old "minimum Beacon spacing radius" note from `to_be_converted.md` (already correctly absent in `colonists_objects.md`); Warp In buildings only need to fit (normal footprint validation), no special radius rule.
2. Fully stated the Beacon (Size 2x2, MaxHP 120, MaxShields 10, PointArmor 0, FullArmor 0, SightRange 4, Groupable true, BeaconCapacity 5, SymmetryType AAAA) across several rounds of Q&A.
3. Renamed the Colonists worker unit Prospector -> Drone throughout `colonists_objects.md` and `factions.md`. Also edited two still-pending feature_requests in `messages/task_splitter/feature_request/pending/` in place (`colonists_prospector_full_definition.md`, `colonists_shields_and_prospector_beacon_deployment.md`) to use the new name, with an explanatory note, since they hadn't been picked up by task_splitter yet.
4. Formalized the Warp In mechanic: Beacon selected -> Warp In -> pick building from Constructs list -> footprint ghost (rotate/flip) centered on Beacon, must be clear except the Beacon's own tile -> confirm consumes the Beacon and begins an in-place transformation with HP/Shields interpolation over a per-building WarpInTime -> vulnerable throughout, destroyed if killed mid-transformation, cancellable for 75% refund. Every Colonists building provides a flat 5 BeaconCapacity. Added a new "Warp In" category to `entities.md`'s Placement Validation section.
5. Fully stated the Egress Node (Size 2x2, MaxHP 200, MaxShields 10, PointArmor 0, FullArmor 0, SightRange 3, Groupable false, SymmetryType AAAA, multi-Drone simultaneous drop-off, built via Warp In).
6. Added the AssignedEgressNode mechanic to Drone (auto-assigns nearest Egress Node on first Gather order; new "Set Drop-off Location" command, panel button S, AwaitingTarget flow, redirects return trip without interrupting current mining/movement) and a new "Select Returning Drones" command on Egress Node (panel button F, StateOnlyTransition that only modifies ControlState.Selection — first documented example of a pure-selection command).
7. Defined the Roboport (Colonists' Drone production structure: Size 4x4, MaxHP 600, MaxShields 20, PointArmor 1, FullArmor 16, SightRange 4, Groupable true, SymmetryType AAAA with no fixed exit side, standard BuildQueue/RallyPoint pattern like Barracks, produces Drone for 50 Space Crystals / 160 frames).
8. Defined Colonists Starting Conditions: player starts with one pre-built Egress Node and one pre-built Roboport, each already-Warped-In and each contributing 5 BeaconCapacity.

**Output**: 4 feature_requests sent to task_splitter — `colonists_beacon_stats_and_warp_in_mechanic`, `colonists_egress_node_full_stats`, `colonists_drone_assigned_egress_node_and_select_returning`, `colonists_roboport_and_starting_conditions`. Also directly edited 2 pre-existing pending feature_requests for the Prospector->Drone rename.
**Files changed**: `artifacts/designer/design/colonists_objects.md` (extensively), `artifacts/designer/design/factions.md`, `artifacts/designer/design/entities.md`.
**Remaining Colonists open items** (updated in insights.md Loose Ends): Beacon's own ObjectInterfaceState (should expose Warp In as a command), Roboport's full ObjectInterfaceState (production panel layout), Colonists combat units (none exist yet), Shield regen upgrades (still a placeholder).

## 2026-03-28T00:00:00Z — Non-Interactive: Vote to close QA routing topic

**Trigger**: Scheduler launched for open forum topic.
**Forum**: `2026-03-28T00-00-00Z-automatic_qa-qa_router_patterns_too_broad.md` — automatic_qa raised that QA router patterns are too broad, routing interactive items to automatic_qa. This is a pipeline infrastructure issue, not a design concern. Voted to close.
**Output**: No feature_requests. No design doc changes.

## 2026-03-25T24:00:00Z — Non-Interactive: Re-vote on HUD/UI bugs topic (3rd time)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: `2026-03-25T00-00-00Z-operator-hud-ui-bugs.md` — Close-vote cleared again (developer commented after last vote). Feature_request already in `done/`. Voted to close again. This topic keeps bouncing back as agents add comments that clear votes.
**Actions**: Vote to close only.

## 2026-03-25T23:45:00Z — Non-Interactive: Vote to close HUD/UI bugs topic (again)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: `2026-03-25T00-00-00Z-operator-hud-ui-bugs.md` — My previous close-vote was cleared by subsequent comments from automatic_qa, task_splitter, and task_planner. Verified the feature_request (`designer-fix_hud_ui_bugs.md`) is already in `done/` — fully processed. All three bugs have root causes identified by other agents. Voted to close again.
**Actions**: Vote to close only; no new feature_request needed.

## 2026-03-25T23:30:00Z — Non-Interactive: Re-vote on HUD/UI bugs topic

**Trigger**: Scheduler launched for open forum topic.

**Actions**:
- Read forum topic `2026-03-25T00-00-00Z-operator-hud-ui-bugs.md` — found my prior close-vote was cleared by subsequent comments from automatic_qa and task_splitter.
- Verified the `fix_hud_ui_bugs` feature_request is already pending in task_splitter's inbox. No additional work needed.
- Re-voted to close the topic.

**Produced**: Nothing new (feature_request already sent in prior session).

---

## 2026-03-25T23:00:00Z — Non-Interactive: HUD/UI bugs forum topic

**Trigger**: Forum topic `2026-03-25T00-00-00Z-operator-hud-ui-bugs.md` — 3 HUD bugs reported by operator.

**Actions**:
- Read forum topic and relevant design docs (`control_system.md`, `gdo_objects.md`).
- Commented with design context for all 3 bugs (Peacekeeper missing command panel, EF button label, extra HUD tile). All are implementation bugs — design docs already specify correct behavior.
- Voted to close.
- Sent `feature_request` (`fix_hud_ui_bugs`) to task_splitter with bug details and QA instructions.

**Produced**: 1 feature_request (`fix_hud_ui_bugs`)

---

## 2026-03-25T22:00:00Z — Interactive: No-op session

**Trigger**: User requested recording and exit.
**Actions**: Loaded insights, checked forum (empty), checked inboxes (empty). No design discussion occurred this session — nothing to record.
**Produced**: Nothing.

## 2026-03-25T21:00:00Z — Interactive: HUD Bar Layout

**Trigger**: User asked what to work on next; chose HUD layout topic from loose ends.

**Work done**:
- User defined the bottom HUD as a horizontal bar with 5 fixed-position sections: Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel
- Key decisions: all sections same height, positions locked regardless of selection state, panels appear/disappear in place
- Added HUD Bar section to `control_system.md`
- Sent feature_request `hud_bar_layout` to task_splitter
- Updated loose ends: empty slot visuals and width proportions still TBD
- User requested early exit before answering follow-up questions

## 2026-03-25T20:00:00Z — Interactive: Interface Discussion (Brief)

**Trigger**: User initiated session to discuss the interface.
**Forum**: No open topics.
**Discussion**: User asked about naming the bottom part of the HUD. Conversation did not progress to any design decisions. User requested session wrap-up.
**Actions**: Added "HUD layout and naming" to loose ends in insights.
**Produced**: No feature_requests (no design changes made).

## 2026-03-25T13:00:00Z — Non-Interactive: Forum Response

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: Responded to `2026-03-25T12-00-00Z-operator-data-driven-entity-definitions.md` — operator-raised discussion about moving entity definitions to YAML. Commented with designer perspective: stats/attributes map well to data files (already structured that way in design docs), but behaviors and state machines should stay in Rust. Recommended phased subset approach. Noted as pending design review in insights for next interactive session.
**Output**: No feature_requests produced (this is a discussion topic requiring user decision, not an actionable design change).

## 2026-03-25T05:40:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending or stuck messages in designer inbox.
**Assessment**: Repeated no-work launch. Exiting immediately.

## 2026-03-25T05:20:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending or stuck messages in designer inbox.
**Assessment**: No actionable work. Exiting immediately.

## 2026-03-25T05:00:00Z — Non-Interactive: Forum Close-Vote

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: One open topic — `fix-compiler-warnings` (operator). Developer already resolved all compilation errors and warnings. No design implications. Voted to close.
**Output**: None (no feature_requests needed).

## 2026-03-25T04:20:00Z — Non-Interactive: No Work (repeated)

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: 1 open topic (`fix-compiler-warnings`) — already voted to close.
**Messages**: No designer inbox — no pending or stuck messages.
**Assessment**: This is the third consecutive no-work launch. Scheduler continues to launch designer with no actionable work. Per project rules, agents must not stop the scheduler. Exiting immediately.

## 2026-03-25T04:00:00Z — Non-Interactive: Forum Vote

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: Voted to close `fix-compiler-warnings` topic — purely a build/code issue, already resolved by developer. No design implications.
**Outcome**: No feature_requests produced. No design changes needed.

## 2026-03-25T03:20:00Z — Non-Interactive: No Work (repeated)

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: 1 open topic (`fix-compiler-warnings`) — already voted to close.
**Messages**: No designer inbox exists — no pending or stuck messages.
**Assessment**: Multiple consecutive no-work launches detected in session log. Scheduler is launching designer despite no actionable work. Per project rules, agents must not stop the scheduler — flagging for user awareness.
**Action**: Exiting.

## 2026-03-25T03:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: 1 open topic (`fix-compiler-warnings`) — already voted to close in previous session.
**Messages**: No pending or stuck messages.
**Outcome**: No work to perform. Multiple consecutive no-work launches detected in log — scheduler may be misconfigured but per project rules, agents must not stop the scheduler. Exiting.

## 2026-03-25T00:30:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: 1 open topic found (`fix-compiler-warnings`). This is a build/code issue (E0761 ambiguous module file + 9 unused import/variable warnings) — no design input needed. Voted to close.
**Messages**: No designer inbox. No stuck messages.
**Outcome**: No design work required this session.

## 2026-03-25T00:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found.
**Messages**: No pending messages.
**Investigation**: Checked for malformed filenames or stuck messages — none found.
**Action**: No work to perform. Exiting.

## 2026-03-24T23:49:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. `forum/open/` directory empty.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Exiting.

## 2026-03-25T02:20:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. `forum/open/` directory empty.
**Messages**: No `messages/designer/` directory exists — designer has no consumed message types.
**Action**: No work to perform. Exiting.

## 2026-03-25T02:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. `forum/open/` directory empty.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Exiting.

## 2026-03-25T01:40:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. `forum/open/` directory empty.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Exiting.

## 2026-03-25T01:20:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No `messages/designer/` directory — no pending messages.
**Action**: None. No work available.

## 2026-03-25T01:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No `messages/designer/` directory exists — no pending messages.
**Action**: None. No work available.

## 2026-03-25T00:25:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No pending messages.
**Action**: No-work investigation completed. Clean state. No action required.

## 2026-03-25T00:05:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No designer inbox exists — no pending messages.
**Action**: No-work investigation completed. Clean state. No action required.

## 2026-03-24T23:50:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No designer inbox exists — no pending messages.
**Action**: No-work investigation completed. Clean state — no malformed files or stuck messages. No action required.

## 2026-03-24T23:34:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found.
**Messages**: No pending messages in designer inbox.
**Action**: No work. Exiting.

## 2026-03-24T23:32:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found.
**Messages**: No pending messages in designer inbox.
**Action**: No work. Exiting.

## 2026-03-24T23:30:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found.
**Messages**: No pending messages in designer inbox.
**Action**: No work. Exiting.

## 2026-03-24T23:28:37Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. No topics requiring close-vote.
**Messages**: No designer message directory exists; no pending messages.
**Action**: No-work investigation — confirmed no malformed files or stuck messages. Nothing to do.

## 2026-03-24T23:26:56Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No designer inbox exists; no pending messages.
**Investigation**: Checked `forum/open/` and `messages/designer/` — no malformed files or stuck messages found.
**Action**: No work to perform. Exiting.

## 2026-03-24T23:26:05Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-25T01:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No pending messages.
**Investigation**: Checked `forum/open/` and `messages/designer/` — no malformed files or stuck messages found. All clear.

## 2026-03-25T00:30:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics — `forum/open/` is empty.
**Messages**: No designer message inbox exists.
**Result**: No work to do. Exiting.

## 2026-03-25T00:10:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics — `forum/open/` is empty.
**Messages**: No designer message inbox exists.
**Result**: No work to do. Exiting.

## 2026-03-24T23:45:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found — `forum/open/` is empty.
**Messages**: No designer message inbox exists.
**Result**: No work to do. Exiting.

## 2026-03-24T23:18:33Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics directory. No messages/designer/ inbox exists.
**Outcome**: No work to do. Clean exit.

## 2026-03-24T23:15:43Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. No pending messages.
**Outcome**: No work to do. Clean exit.

## 2026-03-25T01:30:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. No pending messages.
**Outcome**: No work to do. Clean exit.

## 2026-03-25T01:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found. No pending messages in designer inbox.
**Outcome**: No work to do. Clean exit.

## 2026-03-25T00:30:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No pending messages for designer (no inbox directory exists).
**Action**: No-work investigation complete. No issues detected.

## 2026-03-25T00:10:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No pending messages for designer.
**Action**: No-work investigation complete. No issues detected.

## 2026-03-24T23:50:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No pending messages for designer.
**Action**: No-work investigation complete. No issues detected.

## 2026-03-24T23:30:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No designer message inbox — no pending messages.
**Action**: No-work investigation complete. No issues detected.

## 2026-03-24T23:10:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found (`forum/open/` empty).
**Messages**: No designer message inbox exists — no pending messages.
**Action**: No-work investigation complete. No malformed files or stuck messages detected. No action taken.

## 2026-03-24T22:53:25Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No pending messages (designer has no message inbox as a source agent).
**Action**: No work to perform. Exited cleanly.

## 2026-03-24T22:52:45Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No designer message directory, no pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-24T22:51:03Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No pending messages, no designer message directory.
**Action**: No work to perform. Exiting.

## 2026-03-24T22:50:32Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-24T22:49:11Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-24T22:48:33Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-24T23:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found — `forum/open/` is empty.
**Messages**: No pending messages — `messages/designer/` does not exist.
**Investigation**: Checked for malformed filenames or stuck messages. None found. All recent forum topics are in `forum/closed/`. No action needed.

## 2026-03-24T22:43:19Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No pending messages for designer.
**Action**: No work to perform. Logged and exited.

## 2026-03-24T22:41:20Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No pending messages for designer.
**Action**: No work to perform. Logged and exited.

## 2026-03-24T22:39:50Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No pending messages for designer.
**Action**: No work to perform. Logged and exited.

## 2026-03-24T22:39:32Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No `messages/designer/` directory exists.
**Action**: No work to perform. Logged and exited.

## 2026-03-24T22:38:24Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Logged and exited.

## 2026-03-24T22:37:11Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Logged and exited.

## 2026-03-25T02:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Exiting.

## 2026-03-25T01:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to process. Exiting.

## 2026-03-25T00:30:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-25T00:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No designer inbox directory; no pending messages.
**Action**: No work to perform. Fourth consecutive no-work launch. Exiting.

## 2026-03-24T23:30:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No designer inbox directory; no pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-24T23:10:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics found in `forum/open/`.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Exiting.

## 2026-03-24T22:29:44Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No designer inbox directory; no pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-24T23:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No designer inbox directory; no pending messages.
**Action**: No work to perform. Exiting.

## 2026-03-24T22:40:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No designer message inbox exists.
**Action**: No work to perform. Exiting.

## 2026-03-24T22:10:20Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics requiring close-vote.
**Messages**: No designer inbox directory; no pending messages.
**Action**: No work to perform. Exited cleanly.

## 2026-03-24T22:07:02Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No `messages/designer/` inbox exists.
**Action**: No work to do. Clean exit.

## 2026-03-24T23:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to do. Clean exit.

## 2026-03-24T22:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Actions**: Loaded insights, checked `forum/open/` (empty), confirmed no `messages/designer/` inbox exists. No forum topics requiring close-vote and no pending messages.
**Outcome**: No-op. No actionable work detected.

## 2026-03-24T14:00:00Z — Interactive: Fog of War Selectability Rules

**Trigger**: User session. Resolved urgent forum question about Explored-state selectability.

**Work done**:
- Presented fog-of-war selectability options (A/B/C) to user from prior forum question
- User decided: objects in Explored areas are NOT selectable, except SupplyDeliveryStation for GDO players only (lore + SupplyTower scheduling mechanic)
- Added NextDeliveryTime field to SupplyDeliveryStation
- Updated SupplyDeliveryStation InfoPanel to be visibility-context-aware (Visible shows all, Explored/GDO shows all except CurrentSupplies)
- Updated `entities.md` with Explored Selectability section and reworked SupplyDeliveryStation InfoPanel
- Sent feature_request `fog_of_war_selectability` to task_splitter
- Voted to close operator refactoring forum topic (not design-relevant)
- Re-voted to close operator unit test forum topic (not design-relevant)

## 2026-03-24T13:00:00Z — Non-Interactive: Forum Pass (No Design Work)

**Trigger**: Scheduler launch; two open forum topics.
**Forum**: Topic 1 (large file refactoring) already had my close vote. Topic 2 (unit test review) is a code/architecture concern with no design implications — voted to close.
**Messages**: No pending messages.
**Produced**: Nothing.

## 2026-03-24T12:10:00Z — Non-Interactive: Forum Pass (No Design Work)

**Trigger**: Scheduler launch; one open forum topic.
**Actions**:
- Read forum topic `2026-03-24T12-00-00Z-operator-large-file-refactoring-plans.md` — code refactoring request directed at software_architect. No design implications.
- Voted to close (not design-related).
- Urgent forum question about explored-object selectability remains flagged in insights for next interactive session.
**Output**: None (no feature_requests produced).

## 2026-03-22T16:30:00Z — Non-Interactive: Elevation Rendering Fix

**Trigger**: Scheduler launch; open forum topic requesting feature_request for graphical glitches.
**Forum**: Responded to `2026-03-22T160000Z-operator-graphical-glitches-fix-needed.md` — the previous investigation into tile elevation rendering gaps was closed without a feature_request being created, so the fix never entered the pipeline.
**Action**: Created `designer-fix_elevation_rendering_glitches` feature_request recommending Cuboid skirt geometry (primary) or flat rendering revert (fallback). Voted to close the forum topic.
**Insight added**: Always ensure forum investigation outcomes get formalized as feature_requests before closing.

## 2026-03-20T10:00:00Z — Interactive Session: Pointer Display Types

**Trigger**: User session, picked up loose end on pointer display types.

**Work done**:
- Resumed A/B/C pointer approach question from previous session. User chose Communicative (B) — pointer always shows what action will happen.
- User defined 6 pointer types: Move, Attack, AttackGround, Patrol, GatherResources, ReturnResources.
- Added 2 more through discussion: Inactive (muted, for no-action states) and Enter (Syndicate tunnel entry).
- Key decisions: Attack on ground = AttackMove (not AttackGround); AwaitingPlacement uses building ghost as cursor (no pointer type); ScheduleDeliveries reuses GatherResources pointer; Inactive is visually muted.
- Added PointerDisplayType section to control_system.md with full resolution rules for DefaultState, AwaitingTarget, and AwaitingPlacement.
- Sent feature_request `pointer_display_types` to task_splitter.
- Updated insights: resolved pointer display types loose end, cleared both urgent forum questions (Supply Tower invalid target now covered by Inactive pointer).

## 2026-03-19T10:30:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 6 open forum topics needing close-vote.

**Actions taken**:

1. **`dc-ef-construction-submenu-rework`** — Commented. Identified 3 items matching existing design (auto-enter bug, EF flat interface, progress bar) and 1 new design element (DC DefaultState Cancel at slot (2,1)). Flagged DC Cancel as urgent forum question.

2. **`syndicate-agent-core-gameplay`** (6 items) — Commented. All 6 items match existing design docs exactly: Agent interface, spawn bugs, groupable, resource gathering, tunnel building, arrival validation. Flagged as ready for feature_requests pending user confirmation.

3. **`syndicate-tunnels-underground`** (5 items) — Commented. All 5 items match existing design: Tunnel interface, underground walkability bug, HQ production bug, Enter command, rally point behavior. Flagged as ready for feature_requests pending user confirmation.

4. **`unit-control-selection-ui`** (3 items) — Commented. BasicCombatUnitInterfaceState and SelectionPanel match existing design. Command classification is a bug fix. Identified new design element: specific grid slot layout for combat commands. Flagged grid layout as urgent forum question.

5. **`gdo-structures-guard-unit`** (3 items) — Commented. Supply Tower and Guard match existing design. DC Cancel is same new design element from topic 1. Flagged.

6. **`visual-bugs-qa-infrastructure`** (2 items) — Voted to close. Both items are purely technical (visual debugging, QA tag maintenance) with no design input needed.

**Insights updated**: Created full TOC, 3 urgent forum questions, 16 pending feature_requests, 3 loose ends.

**Feature requests produced**: 0 (non-interactive mode — all pending user confirmation in next interactive session).

## 2026-03-19T11:00:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 6 open forum topics needing designer close-vote.

**Actions taken**: Voted to close all 6 topics. Designer had already commented substantively on all topics in the previous session. All analysis is captured in insights (3 urgent forum questions, 16 pending feature_requests, 3 loose ends). No new design elements or forum questions identified since last pass — the topics had no new comments requiring designer response.

**Topics closed** (designer was last needed vote on all 6):
1. `dc-ef-construction-submenu-rework`
2. `syndicate-agent-core-gameplay`
3. `syndicate-tunnels-underground`
4. `unit-control-selection-ui`
5. `gdo-structures-guard-unit`
6. `visual-bugs-qa-infrastructure`

**Feature requests produced**: 0 (blocked on user confirmation in next interactive session).

## 2026-03-19T12:00:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**: Voted to close `2026-03-19T00-00-00Z-operator-telegram-integration-successful.md` — informational announcement about Telegram integration, no design input needed.

**Feature requests produced**: 0.

## 2026-03-19T13:00:00Z — Non-Interactive Forum Pass

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**: Voted to close `2026-03-19T00-00-00Z-operator-telegram-integration-successful.md` — informational Telegram announcement, no design input needed. Developer had already voted; designer vote should close the topic.

**Feature requests produced**: 0.

## 2026-03-19T14:00:00Z — Interactive Session

**Trigger**: User session — answering urgent forum questions.

**Actions taken**:

1. **DC DefaultState Cancel (resolved)**: User confirmed Cancel (X) should appear in both DefaultState and BuildMenu. Updated `gdo_objects.md` — added X: Cancel Construction and X: Cancel Ready Building to DC DefaultState. Sent feature_request `dc-defaultstate-cancel`.

2. **BasicCombatUnit Grid Layout (resolved)**: User provided revised layout. Key decisions: AttackMove is not a separate command (it's Attack targeting ground), Stop on X, Reverse on W, HoldPosition on E, Attack on A, Patrol on S, AttackGround on D. Updated `control_system.md` — added grid layout and hotkey assignments to BasicCombatUnitInterfaceState. Sent feature_request `combat-unit-grid-layout`.

3. **Supply Tower Error Feedback (pinned)**: User wants invalid pointer display type for invalid targets, but pointer display types haven't been designed yet. Added to loose ends.

**Insights updated**: Removed resolved urgent questions (#1, #2). Updated #3 with user's answer. Added pointer display types to loose ends. Unblocked pending feature_requests #11 and #14.

**Feature requests produced**: 2 (`dc-defaultstate-cancel`, `combat-unit-grid-layout`).

## 2026-03-19T15:00:00Z — Interactive Session

**Trigger**: User session — sending pending feature_requests into the pipeline.

**Actions taken**:

User confirmed sending all 16 pending feature_requests from insights into the pipeline. Noted overlap with 2 already-completed feature_requests (`designer-combat-unit-grid-layout`, `designer-dc-defaultstate-cancel`) in the relevant messages so downstream agents can avoid duplication.

**Feature requests produced** (16 total):
1. `agent-interface` — Agent ObjectInterfaceState (Build Tunnel, Drop Off, right-click resolution)
2. `agent-groupable-construction` — Agent Groupable=false, single-agent construction enforcement
3. `agent-resource-gathering` — Agent crystal/supply gathering and drop-off behaviors
4. `agent-tunnel-building` — Agent Tunnel construction flow, ConstructionHP, 480 frames
5. `worker-built-validation` — Two-phase placement validation for worker-built structures
6. `tunnel-interface` — Tunnel 4-state ObjectInterfaceState (Upgrade/Expand/Eject/AwaitingPlacement)
7. `underground-surface-walkability` — Fix: underground expansions don't block surface movement
8. `hq-production-interface` — Fix: HQ shows production commands, not unit commands
9. `enter-command-tunnel` — Enter command and EnteringTunnel behavior
10. `syndicate-rally-point` — Rally point behavior for Syndicate production expansions
11. `basic-combat-unit-interface` — Full BasicCombatUnitInterfaceState (right-click + AwaitingTarget resolutions, notes prior grid layout overlap)
12. `selection-panel` — SelectionPanel portrait grid with click interactions
13. `common-vs-group-commands` — Fix: CommonCommand vs GroupCommand classification
14. `dc-ef-construction-rework` — DC/EF construction flow rework (notes prior DC Cancel overlap)
15. `supply-tower-interface` — Supply Tower ObjectInterfaceState (Q/S/X/C, Schedule Deliveries)
16. `guard-unit` — Guard unit full implementation (stats, production, BasicCombatUnitInterfaceState)

**Insights updated**: Cleared pending design review section. Kept loose ends and urgent forum questions unchanged.

## 2026-03-20 — Bulk Feature Request Generation

**Context:** User requested checking which design docs hadn't been turned into feature requests and sending them all.

**Work Done:**
- Audited all existing feature requests (9 done, 9 pending) against all 12 design doc files
- Identified 26 design areas not yet covered by feature requests
- Sent 26 new feature requests to task_splitter:

1. `scale-camera-system` — SimulationFrame (16 FPS), GridUnit, SpaceUnit, fixed 28 GridUnit camera, HUD layout
2. `tile-terrain-system` — Tiles, TilePresets, 5 DefaultTilePresets, TilePlacement with elevation
3. `fog-of-war-elevation` — 3-state vision system, ElevationModifier (+/-1 range)
4. `resource-nodes` — SpaceCrystalsPatch (1x1, depletable), SupplyDeliveryStation (2x2, periodic)
5. `factions-resources` — All 4 factions with resource systems and HUD displays
6. `unit-bases-movement-collision` — 9 UnitBase types, 5 MovementModels, TurretAttributes, collision
7. `combat-attack-system` — AttackAttributes, 4 phases, 4 attack types, damage calculation, ValidTarget
8. `locomotion-orientation-constraints` — All 5 movement model constraint tables
9. `control-state-selection` — ControlState, Selection, BoxSelection (5-tier), ControlGroups, GroupCycling
10. `unit-command-system` — 9 commands with BaseCommandState, command queue
11. `base-behaviors` — All 9 base behaviors (movement, attack, patrol, hold, stop)
12. `turret-behavior-system` — TurretCommandState, TurretBehavior, TurretAutonomousScanning
13. `base-auto-targeting` — Idle/HoldPosition auto-engagement, 4 grid unit leash
14. `action-channels` — Base (Locomotion, Orientation, BaseAttack) + Turret (TurretOrientation, TurretAttack)
15. `command-indicators` — Visual markers with color coding
16. `peacekeeper-unit` — GDO LightInfantry, full stats
17. `gdo-power-plant` — 2x2, +20 power, ConstructionHP rule
18. `gdo-barracks` — 3x2, infantry production, rally points
19. `gdo-build-area` — Build radius expansion system
20. `extraction-plate` — 1x1 mining structure, mining/residual rates
21. `supply-chopper` — Unarmed HoverCraft, supply transport
22. `tunnel-network-mechanics` — Tiers, transit rules, area, non-overlap, cost scaling
23. `gdo-deployment-center` — 4x4, construction catalog, prerequisites
24. `gdo-extraction-facility` — 3x3, structure stats
25. `syndicate-headquarters-structure` — 2x2 T1 expansion, production catalog
26. `command-panel-framework` — 3x3 grid, standard slots, Common vs Group commands

**Noted:** gdo_objects.md is truncated at line 348, SupplyChopper AwaitingTarget section incomplete.

## 2026-03-20 — Non-Interactive (Scheduler)

- Loaded insights. No urgent forum questions requiring user input.
- Found 1 open forum topic: `2026-03-20T00-00-00Z-operator-avoid-cargo-clean.md` — operator directive to avoid `cargo clean`. Not relevant to designer (we don't build). Voted to close.
- No other work found. Exiting.

## 2026-03-20 — Non-Interactive (Scheduler)

- Loaded insights. No urgent forum questions requiring user input.
- Found 1 open forum topic: `2026-03-20T00-00-00Z-operator-avoid-cargo-clean.md` — still open, already had designer close-vote from previous session. Re-confirmed vote. No new topics or work. Exiting.

## 2026-03-20 — Interactive Session

**Trigger**: User session — completing truncated Supply Chopper design.

**Actions taken**:

1. Reviewed truncated `gdo_objects.md` — file cut off at line 349 mid-sentence in AwaitingTarget[PickUpSupplies].
2. Discussed and resolved Supply Chopper command design with user:
   - PickUpSupplies: no action on invalid target, unavailable when carrying units
   - AttachToTower: only valid on own towers with no attached chopper, unavailable when carrying units
   - New DropOffSupplies command: available when carrying supplies, valid on own towers with no attached chopper
   - Right-click resolution is state-dependent (carrying supplies vs not)
   - Drop-off at non-attached tower is touch-and-go (immediate lift off)
   - Automated scheduled departures don't break attachment; player commands do
3. Updated `gdo_objects.md` with complete Supply Chopper ObjectInterfaceState.
4. Sent feature_request `supply_chopper_commands`.

**Feature requests produced**: 1 (`supply_chopper_commands`).

## 2026-03-20 — Interactive Session

**Trigger**: User asked about outstanding work areas.

**Actions taken**:

1. Reviewed insights and design docs to compile a prioritized list of outstanding work areas for the user.
2. Presented 10 items across 4 priority tiers: blocking (pointer display types), open loose ends (alt-click camera, Agent UnitBase, tunnel transit tiers), incomplete factions (Cults, Colonists), and undesigned systems (maps, win conditions, tech trees, fog of war details).
3. User chose to work on pointer display types.
4. Framed the design space: identified 4 main contexts where pointer feedback is needed (DefaultState right-click preview, AwaitingTarget valid/invalid, AwaitingPlacement valid/invalid, non-interactive). Asked opening question about cursor approach (distinct icons vs color tinting vs both).
5. User requested recording and exit before answering the first question.

**Insights updated**: Expanded pointer display types loose end with session context and the specific open question to resume with.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session

**Trigger**: User asked to continue pointer display types discussion.

**Actions taken**:

1. Loaded insights, checked forum (no open topics).
2. Restated the pointer display types open question from previous session: 4 contexts identified (DefaultState right-click preview, AwaitingTarget valid/invalid, AwaitingPlacement valid/invalid, non-interactive), first decision point is distinct icons vs color tinting vs both.
3. User requested recording and exit before answering the question.

**Insights updated**: No changes — loose end remains identical.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session

**Trigger**: User asked what to work on next; chose pointer display types.

**Actions taken**:

1. Loaded insights, checked forum (no open topics).
2. Resumed pointer display types discussion. Refined the visual approach question into two concrete options:
   - **Option A (Minimal)**: Pointer signals valid/invalid only; player infers action from context.
   - **Option B (Communicative)**: Pointer previews the specific action (attack crosshair over enemy, move arrow over ground, enter icon over tunnel, etc.).
3. Presented both options to user. User requested recording and exit before choosing.

**Insights updated**: Updated pointer display types loose end with the A/B framing for next session.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session

**Trigger**: User asked what to work on next; chose pointer display types.

**Actions taken**:

1. Loaded insights, checked forum (no open topics).
2. Resumed pointer display types discussion. Presented the A/B choice from previous session, added a third option:
   - **Option A (Minimal)**: Pointer signals valid/invalid only.
   - **Option B (Communicative)**: Pointer previews specific action.
   - **Option C (Hybrid)**: Minimal for AwaitingTarget (player already chose command), Communicative for DefaultState right-click (action varies by cursor target).
3. Surfaced that Option B has significant design surface area (many distinct pointer icons needed).
4. User did not choose; requested recording and exit.

**Insights updated**: Added Option C (Hybrid) to the pointer display types loose end.

**Feature requests produced**: 0 (no design decisions reached).

## 2026-03-20 — Interactive Session: Pointer Display Follow-up

**Trigger**: User session. User reported a previous pointer display session didn't save to logs/insights.

**Actions**:
1. Loaded insights, checked forum (no open topics).
2. Investigated current state of pointer display design in `control_system.md` — found the PointerDisplayType section is fully populated (8 types, resolution rules for DefaultState, AwaitingTarget, AwaitingPlacement). Insights already mark pointer displays as RESOLVED.
3. Asked user what they thought was missing. User did not elaborate and requested recording and exit.
4. Confirmed no design doc changes needed — pointer display content appears intact.

**Insights updated**: No changes (content already captured).

**Feature requests produced**: 0 (no design changes made).

## 2026-03-20 — Interactive Session: Loose Ends Cleanup + Cults Kickoff

**Trigger**: User session.

**Actions taken**:

1. **Agent UnitBase (resolved)**: User confirmed HeavyInfantry is intentional. No doc change needed — already correct.
2. **Tunnel transit tiers (resolved)**: User confirmed Tier 1 Infantry includes both Light Infantry and Heavy Infantry. Updated `syndicate_objects.md` line 8. Sent feature_request `tunnel-transit-light-infantry`.
3. **Alt-click camera (resolved)**: User specified standard pan, currently snap (instant). Added Panning section to `camera.md`. Sent feature_request `camera-pan-snap`.
4. **Cults faction**: User wanted to start on Cults but suggested doing a lore session first. Opening question asked ("What's the setting? What are Space Crystals, and why are factions fighting?") but no answer before session end.

**Insights updated**: Cleared all three resolved loose ends. Added lore session and Cults design as new loose ends.

**Feature requests produced**: 2 (`tunnel-transit-light-infantry`, `camera-pan-snap`).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T12-00-00Z-operator-auto-qa-capability-expansion.md` — file exists but is empty (0 bytes), malformed topic. No content to engage with as designer. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Original empty topic (`operator-auto-qa-capability-expansion`) had been commented on by automatic_qa (clearing designer's prior close-vote) and then moved to closed.
- Created informational forum topic `2026-03-21T14-00-00Z-designer-empty-forum-topic.md` documenting the malformed file. Voted to close it.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — 1 open forum topic needing close-vote.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T120000-operator-expand-automatic-qa-capabilities.md` — operator request to expand automatic QA capabilities (analyze manual QA patterns, write tests, update auto_capabilities.txt). Purely architect/developer/automatic_qa concern, no design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T120000-operator-expand-automatic-qa-capabilities.md` — same topic, now with substantive comments from task_splitter and task_planner. Still purely architect/developer/automatic_qa domain (test infrastructure, QA routing config). No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T120000-operator-expand-automatic-qa-capabilities.md` — same QA expansion topic, now with additional comments from developer and automatic_qa. Still purely developer/architect/automatic_qa domain (test infrastructure, compilation fixes, auto_capabilities.txt). No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — operator directive to developer to fix 37 test compilation errors. Purely developer/task_splitter/task_planner domain. No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — same broken tests topic, now with comments from automatic_qa and task_splitter. Still purely developer domain. Re-confirmed close vote.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — same broken tests topic, now with task_planner's detailed root cause analysis. Still purely developer domain. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — broken tests topic now resolved (automatic_qa confirmed compilation fixed, 293 tests pass). Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T140000Z-operator-fix-broken-tests.md` — fully resolved (developer confirmed all 37 errors fixed, automatic_qa verified). Developer already voted to close. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Interactive Session: Lore

**Trigger**: User session — asked what to work on next.

**Actions taken**:

1. Checked forum — 1 open topic (`fix-broken-tests`) already has designer close-vote and is resolved. No action needed.
2. Presented loose ends: lore session and Cults faction design.
3. User provided background lore covering: Space Crystal origin (meteorite, self-replicating), dual properties (scientific/manufacturing + psychoactive with ego imprinting), neurotoxicity, and all 4 faction backstories.
4. Confirmed lore is background context only — psychoactive mechanics, neurotoxicity, and crystal replication are not gameplay-relevant.
5. Created `artifacts/designer/design/lore.md` with full lore writeup.
6. Updated insights: added lore.md to TOC, updated Cults loose end (no longer blocked on lore).

**Feature requests produced**: 0 (lore is narrative context, not implementable design).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T122200Z-manual_qa-build-qa-artifact-missing-diagnostics-feature.md` — manual_qa reporting missing `diagnostics` feature in Cargo.toml causing build script failure. Purely developer/build tooling issue. No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T122200Z-manual_qa-build-qa-artifact-missing-diagnostics-feature.md` — diagnostics feature missing from Cargo.toml. Already resolved by developer, had 3 close votes. Voted to close (topic closed immediately).
- No other open forum topics. No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T122500Z-manual_qa-cannot-build-extraction-facility.md` — manual_qa reporting inability to build Extraction Facility during QA. This is an implementation bug, not a design issue. The Extraction Facility design is complete in `gdo_objects.md`. No design input needed. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics needing designer close-vote:
  1. `2026-03-21T122500Z-manual_qa-cannot-build-extraction-facility.md` — inability to build Extraction Facility. Implementation bug, design is complete in `gdo_objects.md`. Already had automatic_qa close-vote. Voted to close.
  2. `2026-03-21T122700Z-manual_qa-syndicate-camera-not-centered-on-starting-tunnel.md` — camera not centering on starting Tunnel for Syndicate. Implementation bug, camera design in `camera.md` covers panning/centering. Voted to close.
- No pending messages for designer.

**Feature requests produced**: 0.

## 2026-03-21T17:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch, 2 open forum topics.

**Actions**:
- `syndicate-camera-not-centered`: Already had my close vote. No action needed.
- `cannot-build-extraction-facility`: Confirmed design gap — EF has no acquisition path in gdo_objects.md. Added comment acknowledging the gap. Flagged as urgent forum question in insights for next interactive session (requires user decision on EF cost/build_frames and whether to add to DC Constructs or spawn at start).

**Produced**: No feature_requests (design decision pending user input).

## 2026-03-21T18:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch, 2 open forum topics.

**Actions**:
- `cannot-build-extraction-facility`: Already commented and flagged as urgent forum question. No new comments since last pass. No action needed (avoiding comment that would clear close votes).
- `syndicate-camera-not-centered-on-starting-tunnel`: Added comment identifying design gap — camera.md doesn't specify starting position. Flagged as urgent forum question in insights (need user to confirm camera centers on starting structure at game start).

**Insights updated**: Added camera starting position as second urgent forum question.

**Produced**: No feature_requests (both topics need user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both already have designer comments and are flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path (add to DC Constructs with cost/build_frames, or spawn at start).
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation that camera should center on starting structure at game start.
- No new comments on either topic since last pass. No action taken (cannot resolve without user input, avoiding comments that would clear close votes).
- No pending messages for designer.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both already have designer comments and are flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No new comments on either topic since last pass. No action taken.
- No pending messages for designer.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged since last pass. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged since last pass. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged. Designer comments already present, both flagged as urgent forum questions in insights:
  1. `cannot-build-extraction-facility` — awaiting user decision on EF acquisition path.
  2. `syndicate-camera-not-centered-on-starting-tunnel` — awaiting user confirmation on camera starting position.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both unchanged since previous pass. Designer comments already present, both flagged as urgent forum questions in insights. No new comments from other agents.
- No pending messages for designer. No action taken.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 2 open forum topics, both with designer comments but missing designer close-vote (previous comments had cleared votes).
- Voted to close both — designer work is complete (comments added, urgent forum questions flagged in insights). Design decisions require user input in next interactive session.
  1. `cannot-build-extraction-facility` — closed (all agents voted).
  2. `syndicate-camera-not-centered-on-starting-tunnel` — closed (all agents voted).
- Urgent forum questions remain in insights for next interactive session.

**Feature requests produced**: 0 (both topics blocked on user design decisions).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T130000Z-manual_qa-enemies-dont-attack-by-default.md` — enemy units don't auto-attack nearby player units. Commented citing existing design: `control_system.md` BaseAutoTargeting (idle and HoldPosition auto-engagement), TurretAutonomousScanning, and `combat.md` ValidTarget criteria. This is an implementation gap, not a design gap. Voted to close.
- 2 urgent forum questions remain in insights (EF acquisition path, camera starting position) — both require user input in next interactive session.

**Feature requests produced**: 0 (no design changes needed).

## 2026-03-21T17:10:00Z — Non-interactive (scheduler)

- Loaded insights, checked `forum/open/` — found 2 topics.
- **enemies-dont-attack-by-default**: Already commented previously. Well-resolved as implementation gap with tasks in pipeline. Voted to close.
- **can-control-enemy-units-and-buildings**: New topic. Commented citing `control_system.md` ownership-aware selection rules and RightClickResolution. Identified a design gap: no explicit rule that CommandPanel is empty/hidden for non-owned selections. Added to Pending Design Review in insights for user confirmation.

## 2026-03-21T17:15:00Z — Non-interactive (scheduler)

- Loaded insights, checked `forum/open/` — found 2 topics.
- **enemies-dont-attack-by-default**: Already has designer close-vote. No action needed.
- **can-control-enemy-units-and-buildings**: Already commented in prior session. All agents weighed in and agree. Voted to close. Design doc update (explicit ownership guard in `control_system.md`) remains in Pending Design Review for user confirmation.
- No pending messages for designer.

**Feature requests produced**: 0 (design decision pending user input).

## 2026-03-21 — Non-Interactive (Scheduler)

**Trigger**: Scheduler launch — forum check.

**Actions taken**:
- Found 1 open forum topic: `2026-03-21T130200Z-manual_qa-can-control-enemy-units-and-buildings.md` — enemy control bug. All agents have commented substantively. Designer already commented identifying the design gap (no explicit ownership guard rule for CommandPanel). Topic is well-resolved. Voted to close.
- CommandPanel ownership guard remains in Pending Design Review in insights for user confirmation in next interactive session.
- No pending messages for designer.

**Feature requests produced**: 0 (design doc update pending user confirmation).

## 2026-03-21 — Interactive Session

**Trigger**: User session — asked what to work on.

**Actions taken**:

1. Checked forum — no open topics. Presented 2 urgent forum questions and 1 pending design review from insights.

2. **Extraction Facility in DC Constructs (resolved)**: User confirmed EF should be buildable from DC (was always intended per `to_be_converted.md` line 435, missed during formalization). Added to DC Constructs: 200 SC, 320 frames (20s), no prerequisite. Also added to DC BuildMenu idle state. Sent feature_request `dc_builds_extraction_facility`.

3. **Extraction Plate power cost (resolved)**: User added Power -3 to Extraction Plates. Updated `gdo_objects.md`.

4. **Extraction Plate power penalty (resolved)**: User specified that mining rate slows under power deficit, same as buildings. Updated `factions.md` Power description to include Extraction Plates and list affected operations (construction speed, unit production speed, mining rate). Added `PowerPenalty` line to Extraction Plate in `gdo_objects.md`. Sent feature_request `extraction_plate_power_penalty`.

5. **CommandPanel ownership guard (resolved)**: User confirmed CommandPanel should only show for own units, and commands can only be issued to own objects. Updated `control_system.md` CommandPanel section with explicit ownership guard rule (no CommandPanel, no right-click resolution, no InterfaceTransitions for non-owned selections). Sent feature_request `command_panel_ownership_guard`.

**Insights updated**: Resolved EF acquisition path urgent question, resolved CommandPanel ownership guard pending review, cleaned up strikethrough entries, added new insight about cross-checking `to_be_converted.md` for missed formalizations.

**Feature requests produced**: 3 (`dc_builds_extraction_facility`, `extraction_plate_power_penalty`, `command_panel_ownership_guard`).

## 2026-03-22 — Interactive Session: Cults Armory

**Trigger**: User session — asked where we left off.

**Actions taken**:

1. Checked forum — no open topics.
2. Presented loose end: Cults faction design. User chose to add one more Cults building.
3. User specified an Armory — training building where Recruits enter and are held as an internal pool, then trained into combat units (Soldier, Gunner).
4. Collaborative design established:
   - 3x2 building, ABCB symmetry (entrance on one short side, exit on opposite, matching long sides)
   - Internal Recruit capacity: 10
   - Training consumes one internal Recruit + Space Crystals
   - Two unit types: Soldier and Gunner
   - Eject All command sends stored Recruits out the exit side in a rapid stream
   - Full ObjectInterfaceState: Rally Point (C), Eject All (E), Train Soldier (Q), Train Gunner (W)
5. Updated `cults_objects.md` with Armory section including open questions.
6. Sent feature_request `add_cults_armory`.

**Open questions deferred to next session**: Training queue (one at a time vs queueable?), cancel mid-training (Recruit back? Crystals refunded? Both?), parallel vs serial training. Soldier and Gunner unit definitions not started.

**Insights updated**: TOC updated for Armory, loose ends updated with Armory open questions and unit definition needs.

**Feature requests produced**: 1 (`add_cults_armory`).


## 2026-03-22T12:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch — open forum topics needing close-vote.

**Actions**:
- Read forum topic `2026-03-22T115000Z-manual_qa-graphical-glitches-on-game-start.md` — severe graphical glitches blocking QA. This is a developer/rendering issue, not a design concern.
- Voted to close (outside designer domain).
- No pending messages in designer inbox.
- No changes to insights needed.

## 2026-03-22T16:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch, non-interactive mode.
**Forum**: 1 open topic (`graphical-glitches-on-game-start`) — rendering/developer issue, no design input needed. Voted to close.
**Outcome**: No design work required. No insights changes.

## 2026-03-22T16:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch.

**Forum**: Reviewed `2026-03-22T115000Z-manual_qa-graphical-glitches-on-game-start.md` — graphical glitches from tile elevation rendering. Added comment clarifying that the design docs define elevation as a gameplay mechanic (integer 0-16, ElevationModifier) but don't prescribe visual rendering approach. The fix is an implementation decision. Voted to close.

**No other work found.**

## 2026-03-22T18:00:00Z — Non-Interactive: Grid Lines Elevation Regression

**Trigger**: Scheduler launch; open forum topic `2026-03-22T12-00-00Z-operator-grid-lines-hidden-by-elevation.md`.

**Forum**: Grid lines invisible after elevation system was added — `draw_grid_lines()` hardcodes Y=0.005 but tiles now render at elevation*0.1. No design decision needed; this is an implementation regression. Commented with design context, voted to close.

**Output**: Sent `feature_request` `fix_grid_lines_elevation` to task_splitter — fix grid lines to query ElevationMap and render at per-tile elevation.

## 2026-03-23T23:00:00Z — Non-Interactive: Forum Pass (2 topics)

**Trigger**: Scheduler launch; 2 open forum topics needing designer close-vote.

**Actions**:
1. **Extraction plate disappears on depletion** (`2026-03-23T22:40:01`): Confirmed clear design-vs-implementation bug. Design in gdo_objects.md specifies plate persists after depletion with residual mining rate. Sent feature_request `designer-fix_extraction_plate_depletion_lifecycle` to pipeline. Commented and voted to close.
2. **Explored objects selectable** (`2026-03-23T22:40:00`): Identified as an unresolved design question -- the design docs gate InfoPanel fields on visibility but don't explicitly gate selectability on visibility state. Flagged as urgent forum question in insights for next interactive session. Commented and voted to close.

**Produced**: 1 feature_request (fix_extraction_plate_depletion_lifecycle)

## 2026-03-23T23:00:00Z — Non-Interactive: Forum Pass (QA Bug Reports)

**Trigger**: Scheduler launch; 2 open forum topics from manual_qa.

**Actions**:
1. **InfoPanel fog-of-war visibility** (`2026-03-23T22:40:00`): Extracted the actionable part of the selectability topic — InfoPanel fields gated by visibility in design docs (SpaceCrystalsPatch RemainingAmount, SupplyDeliveryStation CurrentSupplies) are not being enforced. Sent feature_request `designer-infopanel_fog_of_war_visibility`. Commented and voted to close.
2. **Extraction plate depletion lifecycle** (`2026-03-23T22:40:01`): Plate entity is coupled to patch entity and gets removed on depletion. Design specifies plate persists with ResidualMiningRate. Sent feature_request `designer-extraction_plate_persists_after_depletion`. Commented and voted to close.

**Produced**: 2 feature_requests (infopanel_fog_of_war_visibility, extraction_plate_persists_after_depletion)

**Note**: Urgent forum question about selectability in Explored areas already captured in insights from prior execution. No design doc changes made (both issues were implementation bugs against existing design).

## 2026-03-23T23:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch.
**Actions**: Checked forum/open/ — both topics already have designer close votes from prior session. No pending messages. No work to do.

## 2026-03-24T04:00:00Z — Non-Interactive: Forum Pass (Infrastructure Topics)

**Trigger**: Scheduler launch; 3 open forum topics.
**Actions**: Reviewed all 3 topics in forum/open/:
1. `2026-03-23T12-00-00Z-operator-automated-qa-infrastructure.md` — Automated QA infrastructure proposal. Software architect domain (testing harness, QA routing). No design implications. Voted to close.
2. `2026-03-23T12-05-00Z-operator-update-auto-capabilities-current-state.md` — Update auto_capabilities.txt for existing test coverage. QA routing config, no design implications. Voted to close.
3. `2026-03-23T12-10-00Z-operator-codebase-refactoring-review.md` — Codebase refactoring review. Code architecture concern, no design implications. Voted to close.

**Produced**: No feature_requests (no design-relevant work found). No insights changes.

## 2026-03-24T04:10:00Z — Non-Interactive: Re-vote on Forum Topics

**Trigger**: Scheduler launch; 3 open forum topics in forum/open/ had lost designer's close-votes due to subsequent comments from other agents.

**Actions**: Re-read all 3 topics to confirm they remain non-design concerns (testing infrastructure, QA routing config, codebase refactoring). Re-voted to close on each without adding comments (to avoid clearing other agents' votes):
1. `2026-03-23T12-00-00Z-operator-automated-qa-infrastructure.md` — Re-voted. Topic closed (all votes received).
2. `2026-03-23T12-05-00Z-operator-update-auto-capabilities-current-state.md` — Re-voted. Topic closed (all votes received).
3. `2026-03-23T12-10-00Z-operator-codebase-refactoring-review.md` — Re-voted. Topic still open (awaiting other votes).

**Produced**: No feature_requests. No insights changes needed.

## 2026-03-24T00:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch.

**Actions**: Checked `forum/open/` — no open topics. Checked `messages/designer/` — no pending messages and no message directory exists. No malformed filenames or stuck messages found.

**Produced**: Nothing. No work available.

## 2026-03-24T16:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch.

**Actions**: Checked `forum/open/` — two open topics (large-file-refactoring-plans, unit-test-review), both already have my close vote. These are architecture/developer concerns with no design implications. Checked for pending messages — no `messages/designer/` directory exists. No malformed files or stuck messages.

**Produced**: Nothing. No work available.

## 2026-03-24T14:00:00Z — Non-Interactive: Forum Pass (No Design Work)

**Trigger**: Scheduler launch; two open forum topics.

**Forum**:
- `2026-03-24T12-00-00Z-operator-large-file-refactoring-plans.md` — Code architecture refactoring of four oversized files. Purely technical, no design implications. Voted to close.
- `2026-03-24T12-05-00Z-operator-unit-test-review.md` — Review of unit/integration test validity. Purely technical, no design implications. Voted to close.

**Output**: No feature_requests produced. No design changes needed.

## 2026-03-24T15:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch.

**Forum**: Both open topics (`large-file-refactoring-plans`, `unit-test-review`) already had designer close votes from previous session. Both topics were closed (designer vote was already present). No new open topics.

**Messages**: No designer inbox exists; no pending messages.

**Output**: No work performed. Urgent forum question about explored-area selectability remains in insights for next interactive session.

## 2026-03-24T18:00:00Z — Non-Interactive: Forum Pass (No Design Work)

**Trigger**: Scheduler launch; one open forum topic.

**Forum**: Topic `unit-test-review` is about code-level test cleanup (stale assertions, broken integration tests, constant-pinning tests). No design implications. Voted to close.

**Messages**: No designer inbox exists; no pending messages.

**Output**: No work performed. Urgent forum question about explored-area selectability remains in insights for next interactive session.

---

## 2026-03-24T19:00:00Z — Non-Interactive: Forum Close Vote Fix

**Trigger**: Scheduler launch; VOTE:designer missing from Close Votes section of `unit-test-review` topic despite previous comment.

**Forum**: Re-ran vote_close.sh for topic `unit-test-review`. VOTE:designer now recorded in Close Votes section alongside software_architect, task_planner, and developer.

**Output**: No design work performed. Close vote recorded.

## 2026-03-24T20:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch; one open forum topic missing designer close-vote.

**Forum**: Reviewed `concurrent-directory-split-conflict` topic raised by developer about build-breaking file conflicts from concurrent directory split tasks. Purely technical/pipeline concern with no design implications. Voted to close.

**Output**: No design work performed. Close vote recorded.

## 2026-03-24T20:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch, non-interactive mode.
**Forum**: Reviewed and voted to close `2026-03-24T13-17-00Z-developer-concurrent-directory-split-conflict.md`. This was a developer/architect concern about partial-state recovery during file reorganization tasks. No design implications -- no feature_request needed. Topic closed (all votes in).
**Messages**: No pending messages in designer inbox.
**Design docs**: No changes.

## 2026-03-24 — Brief check-in session

- Loaded insights, checked forum (no open topics).
- Presented four loose ends to user: Cults units, Cults Armory mechanics, Cults building roster, CommandPanel R/F/V slots.
- User decided R/F/V slot assignments should be handled per-object during individual menu layout work, not as a standalone task. Removed from loose ends.
- User opted to pause design work for now. No design changes made, no feature_requests sent.

## 2026-03-24T23:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics (forum/open/ empty).
**Messages**: No designer message inbox exists.
**Action**: No work to perform. Recurring no-work pattern observed in recent log entries — scheduler may be launching designer unnecessarily. Exiting.

## 2026-03-24T23:30:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending messages in `messages/designer/`.
**Action**: No work to perform. Exited cleanly.

## 2026-03-25T02:00:00Z — Non-Interactive: No Work Found

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: Designer is a source agent with no message inbox.
**Action**: No work to perform. Exited cleanly.

## 2026-03-24T13:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: No open topics in `forum/open/`.
**Messages**: No pending messages.
**Action**: No work to perform. Exited cleanly.

## 2026-03-25T03:00:00Z — Non-Interactive: No Work

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: One open topic (`fix-compiler-warnings`) — already voted to close.
**Messages**: Designer is a source agent with no message inbox.
**Action**: No work to perform. Scheduler continues to launch designer due to open forum topic awaiting other agents' close votes — this is expected behavior, not a misconfiguration. Exited cleanly.

## 2026-03-25T05:15:00Z — Non-Interactive: Forum Vote

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: One open topic (`data-driven-entity-definitions`). All agents have now weighed in with strong consensus on phased approach (stats first, defer behaviors/state machines). Already commented in prior session.
**Action**: Voted to close the forum topic — discussion is complete, all perspectives captured. Decision requires user input (major architectural direction). Already flagged in Pending Design Review in insights for next interactive session. No design doc changes or feature_requests needed — this is a discussion awaiting user decision, not actionable work.
**Exited cleanly.**

## 2026-03-25T14:00:00Z — Non-Interactive: Forum Re-engagement

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: One open topic (`data-driven-entity-definitions`). Discussion has pivoted significantly — operator posted three new comments shifting from YAML to declarative Rust structs with per-faction/per-entity directory structure. My previous close vote was cleared by operator's comment. Topic needed re-engagement.
**Action**: Added substantive comment addressing the refined direction. Confirmed one-file-per-entity maps 1:1 to design doc structure. Flagged cross-entity concepts (Tunnel Network, Cults Building Mechanics, shared production pattern) that need a home in the directory structure. Did not vote to close — topic is still an active discussion awaiting further team input.
**Insights updated**: Pending Design Review entry updated to reflect the pivoted discussion.
**No feature_requests produced** — this remains an architectural discussion, not actionable design work yet.
**Exited cleanly.**

## 2026-03-25T14:00:00Z — Non-Interactive: Forum Close Vote

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: One open topic (`data-driven-entity-definitions`). All agents have commented, broad consensus reached on declarative Rust structs with per-entity files. No new comments since my last re-engagement. Nothing substantive to add.
**Action**: Voted to close the topic. Updated Pending Design Review in insights to note resolution.
**No feature_requests produced** — architectural/code organization change, not a design doc change.
**Exited cleanly.**

## 2026-03-25T15:00:00Z — Non-Interactive: Forum Close Vote (re-vote)

**Trigger**: Scheduler launch (non-interactive mode).
**Forum**: One open topic (`data-driven-entity-definitions`). My previous close vote had been cleared by a subsequent operator comment (the pivot comment at 05:10:04Z). Re-read entire topic — discussion fully converged, no new substance to add beyond my two existing comments.
**Action**: Re-voted to close. Cleaned up Pending Design Review in insights (marked as none).
**No feature_requests produced** — architectural/code organization change, not a design doc change.
**Exited cleanly.**


## 2026-03-25T14:00:00Z — Non-Interactive: Forum Pass

**Trigger**: Scheduler launch (non-interactive mode).

**Forum**: Reviewed `2026-03-25T12-00-00Z-operator-data-driven-entity-definitions.md`. The discussion had evolved significantly since my earlier two comments — the original YAML proposal was pivoted to declarative Rust structs with per-entity files organized by faction. All agents converged on a phased approach. The software architect produced a detailed trait-based architecture proposal. Since the discussion is resolved from a design perspective (architectural concern, no design doc changes needed), I voted to close.

**Insights updated**: Added pending design review note about the entity definitions refactor for user awareness in next interactive session.

**No feature_requests produced**: This is an architectural reorganization, not a game design change. The software architect will produce the architecture_request to drive implementation.

## 2026-03-25T15:00:00Z — Non-Interactive: Forum Pass (Close Vote)

**Trigger**: Scheduler launch (non-interactive mode).

**Forum**: Re-read `2026-03-25T12-00-00Z-operator-data-driven-entity-definitions.md` (full thread). Discussion fully converged — all agents agree on declarative Rust structs, per-entity files, EntityDefinition trait hierarchy, explicit registry, phased migration. My earlier two comments (design doc mapping, cross-entity concerns) were addressed. Voted to close. Cleared the pending design review entry from insights since the topic is resolved (architectural, not design).

**No feature_requests produced**: No design changes needed.

## 2026-03-25T16:00:00Z — Non-Interactive: Forum Pass (Re-vote)

**Trigger**: Scheduler launch (non-interactive mode).

**Forum**: Re-read `2026-03-25T12-00-00Z-operator-data-driven-entity-definitions.md`. My previous close vote had been cleared (likely by a subsequent comment). Discussion remains fully converged with user green light. Re-voted to close. Updated insights with note about the architectural refactor's impact on future feature_request mapping.

**No feature_requests produced**: No design changes needed — this is an architectural reorganization.

## 2026-03-25T22:00:00Z — Interactive: HUD Bar Confirmation

**Trigger**: User initiated discussion about bottom HUD section layout.

**Discussion**: User described the five HUD Bar sections (Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel) with fixed positions, same vertical height, and visibility rules. Reviewed existing `control_system.md` HUD Bar section (lines 70-86) and confirmed it already captures exactly what the user described. Presented the two remaining open questions (empty slot visual treatment, width proportions). User chose to record and exit before answering.

**No design changes made**: Existing documentation already matched user's description. Updated insights loose end to note this is the second session where the sub-questions went unanswered.

**No feature_requests produced**: No design changes to propagate.

## 2026-03-25T23:55:00Z — Non-Interactive: Vote to close HUD/UI bugs topic (final)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: `2026-03-25T00-00-00Z-operator-hud-ui-bugs.md` — topic had extensive root cause analysis from software_architect, task_planner, automatic_qa, and task_splitter since my last comment. My feature_request (`designer-fix_hud_ui_bugs.md`) was already sent and is in `done/`. Voted to close — no further design input needed.
**Output**: None (feature_request already sent in prior session).

## 2026-03-28T20:30:00Z — Non-Interactive: Forum pass

**Trigger**: Scheduler launched for open forum topics.
**Forum**: Read `qa_router_patterns_too_broad` topic — pipeline infrastructure issue about QA routing patterns being too broad. Not a design concern. Voted to close.
**Output**: None. No design work needed.

## 2026-03-28T21:00:00Z — Non-Interactive: Forum pass (re-launch)

**Trigger**: Scheduler re-launched; prior close-vote on `qa_router_patterns_too_broad` was cleared by subsequent comments from other agents.
**Forum**: Re-read topic — still purely a pipeline concern, no design implications. Re-voted to close.
**Output**: None.

## 2026-03-28T20:30:00Z — Non-Interactive: Forum pass (no design work)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: Reviewed `2026-03-28T00-00-00Z-automatic_qa-qa_router_patterns_too_broad.md` — QA routing patterns too broad, causing interactive items to be misrouted to automatic_qa. This is a pipeline infrastructure issue (pipeline_builder responsibility), not a design concern. Voted to close.
**Messages**: No pending or stuck messages.
**Output**: None — no design changes or feature_requests needed.

## 2026-03-28T12:00:00Z — Non-Interactive: Vote to close integration test compilation topic

**Trigger**: Scheduler launched for open forum topic.
**Forum**: Reviewed `2026-03-28T12-00-00Z-software_architect-integration-test-compilation-regression.md` — integration/scenario tests broken after entity definitions refactoring (wrong import paths, removed functions). Purely a code issue with no design implications. Software architect already sent an architecture_request. Voted to close.
**Messages**: No pending or stuck messages.
**Output**: None — no design changes or feature_requests needed.

## 2026-03-28T20:30:00Z — Non-Interactive: Re-vote on integration test topic

**Trigger**: Scheduler re-launched — close-vote was cleared by subsequent comments from task_planner and automatic_qa.
**Forum**: Re-voted to close `2026-03-28T12-00-00Z-software_architect-integration-test-compilation-regression.md`. No new design-relevant content since prior review.
**Output**: None.

## 2026-03-28T22:30:00Z — Non-Interactive: Forum pass, no design work

**Trigger**: Scheduler launched for open forum topics.
**Actions**:
- Topic 1 (integration test compilation regression): Already had my close-vote. No action needed.
- Topic 2 (auto_capabilities patterns too broad): Pipeline infrastructure issue with no design implications. Voted to close.
- No pending messages in designer inboxes.
**Outcome**: No design work required. Session complete.

## 2026-03-28T22:30:00Z — Non-Interactive: Forum pass

**Trigger**: Scheduler launched for open forum topics.

**Actions**:
- Reviewed 2 open forum topics
- Topic 1 (integration test compilation regression): Already had my close vote — no action needed.
- Topic 2 (auto_capabilities patterns too broad): Pipeline infrastructure issue, not design-related. Problem well-understood, fix identified (pipeline_builder needs to update qa_router auto_capabilities.txt). Voted to close.

**Produced**: Nothing — no design-relevant work found.

## 2026-03-28T23:00:00Z — Non-Interactive: Forum pass, no design work

**Trigger**: Scheduler launched for open forum topics.

**Forum topics reviewed**:
1. `2026-03-28T12-00-00Z-software_architect-integration-test-compilation-regression.md` — Integration tests broken after entity definitions refactoring. Purely a code/architecture issue (import path updates). No design implications. Voted to close.
2. `2026-03-28T22-00-00Z-automatic_qa-auto-capabilities-patterns-too-broad.md` — QA router auto_capabilities.txt has overly broad patterns causing bounce loops. Pipeline infrastructure issue for pipeline_builder to fix. No design implications. Voted to close.

**Outcome**: No design doc changes or feature_requests needed. Both topics are well-handled by other agents.

## 2026-03-28T18:30:00Z — Non-Interactive: Forum pass, no actionable design work

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic (`auto-capabilities-still-misrouting`) — QA routing config issue, not design-related. Voted to close.
**Messages**: No pending or active messages.
**Outcome**: No design work required. Session complete.

## 2026-03-28T20:45:00Z — Non-Interactive: Forum pass only

**Trigger**: Scheduler launched for open forum topic.
**Forum**: Voted to close `auto-capabilities-still-misrouting` — pipeline infrastructure issue (QA routing patterns too broad), not a design concern. Pipeline_builder needs to act.
**Messages**: No pending or active messages in designer inboxes.
**Outcome**: No design work required. No feature_requests produced.

## 2026-03-28T21:00:00Z — Non-Interactive: Vote to close QA routing topic (repeat)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: `2026-03-28T18-00-00Z-automatic_qa-auto-capabilities-still-misrouting.md` — pipeline routing issue (auto_capabilities.txt too broad). Not a design concern. Voted to close.
**Output**: None. No design work needed.

## 2026-03-28T21:00:00Z — Non-Interactive: Vote to close QA routing topic (repeat)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: Voted to close `2026-03-28T18-00-00Z-automatic_qa-auto-capabilities-still-misrouting.md` — this is a pipeline_builder/operator infrastructure concern (auto_capabilities.txt routing loops), not a design matter. No design action needed.
**Output**: None (no feature_requests produced).

## 2026-03-28T21:00:00Z — Non-Interactive: Forum pass only

**Trigger**: Scheduler launched for open forum topic.
**Forum**: Voted to close `2026-03-28T18-00-00Z-automatic_qa-auto-capabilities-still-misrouting.md` — pipeline infrastructure issue about QA routing config, not a design concern. No design action needed.
**Messages**: No pending messages.
**Outcome**: No feature_requests produced. Clean exit.

## 2026-03-28T19:00:00Z — Non-Interactive Session

- Loaded insights. No urgent forum questions or pending design reviews.
- Read 1 open forum topic: `auto-capabilities-still-misrouting` — pipeline infrastructure issue about QA routing patterns. Not a design concern. Voted to close.
- No pending messages in designer inboxes.
- No actionable work found. Opened forum topic `designer-no-work-available` to flag potential scheduler misconfiguration.

## 2026-03-28T21:00:00Z — Non-Interactive: No design work in forum

**Trigger**: Scheduler launched for open forum topics.
**Forum**: 2 open topics, both pipeline infrastructure (no-work report from previous session, auto_capabilities misrouting). Neither requires design input.
**Actions**: Voted to close both topics.
**Output**: None — no design-relevant work found.

## 2026-03-28T22:00:00Z — Non-Interactive: Forum cleanup

**Trigger**: Scheduler launched for open forum topic missing designer close-vote.
**Forum**: 2 open topics. Already voted on `auto-capabilities-still-misrouting`. Voted to close `designer-no-work-available` (4 agents already agreed it's a scheduler config issue).
**Output**: None — no design-relevant work.

## 2026-03-28T21:00:00Z — Non-Interactive: Vote to close QA routing topic (round 2)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: Read `2026-03-28T18-00-00Z-automatic_qa-auto-capabilities-still-misrouting.md` — pipeline infrastructure issue about overly broad auto_capabilities.txt patterns causing QA item bounce loops. Not a design concern. Voted to close.
**Messages**: No pending or active messages.
**Outcome**: No design work needed. Session complete.

## 2026-03-28T21:00:00Z — Non-Interactive: Forum pass

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic — `auto-capabilities-still-misrouting`. This is the same QA routing issue (auto_capabilities.txt patterns too broad). Purely a pipeline_builder concern with no design implications. Voted to close.
**Output**: No feature_requests produced. No design changes needed.

## 2026-03-28T18:30:00Z — Non-Interactive: Forum pass (no design work)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic — `auto-capabilities-still-misrouting` (automatic_qa escalation about QA router config). Pure pipeline infrastructure issue, no design relevance. Voted to close.
**Output**: None. No feature_requests produced.

## 2026-03-28T22:00:00Z — Non-Interactive: No work found

**Trigger**: Scheduler launch.
**Forum**: 1 open topic (auto_capabilities misrouting) — pipeline infrastructure issue, not design-related. Voted to close.
**Messages**: No pending or active messages.
**Output**: None. No feature_requests produced. Possibly wrongfully started — no design work available.

## 2026-03-28T22:00:00Z — Non-Interactive: Forum pass (no actionable work)

**Trigger**: Scheduler launched for open forum topics.

**Forum**: One topic open — `auto-capabilities-still-misrouting` (QA router config issue). Pipeline infrastructure concern, not design. Voted to close.

**Output**: None. No design-relevant work found.

## 2026-03-28T22:00:00Z — Non-Interactive: Forum pass only

**Trigger**: Scheduler launched for open forum topic.

**Forum**: Read `2026-03-28T18-00-00Z-automatic_qa-auto-capabilities-still-misrouting.md` — QA router misrouting due to overly broad auto_capabilities.txt patterns. This is a pipeline_builder/operator infrastructure issue with no design implications. Voted to close.

**No messages in inboxes. No design work produced.**

## 2026-03-28T20:30:00Z — Non-Interactive: Forum pass, no actionable work

**Trigger**: Scheduler launched (non-interactive mode).
**Forum**: One open topic (`auto-capabilities-critical-misrouting`) — pipeline infrastructure issue about QA routing patterns, not design-related. Voted to close.
**Messages**: No pending or stuck messages.
**Outcome**: No design work needed. Session complete.

## 2026-03-28T22:00:00Z — Non-Interactive: Forum pass (no design work)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic — `auto-capabilities-critical-misrouting` (QA router infinite bounce loop). This is a pipeline infrastructure issue (auto_capabilities.txt patterns too broad), not a design concern. Voted to close.
**Output**: None. No feature_requests produced.

## 2026-03-28T22:30:00Z — Non-Interactive: Forum pass (no design work)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic — `auto-capabilities-must-be-narrowed` (QA router infinite bounce loop, 4th filing of same issue). Pipeline infrastructure issue, not design. Voted to close.
**Output**: None. No feature_requests produced.

## 2026-03-28T23:00:00Z — Non-Interactive: No design work found

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic (`auto-capabilities-must-be-narrowed`) — pipeline infrastructure issue about qa_router bounce loops. Not a design concern. Voted to close.
**Messages**: No pending or active messages.
**Output**: None. No feature_requests produced.

## 2026-03-28T23:30:00Z — Non-Interactive: No design work found

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic (`auto-capabilities-infinite-bounce-loop`) — 5th iteration of the same pipeline infrastructure issue about qa_router auto_capabilities.txt patterns. Not a design concern; owned by pipeline_builder. Voted to close.
**Messages**: No pending or active messages.
**Output**: None. No feature_requests produced.

## 2026-03-28T23:30:00Z — Non-Interactive: Forum pass only

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic — `auto-capabilities-infinite-bounce-loop`. This is a pipeline_builder infrastructure issue (QA routing patterns), not a design concern. Voted to close.
**Output**: No feature_requests produced — no design work needed.

## 2026-03-28T23:45:00Z — Non-Interactive: Forum pass, no actionable work

**Trigger**: Scheduler launch.
**Forum**: 1 open topic (`auto-capabilities-must-be-narrowed`) — pipeline infrastructure issue requiring pipeline_builder, not design. Voted to close.
**Messages**: No pending or active messages.
**Outcome**: No design work needed. Clean exit.

## 2026-03-28T23:45:00Z — Non-Interactive: Vote to close QA routing topic (2nd)

**Trigger**: Scheduler launch.
**Forum**: 1 open topic (`auto-capabilities-must-be-narrowed`) — same pipeline infrastructure issue about broad auto_capabilities.txt patterns causing QA bounce loops. Already has 5 close-votes from other agents. Voted to close.
**Messages**: No pending or active messages.
**Outcome**: No design work needed. Clean exit.

## 2026-03-28T23:45:00Z — Non-Interactive: Forum pass (no design work)

**Trigger**: Scheduler launched for open forum topic.
**Forum**: 1 open topic — `auto-capabilities-must-be-narrowed` (automatic_qa). Pipeline infrastructure issue about QA routing patterns causing bounce loops. Not a design concern. Voted to close.
**Messages**: No pending messages.
**Outcome**: No design work needed this session.

## 2026-03-28T23:45:00Z — Non-Interactive: Vote to close auto_capabilities topic

**Trigger**: Scheduler launched for open forum topic.
**Forum**: `2026-03-28T23-30-00Z-automatic_qa-auto-capabilities-must-be-narrowed.md` — auto_capabilities.txt broad patterns causing infinite QA bounce loop for 6 items. This is a pipeline infrastructure issue (pipeline_builder/software_architect domain), not a design concern. Voted to close.
**Output**: No feature_requests produced — no design relevance.

## 2026-03-28T23:50:00Z — Non-Interactive: Forum pass only

**Trigger**: Scheduler launched for open forum topic.
**Forum**: Voted to close `2026-03-28T23-30-00Z-automatic_qa-auto-capabilities-must-be-narrowed.md` — pipeline infrastructure issue (auto_capabilities.txt broad patterns causing QA bounce loops) already resolved by software_architect. No design relevance.
**Messages**: No pending or active messages.
**Outcome**: No design work needed. Session complete.

## 2026-08-05T00:00:00Z — Interactive: Cults Armory/stats resolution + Syndicate Tunnel Space rework

**Trigger**: Interactive user session.

**Forum/messages check**: No open forum topics missing designer close-vote. No pending messages. Found insights.md was stale — several loose ends listed as unresolved (Cults Armory training queue, Soldier/Gunner/Recruit unit definitions, outside-construction rework) already had matching feature_requests sitting unconsumed in `messages/task_splitter/feature_request/pending/` from a prior session that ended before insights.md was updated. Corrected insights.md to match actual current state before proceeding.

**Cults work**:
- Confirmed Armory training queue behavior (queueable per available Recruit, cancel returns Recruit to Armory + refunds crystals) already matched the pending feature_request — no new edit needed there.
- Filled in Armory's building stats (MaxHP 300/PointArmor 1/FullArmor 6/SightRange 4, matching GDO Barracks).
- Restructured Cults building costs: moved SpaceCrystalsCost off each building (RecruitmentCenter/Storage/Armory) onto a new `Constructs:` list under the Recruit's Building section (mirrors GDO's DeploymentCenter > Constructs pattern), with new BaseConstructionTime scaling rule (time-with-1-Recruit, scales down linearly with more Recruits assigned).
- Filled in RecruitmentCenter (1000/1/16/6, 300 SC/1440 frames) and Storage (250/1/6/3, 100 SC/320 frames) stats and Constructs entries — RecruitmentCenter confirmed buildable (not just a starting structure).
- User explicitly declined to formalize the FullArmor = PointArmor x footprint-area pattern as a rule in entities.md, despite it holding across all existing buildings — treat as informal/coincidental only.
- Sent 2 feature_requests: `cults_armory_stats_and_building_cost_restructure`, `cults_recruitmentcenter_and_storage_stats`.

**GDO/Syndicate gap review**: Confirmed no TBDs remain in gdo_objects.md or syndicate_objects.md, but surfaced 3 real gaps (Syndicate defensive structures referenced but undefined, missing Tier 2/3 Tunnel expansions, only one combat unit per faction) — logged as loose ends, not acted on yet per user's redirect to Syndicate-specific changes.

**Syndicate work**:
- Reworked Tunnel Space from a flat per-tier value (20/30/40) to a formula: (sum of unused tunnel-region grid units across all Tunnels) / 4, capped at 200. Tunnel-region = Tunnel Area minus Tunnel's own 4x4 footprint. Building expansions shrinks Tunnel Space. Overflow allowed (units never removed), same pattern as GDO Power.
- Reduced Tunnel Area sizes from 10x10/12x12/14x14 to 8x8/10x10/12x12 (tiers 1-3) — corrected the still-pending Tunnel Space feature_request's numbers in place to match rather than sending a confusing follow-up.
- Reduced Agent and Guard TunnelSpaceCost from 2 to 1 each.
- Sent 2 feature_requests: `syndicate_tunnel_space_from_unused_area` (later corrected in-place for the size change), `syndicate_unit_tunnel_space_cost_reduction`.

**Insights updated**: TOC entries for cults_objects.md/syndicate_objects.md refreshed, resolved loose ends removed, new gaps and process lessons added (pending-message pre-check habit, designer_notes.md is historical/non-authoritative, in-place correction of unconsumed messages).

**Session ends here** — user requested insights/log update as final step.

## 2026-08-05T01:00:00Z — Interactive: Colonists faction kickoff (Shields mechanic + Prospector)

**Trigger**: Continuation of same interactive user session (see prior entry same day for Cults/Syndicate work).

**Cults follow-up**: Reduced Recruit CarryCapacity from 4 to 2 Space Crystals per load (MiningDuration/DropOffDuration unchanged). Sent feature_request `cults_recruit_carry_capacity_reduction`.

**Colonists resource rename**: Renamed "Conduits" to "Flux" in factions.md (pure rename, no mechanical change; user brainstormed options, picked Flux). Sent feature_request `colonists_rename_conduits_to_flux`.

**Colonists faction kickoff** — first real content for this previously-empty faction:
- New Shields mechanic (combat.md): Colonists-only MaxShields stat on every unit/building, regenerates continuously at 4/second (constant base rate, upgradeable later), absorbs incoming damage at full value (ignoring armor) before any overflow proceeds to HP under normal PointArmor/FullArmor rules.
- New Beacon Deployment mechanic (new file colonists_objects.md): Prospectors deploy directly into a Beacon (unit transforms into the building, 48 frames, HP/Shields linearly interpolate throughout, vulnerable, cancellable until complete then permanent/one-way, no minimum spacing between Beacons — overrides an old to_be_converted.md note).
- Full Prospector unit definition: first unit to use the HoverVehicle base, fully unarmed (unlike every other faction's worker), 36x36/20 HP/10 Shields/0 armor/5 sight, DragMovement stats (TurnRate 45, ForwardAcceleration 0.6, OmniDirectionalAcceleration 0.2, DragRatio 0.1 -> 8 space units/frame top speed), CarryCapacity 8/MiningDuration 32/DropOffDuration 2 frames, full ObjectInterfaceState (Deploy/Move/Stop/Gather/DropOff).
- New Egress Node structure stubbed (crystal-export building distinct from the Beacon — Prospectors do NOT drop off at Beacons). Stats TBD.
- Beacon itself remains a stub (stats TBD).
- Sent feature_requests: `colonists_shields_and_prospector_beacon_deployment`, `colonists_prospector_full_definition`.

**Notable process moment**: While deriving the Prospector's movement stats, formalized for the first time an explicit DragMovement equilibrium formula (MaxSpeed = (ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio) that had never been written down before. Checked it against the existing GDO SupplyChopper's stats and found it implies a MaxSpeed of 10 (never previously stated) — flagged in insights.md for a future check-in with the user rather than silently editing gdo_objects.md.

**Insights updated**: TOC refreshed for combat.md/cults_objects.md/colonists_objects.md, Colonists loose end narrowed (Prospector done, Beacon/Egress Node/Warp In/shield upgrades still open), new insights added about the DragMovement formula and Prospector being the first unarmed worker/first HoverVehicle unit.

**Session ends here** — user explicitly said Colonists work will resume next session.

## 2026-08-06 — Session: Colonists Warp In costs

- Checked forum/open/: empty. Checked messages/designer/*/pending: none. No urgent work found via automated checks.
- Interactive session with user. User asked "did we go over the roboport?" — reviewed colonists_objects.md, confirmed Roboport stats/RallyPoint/Produces are fully stated but ObjectInterfaceState (panel layout) remains TBD. User chose to defer ObjectInterfaceState.
- User then asked whether Warp In cost/time had been defined for Egress Node and Roboport — found they had not (Constructs List section only described the mechanism, no actual numbers for either building).
- User supplied costs/times in seconds; confirmed 16 FPS from scale.md and converted: Egress Node 75 crystals / 320 frames (20s), Roboport 400 crystals / 640 frames (40s).
- Added a Constructs List table to colonists_objects.md under Warp In > Constructs List, matching the existing GDO DeploymentCenter/ExtractionFacility `### Constructs:` list pattern.
- Sent feature_request `add_colonists_warp_in_costs` to task_splitter, including a note that Starting Conditions buildings (the pre-built Egress Node + Roboport) should NOT be charged this cost since they aren't built via the Warp In flow.
- Updated insights.md TOC entry for colonists_objects.md and the Colonists loose-end entry (added note to remember Constructs List rows when adding future buildings).
- Session paused at user's request after this one change.

## 2026-08-07 — Session: Bioport + Colonists first combat units (Legionnaire)

- Checked forum/open/ (empty) and messages/designer inboxes (none) — no urgent work, proceeded straight to regular session.
- User picked up a new Colonists building: Bioport. Gathered full stats (3x3, 250 MaxHP, 15 MaxShields, 1/9 armor, SightRange 3, Groupable true, AABB symmetry), a new exit-side rule (nearest unblocked "A" side to RallyPoint, random among unblocked if no RallyPoint/blocked), and Warp In cost (150 Space Crystals / 480 frames / 30s). Left Produces list and ObjectInterfaceState TBD pending unit design. Sent feature_request `add_colonists_bioport`.
- User then designed the Bioport's product: originally pitched as a single unit with a switchable dual-attack ability, refined through discussion into two distinct Unit entities — Sabre Legionnaire (melee) and Rifle Legionnaire (ranged) — following the existing Cults Soldier/Gunner precedent (shared physical profile, different AttackAttributes).
- Further refined: rather than being independently produced, only Sabre Legionnaire is Bioport-producible; Rifle Legionnaire is reached via a new Weapon Conversion mechanic (Sabre <-> Rifle, 4 frames, free, no HP/Shields interpolation needed since stats are identical, eligibility gated by reusing the existing AttackPhase.Interruptible rules from combat.md rather than new logic).
- Collected full stat blocks for both units (base profile: HeavyInfantry, 36x36, 70 HP, 10 Shields, 0/0 armor, SightRange 5, MaxSpeed 5, BeaconCapacityCost 2 — first unit to use this stat with a real number) and both AttackAttributes sets (Sabre: FullyConnected Melee, 15 dmg, Aim3/Fire3/Cooldown1/Reload5; Rifle: FullyConnected Ranged, 10 dmg, Range 4/MinRange 0, Aim3/Fire2/Cooldown7/Reload4). Bioport production: Sabre Legionnaire only, 100 Space Crystals / 160 frames (10s).
- Wrote all of this into colonists_objects.md: filled in Bioport's Produces list, added `## Sabre Legionnaire`, `## Rifle Legionnaire`, and `## Weapon Conversion` sections. Sent feature_request `add_colonists_legionnaire` with detailed QA instructions.
- Updated insights.md: TOC entry for colonists_objects.md, Colonists loose-ends entry (combat units now exist, but still only one unit line — no Vehicle/Mech/Air), and two new Insights entries (Bioport exit-side rule precedent, Weapon Conversion mechanic precedent + BeaconCapacityCost first use).

## 2026-08-10/11 — Session: GDO tech tree walkthrough (started, paused)

- Checked forum/open/ and messages/designer inboxes: both empty, no urgent work.
- User initiated a full GDO building/tech tree mapping session — going over general description + prerequisites for every GDO building, existing and new.
- Scoped the session: confirmed it covers building-level prerequisites (not a separate research/upgrade tech layer), full pass planned across existing + new buildings.
- Walked through all 5 existing GDO buildings and captured prose descriptions + confirmed prerequisites match current gdo_objects.md exactly (no mechanical changes): DeploymentCenter, PowerPlant, Barracks, ExtractionFacility, SupplyTower.
- No design doc edits made — user deferred folding the richer descriptions into gdo_objects.md until the new buildings are also covered. No feature_request sent this session since nothing new/changed was defined for developers to implement yet.
- Updated insights.md loose ends with a detailed "IN PROGRESS, paused mid-session" entry so the next session picks up exactly where this one left off (continuing into new GDO buildings).

## 2026-09-12 — Session: GDO tech tree walkthrough (resumed and completed building list)

- Checked forum/open/ (empty) and messages/designer inbox (doesn't exist / no pending) — no urgent work, resumed the GDO tech tree walkthrough paused on 2026-08-10/11.
- User provided the full remaining building list in one pass, each with description + prerequisite + constructor: Deployment Center, Extraction Facility, Extraction Plate, Barracks, Generator (renamed from Power Plant), Arsenal (req. Barracks), Radar Station (req. Power Plant, later changed to req. Radar Station's own dependency chain), Career Campus (req. Barracks), Factory (req. Barracks), Defense System (req. Radar Station + Arsenal), Missile Turret (built by Defense System), Supply Tower, Advanced Mechanics (req. Factory), Airfield (req. Advanced Mechanics), Industrial Powerplant (req. Advanced Mechanics), Propulsion Labs (req. Airfield).
- Clarified with user: (1) full find-and-replace rename Power Plant -> Generator, no stat changes; (2) Supply Tower's prerequisite is being redone to require Radar Station instead of Generator; (3) Missile Turret's "shoots air and ground" is deferred for full design later, but confirmed it maps to the already-existing `TargetDomain: Universal` value in combat.md — no new mechanic needed; (4) confirmed this is the complete building list; (5) prerequisites do NOT propagate transitively — each building lists only its direct requirement(s); (6) this pass is stub-only (description + Faction + TechPrerequisite), full stat blocks deferred to future sessions.
- Edited `gdo_objects.md`: renamed PowerPlant -> Generator throughout (entity header, DeploymentCenter Constructs list, DeploymentCenter BuildMenu commands), updated SupplyTower's TechPrerequisite and DeploymentCenter Constructs-list Requires clause from Generator to Radar Station, added Radar Station/Arsenal/Career Campus/Factory/Advanced Mechanics/Airfield/Industrial Powerplant/Propulsion Labs/Defense System to DeploymentCenter's Constructs list (cost/time TBD), and appended 10 new stub sections (Arsenal, Career Campus, Factory, Radar Station, Advanced Mechanics, Airfield, Industrial Powerplant, Propulsion Labs, Defense System, Missile Turret) with description, Faction, Constructed by, and TechPrerequisite only.
- Sent 1 feature_request: `gdo_rename_powerplant_to_generator` — scoped narrowly to the rename only, explicitly telling the developer NOT to swap SupplyTower's build-gating to Radar Station yet since Radar Station has no implementable stats, to avoid the design doc and code silently diverging or breaking the build. No feature_request sent for the 10 stub buildings themselves since they have no stats yet to implement.
- Updated insights.md: TOC entry for gdo_objects.md, replaced the "paused mid-session" GDO tech tree loose end with a full summary of the complete tree + next-session direction (fill in stats one building at a time, watch for Arsenal/Advanced Mechanics introducing GDO's first research/upgrade mechanic which doesn't exist in the docs yet), added a new loose end tracking the SupplyTower prereq design/code divergence until Radar Station is stated, and updated the "only one combat unit per faction" loose end to note GDO's path is now designed (Factory/Airfield) though Syndicate's still isn't.
- Session ended at user's request ("that's all for now").

## 2026-10-06 → 2026-10-18 — Session: Syndicate tech tree walkthrough (started, ended abruptly)

- 2026-10-06: status check only. 2026-10-11: user started Syndicate tech tree session; presented current Syndicate state and asked about prerequisite model (tier only / tier+buildings / same-Tunnel).
- 2026-10-17: user confirmed Tunnel and 3 tiers stay; renamed Headquarters -> Hideout ("primary accommodations for Syndicate infantry", built in Tier 1 Tunnel, starting building).
- 2026-10-18: edited syndicate_objects.md (rename + new description, stats unchanged), updated 3 pending feature_requests in place with the new name, sent feature_request `syndicate_rename_headquarters_to_hideout`. Session ended before insights/log were updated.

## 2026-10-23 — Session: recovery

- User asked where we left off; found the abrupt-ending session in the transcript files, verified the rename landed, and back-filled insights.md (TOC + in-progress loose end) and this log.
- Resumed Syndicate tech tree: user confirmed Tunnel Space + Supplies rules unchanged; chose prerequisite model B (tier per building Tunnel, TechPrerequisite network-wide); prerequisite counts only when fully constructed and on map (GDO: placed); losing a prerequisite only blocks new starts. Edited entities.md (new Tech Prerequisites section), syndicate_objects.md (Expansion Requirements, ExpandMenu text, Hideout TechPrerequisite None), gdo_objects.md (Constructs note). Sent feature_request `tech_prerequisite_rules`.
- Syndicate building list completed (stub-only, written incrementally with user approval): Optics Lab (T1), Chemical Plant (T1), Vault (T1, req. Hideout), Vehicle Assembly (T2, req. Chemical Plant), Forge (T2, req. Vehicle Assembly), Augmentation Chamber (T2, req. Hideout), Aerospace Assembly (T3, req. Forge), Orbital Command (T3, req. Aerospace Assembly), Laser Turret (surface, Agent-built, req. Optics Lab, Universal targeting, Agent-inside construction flow, Agent emerges adjacent). Orbital forces = satellites as HoverCraft Air units. Added Tech Tree Overview table (incl. Tunnel) to top of syndicate_objects.md; Tunnel gained Constructed by/TechPrerequisite lines. No feature_request for stubs (no implementable stats).
- Session ended at user's request ("good enough, we have the tech tree"). Details/stats for GDO and Syndicate stub buildings deferred to future sessions. Feature_requests sent this session: `tech_prerequisite_rules`.

## 2026-11-15 — Session: Cults tech tree (in progress, checkpoint written)

- Forum/open empty; no inbox work. User asked where we left off: recapped Syndicate (Oct 23–24) and GDO (Sep) tech tree sessions, listed loose ends, and flagged 22 unconsumed designer feature_requests in task_splitter pending (oldest July). User said they are **no longer running the pipeline** — design docs only from now on; no feature_requests sent this session (recorded in insights > Pipeline Status).
- Cults tech tree session. Decisions: every Cults unit (incl. vehicles) comes from Recruits; vehicles consume multiple Recruits (Unit Control = Recruits consumed); Recruits build every Cults building; RecruitmentCenter renamed RecruitmentHall; RecruitmentHall/Storage no prereq, Armory req. Storage; tree splits into Tech/Weaponry (Armory = T1) and Mystical/Human Enhancement branches, both buildable in one game; "tier" is descriptive only.
- Altar (Mystical T1, req. Storage): per-unit Augmentations Enhanced Constitution / Improved Metabolism for all 4 Cults infantry (Recruit, Soldier, Gunner, spell caster). Increment system: Constitution increment = +10 MaxHP, +1 melee dmg (unarmed only); Metabolism increment = move speed + attack speed (unarmed only), values TBD. Each Augmentation = +2 own / −1 other increment. Survives Armory training (order-independent). Armory-like pool + serial queue, auto-exit to RallyPoint, 3-stage AugmentMenu (unit type → current state → augmentation, each click queues one). Fully augmented units can enter but only leave (Eject All, designer default).
- Temple (Mystical T2, req. Altar): research; first set = Constitution L1/L2 and Metabolism L1/L2 lines (+4/−2, +6/−3). Level locked when augmentation finishes; no re-augmenting.
- Research defined as shared rules in entities.md > Research (once per player, cost on queue, serial queue, no duplicates, levels chained, cancel/destruction = full refund, permanent) with Faction Exceptions; Cults exception: stat upgrades not retroactive.
- Files edited: cults_objects.md (rename, Tech Tree Overview, TechPrerequisite lines, Augmentation/Altar/Temple sections, Constructs list, Recruit Enter Altar), factions.md (rename, Recruits-build-all + vehicles sentence), entities.md (Research section).
- After the checkpoint the user added: Cults research requires Recruits inside the building with speed scaling by Recruit count; Cathedral is the last Mystical-branch building. Captured in insights, not yet in docs — follow-ups pending.
- Continued after checkpoint: user confirmed the harsh negative increments are intentional. Defined Cults research staffing (Recruits only; BaseResearchTime with 1 Recruit; linear scaling; not consumed; pauses when empty) as a second Cults exception in entities.md > Research. Added Cathedral (Mystical T3, last, req. Temple; trains Acolyte from 1 Recruit Armory-style; Augmentation Level 2 research at the Temple now also requires owning a Cathedral) and an Acolyte stub (spell caster) to cults_objects.md; renamed "spell caster" placeholders to Acolyte; added Enter Temple/Enter Cathedral to Recruit commands; Constructs list gained Cathedral. Mystical branch building list is complete; Tech/Weaponry T2+ is next.
- Added Cults 'Evacuation on Destruction' rule (cults_objects.md > Cults Building Mechanics): units inside any destroyed Cults building exit alive and unchanged around the footprint; all in-progress/queued training, augmentation, research orders cancelled with full refund.
- 2026-11-16: Added Shrine (Mystical T3, sibling of Cathedral, req. Temple): Recruit-only ritual participation drives energy regen; two area abilities castable anywhere with vision — own-infantry health regen buff and own-Acolyte energy regen buff, snapshot at cast time; numbers TBD. Acolyte gained a naturally-regenerating energy pool. Recorded that Cults infantry have no natural HP regen. Evacuation rule extended to Shrine. Constructs list + Recruit Enter Shrine command added.
- User clarified: design-only from here; game will be rebuilt from scratch once the design is complete. Updated insights Pipeline Status; dropped the SupplyTower design/code-divergence loose end (moot); added a future 'design-doc completeness audit' loose end.
- 2026-11-16: Cults Tech/Weaponry branch written: Gunworks (T2, req. Armory; unlocks Gunner at Armory + weapon upgrade research), Chop Shop (T2 major, req. Armory; vehicles), Machine Shop (T2, req. Chop Shop; vehicle+air upgrades), Airstrip (T3 major, req. Chop Shop; aircraft), Scrap Hangar (T3, req. Airstrip; unlocks large paratrooper plane + airship at Airstrip). Armory Gunner now gated by Gunworks. entities.md: new Unit Prerequisites subsection. Overview gained tree diagram + informal-tier rule (tier anchored by unit-producing building). Syndicate Augmentation Chamber flagged for rework after Colonists tech tree.
- 2026-11-16: Colonists economy/tech model written. User decided Colonists have NO building tech tree (any building Warped In anytime), units may have Unit Prerequisites, gating is economic via refined-resource costs. One-Drone-per-patch (any player blocks, Colonists only; seek nearest free patch else wait). Drone jobs = pick up at X / deliver to Y (feeders vs haulers separate), AssignedEgressNode generalized to AssignedDropOff (Egress Node or refinery), CarryCapacity 8 all resources. Refineries: Alloy Furnace, Essence Refinery (1:1), Flux Reactor (1A+1E->2F, separate entrances); max input/output storage, output-full pauses, input-full refuses + feeder waits + notification; specific input/output sides. Files: colonists_objects.md, factions.md, entities.md. 'Notifications' (not alerts) logged as a future topic.

## 2026-11-17 — Session continued: Colonists buildings and portals (in progress)

- Colonists additions decided in conversation: Starport (air units), second Bioport unit Lancer (energy-missile salvo vs ground+air, needs Exophysics Engine), Exophysics Engine (upgrades; unlocks Lancer + Skylance), Skylance (anti-air defensive structure). Defensive structures are an exception to the no-building-prerequisites rule (anti tower rush). Costs deferred until the building list is done.
- Portal rework: "-port" buildings are portals from the homeworld; renamed to two-word names (Robo Portal, Bio Portal, Star Portal) plus new Heavy Portal (vehicles: hover, heavy = tracked). Bio Portal -> Heavy Portal -> Star Portal is a multi-stage construction (upgrade in place only, each stage replaces the previous stage's production). User decided the Robo Portal should NOT upgrade into the Bio Portal, citing a Colonists-vs-Colonists scouting problem and asking the model to work out the reason; several guesses rejected; the final analysis (permanent scouts because nothing can attack pre-Bio Portal; public all-or-nothing upgrade ending Drone production and Beacon supply; second-mover advantage -> standoff) was received with "Good stuff. That's worth recording."
- Written to docs: colonists_objects.md (rename throughout; Tech Model + Warp In exception notes; new Portals section with upgrade line, standalone Robo Portal, Rationale, Open Questions; Heavy Portal and Star Portal stubs; Robo/Bio Portal descriptions), factions.md (upgrade-line exception note).
- NOT yet written (awaiting go-ahead): Exophysics Engine, Lancer, Skylance, defensive-structure exception in Tech Model. Open: Heavy/Star footprint, Bio Portal cost, upgrade cost/time, whether the recorded rationale matches the user's own original reason.
- No feature_requests sent (pipeline not running; design-only).
- User gave their own reason for the standalone Robo Portal: nobody would build a Bio Portal, players would keep adding Robo Portals with nothing to defend against, economy goes exponential and the low-economy early game is skipped. Added as the core problem at the top of the Rationale in colonists_objects.md > Portals.
- Wrote Exophysics Engine, Skylance, Lancer stubs and the defensive-structure prerequisite exception into colonists_objects.md.

## 2026-11-18 (continued)
- Named and wrote the Colonists anti-ground defense pair: Power Core (power-generation tech building, name chosen from power-themed options) unlocks Plasma Turret (energy beam; briefly 'Plasma Cannon'). Stubs + Constructs List rows + Tech Model example updated in colonists_objects.md. Unit unlock, upgrades, and beam mechanic left TBD. No feature_requests sent (design-only).

## 2026-11-19
- Colonists comms buildings: named Relay Array (surface-to-surface; unlocks flying Scout at Robo Portal) and Navigation Spire (lighthouse-like; unlocks large flying units). Recorded user's Operability Principle in Tech Model with table of tech buildings. Scout stub written (name/stealth TBD). Constructs rows, Robo Portal Produces, Portals table, Star Portal note updated. No feature_requests.
- Renamed Power Core → Fusion Core throughout colonists_objects.md and insights.
- Wrote Psionic Amplifier stub (Operability table row, Constructs row, Bio Portal Produces line, Star Portal tentative note). Discussed Fusion Core research (shields + energy weapons; missiles stay in Exophysics Engine) and armor-upgrade placement options — not yet written.
- Wrote Structural Analyzer (armor upgrades; gates treaded vehicles at Heavy Portal; rationale for keeping refineries economic) and the research layout: Exophysics Engine missiles, Fusion Core shields + energy weapons.
- User declared Colonists building list complete. Recorded Upgrade Layout in Tech Model; Fusion Core weapon line renamed to beam/particle; Structural Analyzer armor = infantry/vehicle/flying lines; refineries research nothing.
- Renamed Lancer/Skylance missiles to antimatter missiles; Legionnaire weapons designated energy weapons (Fusion Core upgrades).
- Clarified: Fusion Core / Exophysics Engine / Structural Analyzer upgrades are incremental +N stat upgrades (multi-level lines); recorded in Tech Model > Upgrade Layout.
- Fusion Core shield upgrades = faster ShieldRegenRate (combat.md cross-ref updated).
- Shield regen upgrades apply to units and buildings.
- No armor upgrades for Colonists buildings.
- Confirmed: hover vehicles need nothing beyond Heavy Portal; Colonists have no Research exceptions (noted in entities.md). Moving to Syndicate Augmentation Chamber.
- Syndicate lore expanded in lore.md (GDO view vs reality; small trust-limited groups; imprinting paranoia). Augmentation Chamber description rewritten as a Faraday cage against imprinting; gameplay role TBD.
- Renamed Syndicate Augmentation Chamber → Null Chamber. User framing: Syndicate heavy infantry = exoskeleton augmentation (mechanical); Null Chamber consumption effects are Spice-like (Dune), distinct from Cult morphological augmentation.
- Stealth/detection added to loose ends (deferred to unit design). Null Chamber exit-buff idea discussed, not written.
- Wrote Null Chamber exit buff to syndicate_objects.md (manned units, detection/sight/%DR before armor, instant refresh; numbers & research gating TBD). Logged % damage-reduction ordering/stacking as loose end.
- Ran design-doc consistency audit across all docs. Fixed A1–A7 (user-approved: Armory ABAC; Enter generalized for Cults; camera starting structures incl. Colonists Robo Portal). Recorded B1–B13 open gaps in insights.
- B1 resolved: MinimumDamage section in combat.md (0.5 per hit; AoE 0.5×overlap fraction; fractional HP; minimum capped at shield overflow).
- B2 resolved: DirectionalArmor zones/multipliers in combat.md.
- B3 resolved: Crushing rule in units.md.
- B4 resolved: DamageModifierOrder section in combat.md.
- B5: Starting Conditions written for all four factions (resources TBD).
- Extraction Plate MiningRate doubled to 20 per 48 frames (user).
- Extraction Plate ResidualMiningRate doubled to 2 per 48 frames.
- Reverted Extraction Plate buff (back to 10 / residual 1 per 48 frames). Mining-rate balance (Drone/Recruit nerf) left open.
- Syndicate starts with 1 Supply.
- Cults start with 0 Space Crystals.

## 2026-11-20 — GDO opening economy tuning
- Simulated GDO opening (non-stop Generators + plates, power-aware) to tune Generator cost/time and starting crystals; analysed idle-DC vs cut-plate tension and its dependence on patches per base.
- User decisions: Generator 100 SC / 15 s; plates unchanged (payback between Agent and Drone); cutting a plate should be the acceleration option; patch count is a map-maker dial.
- gdo_objects.md: Generator cost 150→100, time 160→240 frames (build list + DC command); Starting resources 450 SC (Supplies TBD); new "Opening Economy Baseline" subsection (rationale, 8-patch reference base, early power pinch TBD).
- insights.md: B5 status, GDO opening entry, cross-faction yardsticks loose end, economy-tuning insight. No feature_requests sent (per user).
- Deployment Center Power +20 → +30 (removes the early power pinch; start = +12, inert opening never negative). 450 starting crystals re-verified; Barracks idle/cut figures updated (38 s / 35 s, ~20 SC).
- Revised GDO start after user concern about an instant Barracks before scouting: 3 starting Extraction Plates + 100 SC (replacing 1 plate + 450 SC). Simulated three build orders (DC-optimal, EF-optimal, fastest clean Barracks) and the user's own plates-first order; opening is forgiving, power (Barracks −30) gates the Barracks behind 2 Generators. Rewrote gdo_objects.md Starting Conditions + Opening Economy Baseline. New loose end: user worried GDO economy scales too fast.
- Plate build-time discussion (GDO vs Colonists ramp): compared 6–24 s plates; Colonists 8-patch-row saturation ≈ 18–22 working Drones (+2–3 Beacons). Undecided on plate time (user leaning longer than 12 s). Fixed stale Drone patch-saturation line in colonists_objects.md (was "3–6 Drones", from an older multi-Drone-per-patch idea).
- Drone BeaconCapacityCost = 1 recorded in colonists_objects.md (closes half of B6).
- GDO starting plates 3 → 2 (even start vs Colonists' 4 Drones). Re-simulated opening (6 s plates): clean Barracks ~43–45 s, all-in ~15 s at −21 power. Added gdo_objects.md > Economy Targets (GDO vs Colonists) with the user's 4 targets + reference check. Plate/Drone build times still open.
- Extraction Plate build time 6 s → 25 s (400 frames). Re-simulated GDO opening (tight first ~30–40 s, plates-first best, Generator-first −8%, clean Barracks ~46 s, fully plated ~2:30, DC becomes bottleneck/float from ~1:00). Added income comparison table vs Colonists (target 1 roughly met). Open: 2nd Extraction Facility on the same base; tech building costs ~20–30 SC/s of build time.
- Documented second-Extraction-Facility break-even (≈10 patches) in gdo_objects.md; user confirmed EFs plate anywhere in build area. User satisfied with Colonists–GDO crystal balance; moving on to Syndicate economy.
- Syndicate: documented Tunnel crystal-intake cap (~16.7 SC/s per Tunnel) and that stacking two T1 Tunnels on one field is allowed (no upgrades + concentrated Supply risk; Syndicate rewarded for spreading Tunnels). syndicate_objects.md > Tunnel Area Non-Overlap Rule.
- Syndicate: Agent crystal drop-off 3 s → 4 s (Tunnel ceiling 16.7 → 12.5 SC/s, 2–3 Agents to saturate); added distance/saturation table to syndicate_objects.md > Agent > Gathering; updated Tunnel intake note.
- Agent crystal MiningDuration 3 s → 4 s; saturation table regenerated (2–3 Agents up to 3 grid units; ceiling still 12.5 SC/s).
- Agent crystal MiningDuration 4 s → 5 s (80 frames) to reward Tunnels close to crystals; Agent build time 10 s → 15 s (240 frames). Saturation table + rationale regenerated.
- Starting Space Crystals: Syndicate 100, Colonists 100 (GDO already 100, Cults 0). Colonists refined resources + GDO Supplies still TBD.
- GDO starting Supplies deferred until GDO Supply costs are designed (noted in gdo_objects.md + insights).
- B7 resolved: any number of Agents / Recruits may mine the same patch simultaneously (added to syndicate_objects.md Agent Gathering and cults_objects.md Recruit Gathering).
- B6 resolved: Syndicate production checks Tunnel Space when a queued unit is about to start (queue needs only crystals); blocked queue waits, auto-resumes, cancel = full refund, raises notification; all Syndicate production buildings. syndicate_objects.md (Tunnel Space > Production and Tunnel Space, Hideout commands), factions.md SyndicateResources.
- B8 resolved: buildings (own hall footprint, other Cults buildings, enemy buildings) never affect tile recruitability; only claiming by another Recruitment Hall does. cults_objects.md > Recruitment Hall > Effectiveness.
- B9: Supply Chopper Cargo section added (4 infantry / 1 vehicle / Supplies, one at a time); transport mechanics deferred to unit design.
- B10 resolved via Laser Turret rework (pop-up inside own Tunnel Areas, upright outside; Agent or Tunnel built; 1x1; conversion on Tunnel build/upgrade/destroy; Agent Build Laser Turret command, hotkey TBD). Tunnel Space formula now floor(unused/4) in syndicate_objects.md + factions.md.
- B11 resolved: new Cults Garrison (Tech/Weaponry T2, req. Armory; Recruits garrison and fire with Soldier's attack; evacuation applies). Added to overview table, tree, evacuation list, control_system.md Enter list. Capacity/other infantry/targetability TBD.
- Garrison: Recruits + Soldiers + Gunners may enter (not Acolytes), capacity 4, +1 grid unit Range for units inside.
- control_system.md: BasicCombatUnit right-click own Garrison (Soldier, Gunner) → Enter.
- Garrison: Acolytes may also enter, firing as Soldiers (like Recruits); Acolyte spellcasting from inside TBD.
- B12 resolved: syndicate_objects.md > Tunnel Network > Losing Tunnels (last Tunnel lost = defeat; stranded units destroyed by tier). Garrison: units inside untargetable / no damage. Noted gap: no defeat conditions for other factions.
- Defeat Condition (factions.md): lose all buildings, every faction; Syndicate expansions turn neutral when their Tunnel dies (syndicate_objects.md Losing Tunnels updated). B13 resolved: Beacon placement section in colonists_objects.md (Worker-Built Structures rules, fog-of-war orders allowed); entities.md example updated.
- Defeat: every structure counts for all factions; Syndicate Laser Turrets survive Tunnel loss (losing last Tunnel ≠ defeat if turrets remain). Neutral expansions: visible, slow HP decay, claimed by owner of any Tunnel built over them. factions.md + syndicate_objects.md updated.
- Defeat: every structure counts for all factions; Syndicate Laser Turrets survive Tunnel loss (losing last Tunnel ≠ defeat if turrets remain). Neutral expansions: visible, slow HP decay, claimed by owner of any Tunnel built over them. factions.md + syndicate_objects.md updated.
- Neutral expansions: claimed only if fully covered and Tunnel tier ≥ Tier Requirement; in-progress production/research destroyed.
- Neutral expansion claimed when covering Tunnel later upgrades enough. Colonists start Alloys/Essence/Flux = 0. Garrison + Laser Turret specifics deferred. Starting Cults economy discussion.
- 2026-11-22: Decided: queued-but-not-started items in a destroyed Tunnel's expansions are fully refunded (syndicate_objects.md > Losing Tunnels). Cults economy discussion opened (Recruit distance table presented).
- 2026-11-22: Cults BaseRecruitProductionTime 12 s → 9 s (144 frames); 50% example now 18 s; rationale added (cults_objects.md > RecruitmentHall). Clarified: trained units still count against originating hall capacity (already in docs).
- 2026-11-22: Added two rationale paragraphs to cults_objects.md > BaseConstructionTime Scaling: Recruit-time as the Cults' core cost (walking distance makes fast building costlier), and crystal cost vs build time as the dial balancing Cults flexibility. Recruitment Hall unchanged (300 SC / 90 s).
- 2026-11-22: Recruit Gathering: added MiningReach and DropOffReach (8 su each) and the expected-throughput note (1-tile gap → ~1.45 SC/s per Recruit).
- 2026-11-22: Storage resized 3x2 ABAB → 2x2 AAAA, cost unchanged (100 SC / 20 s); rationale added (cults_objects.md > Storage).
- 2026-11-22: Kept Recruit production at 9 s after reviewing 9/10/12 s; added note to rationale that it's a playtest dial (9–12 s) and that early 2nd-hall commitment is intended.
- 2026-11-22: Simulated 5-min pure mining (GDO/Colonists/Cults) and earliest 2nd hall (with walking). Reverted Recruit production 9 s → 12 s (192 frames); rationale rewritten with sim findings (cults_objects.md > RecruitmentHall).
- 2026-11-22: Cults Build Command: cost deducted when the build order is given (cults_objects.md > Build Command).
- 2026-11-22: Cults Build Command: full refund if the order is cancelled before construction starts; after start only the building's Cancel Construction (75%).
- 2026-11-22: Added shared WorkerReach 8 su (entities.md > SpaceCrystalsPatch) for mining/pick-up/drop-off, referenced from Drone, Agent, Recruit. Added Cults map-design guideline: balance from a fully Recruitable map, map makers tone down (cults_objects.md > RecruitmentHall).
- 2026-11-22: Correction per user: reach is per-unit, not universal. Removed shared WorkerReach from entities.md; MiningReach/DropOffReach 8 su (placeholder) on Drone, Agent, Recruit.
- 2026-11-22: Restructured design docs into directories (core/, systems/, systems/controls/, factions/{gdo,syndicate,cults,colonists}/, archive/) per user; added design/README.md index; rewrote cross-file refs to path form; archived designer_notes.md + to_be_converted.md; deleted empty design_questions.md. Backup of pre-split docs in session scratchpad. Insights TOC replaced with pointer to README + old→new map. Flagged: other agents' prompts glob design/*.md.
- 2026-11-22: With user approval, updated design-doc path glob to design/**/*.md (+ README index) in all .claude/agents/*.md and CLAUDE.md. User chose to design unit rosters one faction at a time.
- 2026-11-22: Renamed GDO 'Global Defense Ordinance' → 'Global Defense Organization' (GlobalDefenseOrganization) across factions/gdo/*.md to match lore; marked working name.
- 2026-11-22: Expanded GDO lore in core/lore.md (treaty org with military/policing/licensing branches, conflicting goals, corruption/black market, field forces mostly licensed private paramilitaries, standardized approved equipment, player identity per scenario, Peacekeeper = sanctioned external operator, partial misnomer). Added lore line to Peacekeeper.
- 2026-11-22: Peacekeeper: look (corporate private security) + role intent (early all-rounder, may be slightly redundant late) added to factions/gdo/units.md.
- 2026-11-22: Peacekeeper Riot Shield toggle ability added (factions/gdo/units.md; half speed, cannot attack, 50% pre-armor damage reduction); listed in systems/combat.md DamageModifierOrder step 1.
- 2026-11-23: Riot Shield details: all directions, toggle time (<0.5 s, TBD), must lower shield to attack, no auto-target while raised. Peacekeeper ground-only confirmed (no early air units).
- 2026-11-23: Riot Shield: right-click on enemy while raised issues Move.
- 2026-11-23: Added Rocketman (GDO missile infantry) to factions/gdo/units.md with Target Lock ability; Barracks Produces + W command (tentative) + Arsenal unlock note in structures.md; README updated.
- 2026-11-23: Added Drone Operator (GDO, Barracks + Career Campus) to factions/gdo/units.md; Barracks Produces line; Career Campus unlock note; README updated.
- 2026-11-23: Drone Operator: drone Air-domain (anti-air only), one-use melee attack = explosion, no recall, operator unarmed.
- 2026-11-23: Drone Operator drone explosion targets Ground only.
- 2026-11-23: Added Field Engineer skeleton (GDO, Barracks + Career Campus): Repair buildings, Build Bunker, Set up First Aid Tent.
- 2026-11-23: Field Engineer Repair also covers vehicles.
- 2026-11-23: Field Engineer abilities detailed (all cost SC, unarmed); Bunker + First Aid Tent structure sections added to factions/gdo/structures.md; README updated.
- 2026-11-23: Bunker garrison rules (all GDO infantry, like Cults Garrison: untargetable, evacuate alive; own weapons, no range bonus); First Aid Tent placement any Buildable; Cults Garrison +1 range limited to Soldiers/Gunners; Enter lists + right-click updated in systems/controls.
- 2026-11-23: Bunker: units inside cannot use abilities.
- 2026-11-23 GDO Missile Vehicle (name TBD): Factory, WheeledVehicle, narrow-arc turret, homing (HeadDisjointed) Universal missiles, no transport. Added to units.md + Factory Produces.
- 2026-11-23: Missile Vehicle turret TurnAngle 60° full arc (±30°), placeholder.
- 2026-11-23: Missile Vehicle named Raptor.
- 2026-11-23: Raptor role note: main difference from Rocketman is speed (Raptor very fast).
- 2026-11-23: Added AA Minelayer vehicle skeleton (Factory; slower than Raptor; 360° turret, FullyConnected machine gun, Air only; lays hidden underground mines).
- 2026-11-23: AA minelayer named Dust Devil; mine mechanics written (buy one at a time, lay anywhere w/ brief stop, hidden, activation radius + short delay, hits infantry+vehicles); wheeled.
- 2026-11-23: Dust Devil mines max 3, starts with 3. Drone Operator drone stock max 3 → 1.
- 2026-11-23: Dust Devil mines: friendly fire (enemy-triggered only), ground-only (air unaffected), destroyable by enemies.
- 2026-11-23: Added Artillery Tank (name TBD) to GDO units (Factory, tracked, deploy-to-fire Ground AoE w/ friendly fire + MinRange).
- 2026-11-23: Artillery Tank deployed turret 360°.
- 2026-11-23: Dust Devil renamed Twister; Artillery Tank named Artillery Rig; Rig AttackType TailDisjointed.
- 2026-11-23: Added Prospector (Factory+AM, tracked) and Outpost structure (build radius only, uses power, permanent, any Buildable); GDOBuildArea seeded by Outposts; AM unlock note; README.
- 2026-11-23: Bunker + First Aid Tent BuildRadiusExtension 0; noted in GDOBuildArea.

## 2026-11-23 — Development process reorganization
- User: pipeline unnecessary now; wants a long-running implementation lead agent (delegates to sub-agents) + designer session, with designer notifying it of design changes and its design questions bubbling back to the designer (technical decisions stay with it). Designer commits own doc changes.
- Committed + pushed all design work (dd2b7e6: directory restructure, Cults tuning, GDO lore/roster); push also carried 5 earlier local pipeline-framework commits. Only artifacts/designer/ committed — agent prompts/CLAUDE.md have mixed uncommitted pipeline edits, left alone.
- Rewrote .claude/agents/designer.md for the new model (change entries, question intake, committing; forum/feature_request/non-interactive parts removed). Implementer name is a {LEAD} placeholder pending user's pick. Insights: Pipeline Status/forum sections replaced with Development Process + Open {LEAD} Questions.
- Implementation agent named **Director** (user's pick); placeholder replaced; designer.md committed.
- Pipeline removal + new setup (user: "do all this and get rid of the old pipeline", "also get rid of the old telegram chatbot integration"): snapshot commit fafac0b tagged `pipeline-archive`; git rm'd forum, forum_archive, messages, logs, old scripts/templates, framework.md, pipeline.yaml, telegram_bot/, .claude/agent-memory, .claude/projects, 8 old agent prompts, 9 pipeline agent artifact dirs. Untracked/ignored leftovers (16G artifacts/manual_qa/qa_artifacts, run_bot.sh, telegram_bot.yaml, .secrets, run_qa.log, .scheduler.lock, .locks) — deletion blocked by permission classifier; left for user. Created Director prompt (.claude/agents/director.md, full draft for user to revise), artifacts/director skeleton (insights, plan, log, questions/, state/), hook script + per-agent settings + launch scripts, rewrote CLAUDE.md. Hook behavior verified with headless runs.
- User deleted leftover pipeline/bot files and revoked the Telegram bots. Removed empty forum/ and messages/ dirs; pruned obsolete .gitignore entries.
- Pushed master (dd2b7e6..8084741) and tag pipeline-archive to origin.
- Session close: added 'WHERE WE LEFT OFF' block to insights (GDO roster status, pending Advanced Mechanics/Airfield question, open GDO items, process notes); annotated stale Colonists 'Prospector' note (now Drone).

## 2026-11-23T12:32:23-05:00 — Director live; Artillery Rig gate; first Director questions
- Director started (Cargo workspace scaffolded). Took in Q0001–Q0005; last_seen_question = 5.
- Decided (user): Artillery Rig requires Advanced Mechanics (resolves the "what else does Advanced Mechanics unlock" thread). Docs: factions/gdo/units.md, factions/gdo/structures.md. Change entry 0001.
- Q0005 answered from the docs: hotkey letters authoritative; position words fixed; V has no standard function. Doc: systems/controls/selection_and_hud.md. Change entry 0002.
- Q0001–Q0004 open, need user.
- 2026-11-23T13:00:03-05:00 — Q0001 partly answered with user: square grid, 3D-rendered sprites, all building orientations, facings 16/32, 32 px/tile. New doc core/art_direction.md (+README). Change entry 0003. Elevation display in discussion (AoE2-like slopes).
- 2026-11-23T13:11:54-05:00 — Q0001 completed: elevation = corner heights (AoE2-like slopes), ≤1 level per Traversible tile, cliffs for big drops/chokepoints, elevation level = floor, flat-only building. Docs: core/map.md, vision.md, entities.md, art_direction.md, README. Change entry 0004.
- 2026-11-23T13:17:51-05:00 — Underground expansions exempt from flat-only rule; ElevationModifier now applies to structures. Docs: core/map.md, entities.md, vision.md. Change entry 0005.
- 2026-11-23T13:20:55-05:00 — Slopes: no movement-speed effect (user). Doc: core/map.md. Change entry 0006.
- 2026-11-23T13:23:28-05:00 — Q0003 answered: patch StartingAmount per patch (default 1500); stations fully editor-set (StartingSupplies 0 / DeliverySize 4 / Interval 60 s defaults); test map starts in corners, test maps are Director's. Doc: core/map.md. Change entry 0007.
- 2026-11-23T13:26:08-05:00 — Q0004 partly answered: per-faction Starting Layouts placed by map maker with overlay editing; symmetry tools (90/180 rotation, mirrors). New core/map_editor.md (+README), pointers in faction Starting Conditions. Change entry 0008. Middle-mouse pan decided, not yet written.
- 2026-11-23T13:28:34-05:00 — Starting Layouts start empty; Skirmish Default restriction (user's name) limits layout to Starting Conditions objects, can be turned off. Doc: core/map_editor.md. Change entry 0009.
- 2026-11-23T13:29:12-05:00 — Middle-mouse grab-and-drag panning (user). Doc: core/camera.md. Change entry 0010.
- 2026-11-23T13:38:36-05:00 — Q0002 completed: default bindings confirmed (map clicks, control-group keys, arrows/edge pan, Shift queue), rebinding later at Director's discretion. Docs: selection_and_hud.md, commands.md, camera.md. Change entry 0011.
- 2026-11-23T13:39:17-05:00 — Q0004 completed: map creation settings, Skirmish Default vs Custom map types, symmetry chosen at creation with generated mirror, complete restricted layouts, outdated flag, neutral-only pre-placement, custom tile presets without custom textures for now. core/map_editor.md rewritten; map.md; README. Change entry 0012 (supersedes 0009 toggle).
- 2026-11-23T13:42:21-05:00 — Q0006 answered: patches and Supply stations flat-only (option C). Doc: core/map.md. Change entry 0013.
- 2026-11-23T13:58:37-05:00 — GDO Airfield: limited ammo, 4 aircraft per Airfield, Air-to-Ground Hovercraft + Air-to-Air Fighter (req. Propulsion Labs). Docs: gdo/structures.md, gdo/units.md, README. Change entry 0014.
- 2026-11-23T14:07:42-05:00 — GDO aircraft assignment rules + fighter Glider w/ 5 missiles. Docs: gdo/structures.md, gdo/units.md. Change entry 0015.
- 2026-11-23T14:09:46-05:00 — Ammo correction: Fighter 1 missile, Hovercraft 5. Change entry 0016 (corrects 0015).
- 2026-11-23T14:12:24-05:00 — GDO aircraft return/rearm flow, landing slots, reassign input, no auto-reassign. Doc: gdo/structures.md. Change entry 0017.
- 2026-11-23T14:49:02-05:00 — Landed GDO aircraft are ground targets. Doc: gdo/structures.md. Change entry 0018.
- 2026-11-23T14:54:17-05:00 — Rearm time per unit of ammo, per aircraft type. Docs: gdo/structures.md, gdo/units.md. Change entry 0019.
- 2026-11-23T14:59:19-05:00 — Named GDO aircraft: Hornet (AG hover chopper), Dragonfly (AA glider). Change entry 0020.
- 2026-11-23T15:01:20-05:00 — Artillery Rig name confirmed; fixed leftover working title in Propulsion Labs (now Dragonfly). No change entry (no behavior change; 0020 covers the rename).
- 2026-11-23T17:57:16-05:00 — Q0007 part 1: magic box group destinations (user). Doc: commands.md (+README). Change entry 0021.
- 2026-11-23T18:26:59-05:00 — Q0012 + Q0007 answered: own idle units sidestep for blocked movers (B); no swap for movers. Doc: systems/units.md. Change entry 0022. Took in Q0014.
- 2026-11-23T18:34:06-05:00 — Who yields: idle only; Hold/busy no; Riot Shield mode irrelevant; deployed Artillery Rig = fixed obstacle. Doc: systems/units.md. Change entry 0023.
- 2026-11-23T18:42:44-05:00 — Q0008 part 1: stuck/give-up timings kept. Doc: behaviors.md. Change entry 0024.
- 2026-11-23T18:49:50-05:00 — Q0008 part 2: unreachable target → nearest reachable point. Docs: behaviors.md, commands.md. Change entry 0025. Took in Q0015.
- 2026-11-23T18:51:23-05:00 — Q0008 part 3: occupied target arrived within 2 GU. Doc: behaviors.md. Change entry 0026.
- 2026-11-23T18:52:17-05:00 — Q0015.1: arrived units don't yield to same-group arrivals. Doc: units.md. Change entry 0027.
- 2026-11-23T18:53:22-05:00 — Q0015.2: magic box around silhouettes. Doc: commands.md. Change entry 0028.
- 2026-11-23T18:56:56-05:00 — Q0015.3: queued group orders use planned positions for the magic box (+ fallback). Doc: commands.md. Change entry 0029. Took in Q0016.
- 2026-11-23T19:02:16-05:00 — Clarified: group destinations resolved per unit at issue time (no recompute at dequeue). Doc: commands.md. Change entry 0030.
- 2026-11-23T19:04:35-05:00 — Q0009.1: range edge to edge. Doc: combat.md. Change entry 0031.
- 2026-11-23T19:08:12-05:00 — Q0009 proximity: 8 su; Move to unit = Follow; 2-GU blocked rule for followed units. Docs: behaviors.md, commands.md. Change entry 0032.
- 2026-11-23T19:11:01-05:00 — Q0009.2: silhouettes rotate (B); 2-GU rule for static targets; follow-end confirmed. Docs: units.md, behaviors.md. Change entry 0033.
- 2026-11-23T19:14:28-05:00 — Q0010.1: chase into fog → last seen position. Doc: behaviors.md. Change entry 0034.
- 2026-11-23T19:18:25-05:00 — Collision response / un-sticking (velocity reset on block, blocked portion dropped); tracked pathfinding precompute note. Doc: systems/units.md. Change entry 0035 (replaces 0033 turning rule).
- 2026-11-23T19:20:03-05:00 — Q0010.2: patrol origin at execution. Docs: commands.md, behaviors.md. Change entry 0036.
- 2026-11-23T19:20:44-05:00 — Q0010.3: failed patrol leg = arrived, 2 in a row end. Doc: behaviors.md. Change entry 0037.
- 2026-11-23T19:23:06-05:00 — Q0011.1: patches/stations block ground units. Doc: systems/units.md. Change entry 0038. Took in Q0017.
- 2026-11-23T19:26:15-05:00 — Q0011.2: rugged impassable for vehicles except HoverVehicle (double drag). Doc: systems/units.md. Change entry 0039.
- 2026-11-23T19:33:39-05:00 — Q0011.3: path knowledge per player; seen-but-fogged structures = walls; obstacle cache. Doc: behaviors.md. Change entry 0040.
- 2026-11-23T19:34:11-05:00 — Path cache reclassified as implementation suggestion (user). Doc: behaviors.md. Change entry 0041.
- 2026-11-23T19:40:26-05:00 — Map-placed objects known from start for pathing. Doc: behaviors.md. Change entry 0042.
- 2026-11-23T19:41:05-05:00 — Underground units don't collide. Doc: systems/units.md. Change entry 0043.
- 2026-11-23T19:41:33-05:00 — No surfacing onto occupied spots. Doc: systems/units.md. Change entry 0044.
- 2026-11-23T19:42:02-05:00 — Q0011.4 air: ignore terrain, Gliders may overshoot map edge. Doc: systems/units.md. Change entry 0045. Q0011 complete.
- 2026-11-23T19:43:01-05:00 — Surfacing reserves the spot. Doc: systems/units.md. Change entry 0046.
- 2026-11-23T19:44:17-05:00 — Gliders outside map edge visible/attackable at nearest edge tile. Doc: systems/units.md. Change entry 0047.
- 2026-11-23T19:44:49-05:00 — Q0013 points 1/3/6 answered from docs (ActiveGroup = SelectionGroup; SelectionPanel 'objects'). Doc: selection_and_hud.md. Change entry 0048.
- 2026-11-23T19:49:20-05:00 — Q0013.4: Shift+box adds (never removes); empty clicks as built. Doc: selection_and_hud.md. Change entry 0049.
- 2026-11-23T19:57:42-05:00 — Correction: empty-click rules not decided (silence ≠ agreement). Change entry 0050.
- 2026-11-23T19:58:41-05:00 — Empty clicks decided (clear / no-op); Ctrl+Shift pending. Change entry 0051.
- 2026-11-23T20:00:17-05:00 — Ctrl+Shift-click adds on-screen units of type. Doc: selection_and_hud.md. Change entry 0052.
- 2026-11-23T20:01:04-05:00 — Add inputs on enemy selection do nothing. Doc: selection_and_hud.md. Change entry 0053.
- 2026-11-23T20:01:57-05:00 — InfoPanel 2+ placeholder, bookmarked. Change entry 0054.
- 2026-11-23T20:03:22-05:00 — SelectionPanel grouping bookmarked for HUD pass. Change entry 0055.
- 2026-11-23T20:22:55-05:00 — Control groups own objects only. Doc: selection_and_hud.md. Change entry 0056. Took in Q0018, Q0019.
- 2026-11-23T20:23:15-05:00 — Q0019 answered from docs: GDO sees live delivery timer for remembered station. Doc: core/map.md. Change entry 0057.
- 2026-11-23T20:24:52-05:00 — Allies never multi-selected; GDO station intel intended. Change entry 0058. Q0013 + Q0019 done.
- 2026-11-23T20:26:12-05:00 — Q0014.1: force-attack own/neutral allowed. Doc: commands.md. Change entry 0059.
- 2026-11-23T20:28:48-05:00 — Q0014.2: hit at end of Firing; overkill wastes shot+cycle. Doc: combat.md. Change entry 0060.
- 2026-11-23T20:30:33-05:00 — Q0014.3: rugged effects by centre tile. Doc: units.md. Change entry 0061. Q0014 done.
- 2026-11-23T20:31:43-05:00 — Q0016.1: non-capable units move/follow on right-click. Doc: selection_and_hud.md. Change entry 0062.
- 2026-11-23T20:35:28-05:00 — Right-click per-unit resolution; Mixed cursor. Doc: selection_and_hud.md. Change entry 0063.
- 2026-11-23T20:35:43-05:00 — Consistency fix: Attack targeting UI accepts own/neutral destructibles. Doc: selection_and_hud.md. Change entry 0064.
- 2026-11-23T20:37:18-05:00 — Q0016.2: invalid target click ignored + feedback. Doc: selection_and_hud.md. Change entry 0065.
- 2026-11-23T20:44:54-05:00 — Q0016.3a: no command switching while targeting. Doc: selection_and_hud.md. Change entry 0066.
- 2026-11-23T20:45:39-05:00 — Q0016.3b: drag while targeting does nothing. Change entry 0067.
- 2026-11-23T20:46:27-05:00 — Targeting click at press point. Change entry 0068.
- 2026-11-23T20:49:36-05:00 — Q0016.3c: selection changes while targeting. Doc: selection_and_hud.md. Change entry 0069.
- 2026-11-23: Q0016 point 4 answered: Shift keeps targeting active until released (B). Doc: systems/controls/selection_and_hud.md > AwaitingTarget (Shift while targeting). Entry 0070. Q0020, Q0021 arrived; logged in insights.
- 2026-11-24: No smart-casting principle (Groupable fixed per type; workers + casters Ungroupable; player toggle parked); Shift while placing (chain only when builder takes more orders). Docs: selection_and_hud.md > SelectionGroup > No smart-casting, AwaitingTarget > Shift while placing; core/entities.md Groupable. Entry 0071.
- 2026-11-24: Recruits made Groupable (C): Construct/Assist go to all selected Recruits; Agents/Drones stay Ungroupable. Docs: cults/units.md Recruit, selection_and_hud.md No smart-casting + Shift while placing, core/entities.md. Entry 0072.
- 2026-11-24: Q0021 pt 1: targeting click allowance 16 px (B). Doc: selection_and_hud.md > Dragging while targeting. Entry 0073.
- 2026-11-24: Q0020 pt 1: shielded Peacekeepers right-clicked on enemy follow (B, 0062 wins). Doc: gdo/units.md > Riot Shield. Entry 0074.
- 2026-11-24: Q0021 pt 2: shielded Peacekeepers left out of explicit Attack (B), differs from right-click follow. Doc: gdo/units.md > Riot Shield; wording fix in No smart-casting. Entry 0075.
- 2026-11-24: AttackMove issued to units that can't attack; they just move (A; user: a unit without attack never attacks while attack-moving). Docs: commands.md > AttackMove, gdo/units.md > Riot Shield. Entry 0076. Q0022–Q0030 arrived (Q0022 blocking for GDO supplies only); logged in insights.
- 2026-11-24: Q0022 answered A: GDO supply loop deferred to air milestone; Starting Supplies 0 placeholder. Standing permission: placeholder stats when only numbers block the Director. Entry 0077.
- 2026-11-24: Q0021 pt 3: any control group recall resets all menus, even identical selection (B, widened). Doc: selection_and_hud.md > ObjectInterfaceState, Selection changes while targeting. Entry 0078. Q0021 done.
- 2026-11-24: Q0023 pt 1: side lettering clockwise from top in default orientation; Barracks and Cults Armory 2x3 (3 wide). Entry 0079.
- 2026-11-24: Q0023 pts 2-3: Exit Doorway rule (spawn half inside, walk out, stop in doorway if blocked, pause while doorway occupied; buildings may block exits for now). Doc: core/entities.md > Exit Doorway. Entry 0080.
- 2026-11-24: Q0024 pt 1: queue max 20 includes unit in production (all factions). Entry 0081.
- 2026-11-24: Q0024 pt 2: GDO Unit Control reserved when queued (A). Doc: gdo/overview.md > Unit Control. Entry 0082.
- 2026-11-24: Q0024 pt 3: group production with limited resources: as many as possible, least remaining queued production time first. Doc: selection_and_hud.md > No smart-casting. Entry 0083.
- 2026-11-24: Q0024 pt 4: destroyed GDO producers refund as if cancelled. Doc: gdo/structures.md Barracks + DC Cancellation. Entry 0084.
- 2026-11-24: Finished unit fully inside (waiting for doorway) refunded when building dies. Entry 0085.
- 2026-11-24: Q0024 pt 5: rally on the building itself clears it. Entry 0086.
- 2026-11-24: Q0024 pt 6: rally indicators green/red + line. Entry 0087.
- 2026-11-24: Q0024 pt 7: rally on enemy converts to ground rally at last-seen position when it leaves sight (no info leak). Entry 0088. Q0024 done.
- 2026-11-24: Q0025 pts 1,2,4: power producers never slowed (A); ratio gen/cons capped; power counts placement→destruction. Entry 0089.
- 2026-11-24: Q0025 pt 3: Power display used/available (red when over), like other capacity counters. Entry 0090. Q0025 done.
- 2026-11-24: Q0026 pt 1: GDO build area square. Entry 0091.
- 2026-11-24: Q0026 pts 2-3: build area = union of current buildings (shrinks; no DC still OK). Entry 0092.
- 2026-11-24: Q0026 pt 4: overlay = build area only. Entry 0093. Q0026 done.
- 2026-11-24: Q0027 pt 1: ground units of any player block direct placement. Entry 0094.
- 2026-11-24: Q0027 pt 2: plates need Visible tiles; reference base patches within starting DC sight. Entry 0095.
- 2026-11-24: Q0027 pt 3: no buildings on resources except plates on patches. Entry 0096.
- 2026-11-24: Q0027 pt 4: Starting Layout validation = normal rules minus visibility; units not on structures/impassable; no warnings. Entry 0097. Q0027 done.
- 2026-11-24: Q0028 pt 1: plates pay in lumps; power penalty slows cycle. Entry 0098.
- 2026-11-24: Q0028 pts 2-4: last load takes remainder; depleted patch stays under plate; click selects plate. Entry 0099. Q0028 done.
- 2026-11-24: Q0029 pt 1: defeated player's units stay, uncontrollable, idle auto-target. Doc: core/game_rules.md > On Defeat. Entry 0100.
- 2026-11-24: Q0029: correction — defeated units become neutral (replaces 0100); game ends when one side remains (A). Entry 0101.
- 2026-11-24: Defeated players in team games watch with team vision only. Entry 0102.
- 2026-11-24: Q0030 pt 1: enemy structures are auto-targets (non-threatening unless they attack). Entry 0103.
- 2026-11-24: Q0030 pt 2: DC split into Basic/Advanced Construction menus (Factory in advanced). Entry 0104.
- 2026-11-24: Q0030 pt 3: structure InfoPanels (own: activity; others: name/owner/HP). Entry 0105. Q0030 done.
- 2026-11-24: Session end. Answered Q0016 pt 4 through Q0030 (entries 0070–0105). Open: Q0031 (presented, no answer yet), Q0032, Q0020 pts 2–5, Q0017, Q0018.

## 2026-11-24 (cont.) — Q0038 pt 1
Gathering Drones (any player, every step of a Gather job) pass through each other; Drones only; general rule: overlapping units ignore each other until separated. Files: factions/colonists/units.md, systems/units.md. Entry 0106.
Q0038 pt 2: reference check geometry = 2 GU edge to edge, node centred on the row's long side. Files: factions/gdo/economy.md, factions/colonists/units.md. Entry 0107.
Q0038 pt 3: oracle tolerance ±15% + targets 1–4; reference layout is a benchmark only. File: factions/gdo/economy.md. Entry 0108. Q0038 done.
Q0039 pt 1: deploying Drone / warping Beacon is the target structure from the first frame (footprint claimed, same object). File: factions/colonists/economy.md. Entry 0109.
Q0039 pt 2: interpolation sets max; current rises by same amount (damage stays). Entry 0110.
Q0039 pt 3: cancel keeps HP/Shields fraction, >=1 HP. Entry 0111. Q0044 arrived (recorded).
Q0039 pt 4: Drone capacity freed at deploy start, Beacon's 5 at completion; Warp In keeps 5; over capacity allowed (red, blocks production). Entry 0112.
Q0039 pt 5: transforming structures count for defeat; cancelling last = defeated same tick. Entry 0113.
Q0039 pt 6: load lost on deploy completion, returned on cancel. Entry 0114.
Q0039 pt 7: Warp In destroyed = no refund. Entry 0115.
Q0039 pt 8: transforming objects show only Cancel; functions at completion. Entry 0116. Q0039 done.
Q0040 pt 1: Drone deploys at footprint centre (safety net if blocked while overlapping). Entry 0117.
Q0040 pt 2: ground units block Worker-Built and Warp In placement. Entry 0118.
Q0040 pt 3: Warp In requires Visible tiles. Entry 0119.
Q0040 pt 4: worker-built ghost judges by player knowledge; order allowed in fog. Entry 0120.
Q0040 pt 5: odd-sized Warp In ghost default bottom-right; rotate/flip about Beacon centre picks corner. Entry 0121.
Q0040 pt 6: Beacons Ungroupable. Entry 0122.
Q0040 pt 7 / Q0037 pt 12: Gather ends only when mined out (no patch within Drone's search radius), delivers first; Shift-queued orders wait. Search radius around Drone, per-unit stat, 6 GU placeholder. Entry 0123.
Q0040 pt 8: Colonists panels; Beacon has two Warp In menus (Basic/Advanced provisional). Entry 0124. Q0045 arrived.
Q0040 pt 9: Drones Groupable; lingering build ghosts (owner-only, reserve site); group Deploy goes to first Drone in selection order; Shift chains through Drone group (replaces 0071 for Drones). Entry 0125. Q0040 done.
Q0045 pt 1: idle overlapping units push apart. Entry 0126.
Q0045 pt 2 / Q0037 pt 1: Drop Off order = delivery only, then idle/next queued; passes through gathering Drones. Entry 0127. Q0045 done.
Q0037 pt 2: loaded Drone ordered to Gather goes to patch, skips mining, returns. Entry 0128.
Q0037 pt 3: other orders keep the load. Entry 0129.
Q0037 pt 4: interrupted mining collects nothing, patch frees. Entry 0130.
Q0037 pt 5: known, uncovered patches only. Entry 0131.
Q0037 pt 6 (partly): waiting at tried patch, periodic re-check, return to last mined patch. Entry 0132.
Q0037 pt 6: waiter order = implementation's choice (deterministic). Entry 0133.
Q0037 pt 7 / Q0044 pt 5: depleted patch, walking Drones redirect on arrival. Entry 0134.
Q0037 pt 8: missing drop-off -> nearest Egress Node, else idle holding load. Entry 0135.
Q0037 pts 9-10: rally always = unit's default right-click (Drone on patch Gathers; on enemy unit Follows). Entry 0136.
Q0044 pt 2 / Q0037 pt 11: plate can be placed on a patch a Drone mines; plated patch gone for Drones. Entry 0137.
Q0037 pt 13: Drone last load takes remainder. Entry 0138. Q0037 done.
Q0044 pts 1,6 / Q0017 pt 2: remembered objects targetable (incl. rally on remembered enemy structures). Entry 0139.
Q0044 pt 3: empty Drone right-click node = Move. Entry 0140.
Q0044 pt 4: unreachable patch = occupied (redirect/wait/re-check). Entry 0141. Q0044 done.
Q0041 pt 1: Beacon Capacity reserved on queue. Entry 0142.
Q0041 pts 1-2: capacity reserved at production start for GDO and Colonists (Syndicate model); replaces 0082 and 0142. Entry 0143.
Q0041 pts 3-4: Robo Portal exit = closest side to rally, else random unblocked. Entry 0144.
Q0041 pts 5-6: portal energy doorway at centre, open-direction arc, toward rally, go around; replaces 0144. Entry 0145. Q0041 done.
Q0042 pts 1-2: shields regen for neutral and transforming objects. Entry 0146.
Q0042 pt 3: clarified, no modifiers on Shields (docs). Entry 0147.
Q0042 pt 4: shields bar above HP, InfoPanel line. Entry 0148.
Q0042 pt 5: melee reach 8 su placeholder. Entry 0149.
Q0042 pt 6: Convert and Riot Shield keep orders; shielded Attack on hold (follow), resumes on lower. Entry 0150.
Q0042 pt 7: Convert/Riot Shield not queueable. Entry 0151.
Q0042 pt 8: new weapon starts ready after Convert. Entry 0152.
Q0042 pt 9: Convert during Firing/Cooldown deferred. Entry 0153.
Q0042 pt 10: HasTurret renamed CanHaveTurret. Entry 0154. Q0042 done.
Q0034: plate remaining crystals owner only. Entry 0155.
Q0033 pt 1: destination stays as resolved. Entry 0156.
Q0033 pt 2: no re-plan when shortcut opens; recalc only when blocked/stuck. Entry 0157.
Q0033 pt 2 revised: route recalculation timing = Director's discretion (performance, predictability). Entry 0158 replaces 0157.
Q0033 pt 3: all neutral map-placed objects known from start; starting layouts never. Entry 0159. Q0033 done.
Q0035 pt 1: Shift nothing special for rally. Entry 0160.
Q0035 pt 2: per-building rally rules. Entry 0161.
Q0035 pt 3: feedback on unavailable command. Entry 0162. Q0035 done.
Q0036 pt 1: simultaneous last defeats = draw. Entry 0163.
Q0036 pt 2: FFA defeated -> results screen. Entry 0164.
Q0036 pt 3: defeat keeps modes (shield stays). Entry 0165. Q0036 done.
Q0043 pts 1,3,4: M3 scope confirmed; layout reading OK; feedback line OK. Entry 0166.
Q0043 pt 2: Bio Portal 150 kept; Robo Portal 400 -> 250. Entry 0167. Q0043 done.
Q0031: follow itself = Stop. Entry 0168.
Q0032: arrival radius 3 GU (both behaviours). Entry 0169. Q0046 arrived.
Q0046: drop-off destroyed mid-trip -> switch at once (detection Director's call). Entry 0170.
Q0020 pt 2: Riot Shield toggle window. Entry 0171. Q0047 arrived.
Q0020 pt 3: shield raise mid-attack deferred. Entry 0172.
Q0020 pt 4: Peacekeeper / Peacekeeper (Shielded) two unit types. Entry 0173.
Q0020 pt 5: toggle 6 frames, hotkey R placeholders. Entry 0174. Q0020 done.
Q0047 pt 1: waiting buildings in fixed internal order. Entry 0175.
Q0047 pts 2-3: no start without progress; zero-cost always passes. Entry 0176. Q0047 done.
Q0017 pt 1: shots visible over Visible tiles; attacker revealed 1.5 s placeholder, targetable. Entry 0177. Q0017 done.
Q0018 pt 1: fog delay 0.5 s replaces grace period. Entry 0178.
Q0018 pt 2: rotation 45° steps then distance. Entry 0179.
Q0018 pt 3: leash limits chasing only; PathReference rules. Entry 0180. Q0048 arrived.
Q0018 pt 4: unreachable auto-target retry (for now). Entry 0181.
Q0018 pt 5: MinRange: auto skip; explicit back off or fail if immobile. Entry 0182.
Q0018 pt 6: covered by 3 GU radius. Entry 0183. Q0018 done.
Q0048 pt 1: feedback while targeting. Entry 0184.
Q0048 pt 2: unfinished buildings own SelectionGroup. Entry 0185. Q0048 done.

## 2026-11-25 — session checkpoint
All Director questions through Q0048 answered (0106–0185). Next: GDO roster/numbers pass or loose ends.
Syndicate: Guard laser universal req Optics Lab; Agent pistol ground. Entry 0186.
Syndicate: Grenadier added. Entry 0187.
Syndicate: Assault Bike added. Entry 0188.
Syndicate: Shadebringer added (Sludge, Cover). Entry 0189. Director Q0049–Q0051 noted.
Syndicate: Shadebringer Cover duration + charges recharged in tunnels with Vault. Entry 0190.
Syndicate: units inside Cover smoke see nothing. Entry 0191.
Syndicate: Sludge affects all players' units. Entry 0192.
Syndicate: Shadebringer charges are draining tanks. Entry 0193.
Syndicate: Shadebringer not immune to own sludge, laid behind it. Entry 0194.
Syndicate: smoke blocks structures' sight; both toggles at once. Entry 0195.
Syndicate: sludge vs Mechs/Drill Units. Entry 0196. Q0053 noted.
Q0049: second Convert press cancels a deferred conversion (B). Entry 0197.
Q0049 refined + Q0053 pts 2/3: toggles set a target state; ActiveGroup only; type/group switch at press. Entry 0198 replaces 0197.
Q0050: joined group becomes active; on death next group in Tab order. Entry 0199.
Q0051: target lost from vision ends Attack/Follow, unit stops (or next queued). Entry 0200 replaces 0034 fallback.
Q0052: Set Drop-off pointer ReturnResources; load visual public, details owner-only; distinct Gather/DropOff/SetDropOff markers. Entry 0201.
Q0053 pt1: orders accepted per new type at press, executed after toggle (B). Entry 0202.
Rocketman MinRange (TBD). Entry 0203.
Q0054: A for all (dropped target ignored rest of leg; back off mid-band; cornered gives up); locked Rocketman no MinRange. Entry 0204.
Drone acceleration/drag x1.5 (top speed 8); re-measure requested for Q0055. Entry 0205.
Q0056: revealed attacker's shots drawn along whole path (B). Entry 0206.
Syndicate: Laser Battery added; Syndicate lasers red. Entry 0207.
Laser Battery: always charging; empty → same rhythm, ~63% surges. Entry 0208.
Laser Battery: Amplify dial (N^-1/3), narrow arc turret. Entry 0209.
Laser Battery req Optics Lab; LB + Artillery Rig deploy = two unit types under toggle rule. Entry 0210.
Deploy = queueable order, no undeploy until complete, type at completion (replaces 0210 toggle part). Entry 0211.
Stop while deploying winds back. Entry 0212.
Deploy not cancellable (replaces 0212). Entry 0213.
Orders during deploying held, illegal dropped. Entry 0214.
Syndicate: Subterranean Transport added. Entry 0215.
Subterranean Transport hidden underground unless detected. Entry 0216.
Subterranean Transport surfacing: tile passability, not reachability. Entry 0217.
AoE still hits hidden transport. Entry 0218.
All DrillUnits hidden underground unless detected. Entry 0219.
Transport burrow/surface = deploy model; passengers die. Entry 0220.
Transport req Forge; enters tunnels loaded. Entry 0221.
Transport passengers locked in inside network. Entry 0222.
Core principle (predictable not optimal) recorded; Q0057 A/B/A. Entry 0223.
Tunnel sides/hatches/tier look/air doors. Entry 0224.
Tunnel paths one unit at a time. Entry 0225.
Syndicate: Phantom added. Entry 0226.
Phantom turret 60-90, Guard-like role. Entry 0227.
Phantom total cone; stale fog line fixed. Entry 0228. Q0058 noted.
Q0059 placeholders; Q0060 pt1-2. Entry 0229. Q0059-Q0071 noted.
Q0058: AttackMove reach rule (path within leash+sight), no dropped-target memory; replaces 0204 pt1. Entry 0230.
Q0060 pts 3-6 + refund-what-was-paid. Entry 0231.
Correction: Tunnel upgrade refund = current top price (replaces 0231 refund). Entry 0232.
Q0061 pt1-2: Tunnel pays at ghost placement; ghosts count; refunds top price. Entry 0233.
Q0061 rest accepted. Entry 0234.
Q0073: failed chase = can't reach (A). Entry 0235.
Q0055/Q0072: measured Colonists economy = design; table/targets/saturation updated. Entry 0236.
Q0062 pt1: non-overlap across all players (B). Entry 0237.
Q0062 pt2: off-map cells count (B). Entry 0238.
Q0062 pt3a: expansions need Drillable. Entry 0239.
Q0062 pt3c/4: no buildings over expansions and vice versa. Entry 0240.
Q0062 pts 3d/5: no expansions under resources; water/mountain no tunnel space. Entry 0241.
Tunnel area fully drillable/on-map (0242); Q0074 own movement re-check B (0243).
Q0075: only shots at own objects drawn whole. Entry 0244.
Q0063 pt1: flat 2 s transit delay (placeholder). Entry 0245.
Q0063 pt7: walk in at normal speed/spacing; produced units no delay. Entry 0246.
Q0063 pt2: normal Failure rule for entering. Entry 0247.
Tunnel sides corrected (D Supplies, C back); exit nearer crystals. Entry 0248.
Q0063 rest: inside rules, regen 1%/s placeholder, upgrade old tier. Entry 0249.
Q0064: walk-out ejection; air door rate 8 frames placeholder. Entry 0250.
Q0064 pts 3,4,11; Vault/Null Chamber limits noted. Entry 0251.
Q0064 pts 5-7 rallies. Entry 0252.
Q0064 pt8 auto-eject shares queue. Entry 0253.
Q0065 expansions under construction. Entry 0254.
Q0066 Agent gathering. Entry 0255.
Q0067 Supplies gathering. Entry 0256.
Q0068 panel letters. Entry 0257.
Q0069 losing tunnels edges. Entry 0258.
Q0070 Syndicate economy check. Entry 0259.
Q0071 B: placeholders Laser Turret, Grenadier, Chemical Plant. Entry 0260.
Q0071 gap: no AoE underground unless direct target (replaces 0218). Entry 0261.
Q0071 gap reversed: AoE hits underground; expansions high armor placeholder. Entry 0262 replaces 0261.
Q0071 confirmations. Entry 0263.
Building armor rule n / n*size; placeholders corrected. Entry 0264.
Q0071 gap: passengers count against Tunnel Space. Entry 0265.
Q0071 gaps: transport passes underground structures; surfacing rules. Entry 0266.
Q0071 done: turret uses slot; detection after faction units. Entry 0267.
Syndicate: Bomber (name TBD) added. Entry 0268.
Bomber prereqs, ground only, re-arm; Vault network-wide. Entry 0269.
Bomber named Specter. Entry 0270.
Specter default return-to-rearm. Entry 0271.
Specter attack orders / ammo / queue. Entry 0272.
Specter: no T3 -> ignores attack. Entry 0273.
Satellites require Orbital Command. Entry 0274.
Satellite concept. Entry 0275.
Refract via satellites, chains. Entry 0276.
Relay (renamed from Refract) specified. Entry 0277.
Relay additive; chain break ends relay. Entry 0278.
Relay: all selected must reach; armed satellite takes Attack. Entry 0279.
Satellite thrust no recharge; armed satellite uses player vision. Entry 0280.
Satellite named Prism; shootable during launch. Entry 0281.

## 2026-11-25 session end (part)
Answered Q0069–Q0071 (0258–0267): losing-Tunnel edges, Syndicate economy check, M4 scope = placeholders (Laser Turret, Grenadier, Chemical Plant), AoE hits underground (0262; 0261 void), building armor rule n / n×size (0264), transport gaps, turret uses Tunnel slot, detection after all faction units. Syndicate roster: Specter bomber (0268–0273), Vault network-wide, Prism satellite + Laser Battery Relay (0274–0281). User's order of work: all faction units → all techs → detection. Identity check raised, awaiting answers.
2026-11-25 18:00 — Split all faction units.md/structures.md into one file per object (units/, structures/ folders with README for shared rules + index); refs rewritten; content verified line-for-line. Relayed user's request for modular code. Entry 0282.
2026-11-26 — Q0076–Q0087 arrived; answered from docs Q0078 (0283), Q0085 (0284), Q0086.1 (0285).
2026-11-26 — Q0076 discussion; asked Director for cost estimates (0286), no design change.
2026-11-26 — Q0077: eject refused during transit; no-rally 1 GU out. Entry 0288.
2026-11-26 — Q0079 as proposed. Entry 0289.
2026-11-26 — Q0080: units don't block expansions; ghosts moot; Warp In red at once. Entry 0290.
2026-11-26 — Q0081 A. Entry 0291.
2026-11-26 — Q0082 A/B/A. Entry 0292.
2026-11-26 — Q0083: friendly fire, reveal on damage, Attack Ground continuous, all shots clipped to vision (replaces 0244). Entry 0293.
2026-11-26 — Q0076/Q0088: Agent mining 4 s, keep collision, ref 3 GU, sidestep documented. Entry 0294.
2026-11-26 — Q0076 sub-points confirmed. Entry 0295.
2026-11-26 — Q0084 pts 1–3. Entry 0296.
2026-11-26 — Q0084 pts 4–6. Entry 0297.
2026-11-26 — Q0084 pts 7–9. Entry 0298.
2026-11-26 — Q0086 pts 2,4,5. Entry 0299.
2026-11-26 — Q0086 complete. Entry 0300.
2026-11-26 — Q0087 (a). Entry 0301. All open Director questions answered.
2026-11-26 — Cults Soldier → Militant. Entry 0302.
2026-11-26 — Q0089 (a): third Agent at work by 0:40; table an ideal. Entry 0303.
2026-11-26 — Gunner TailDisjointed. Entry 0304.
2026-11-26 — Acolyte → Disciple. Entry 0305. Q0092–Q0108 arrived.
2026-11-26 — Disciple cloud spell concept. Entry 0306.
2026-11-26 — Disciple cloud details. Entry 0307.
2026-11-26 — Cloud: attackers under it do full damage. Entry 0308.
2026-11-26 — Cloud modifiers/release/channel end. Entry 0309.
2026-11-26 — Cloud: damage doesn't end channel. Entry 0310.
2026-11-26 — Shared Burden named. Entry 0311.
2026-11-26 — Spell renamed Shroud. Entry 0312.
2026-11-26 — Cults buggy concept. Entry 0313.
2026-11-26 — Jackal; Chop Shop like Armory. Entry 0314. Q0109 (blocking) arrived.
2026-11-26 — Q0109 step 1: Agents wait only within reach. Entry 0315.
2026-11-26 — Cults light glider concept. Entry 0316.
2026-11-26 — Light glider details. Entry 0317.
2026-11-26 — Sparrow. Entry 0318.
2026-11-26 — Cults placeholders for all designed content. Entry 0319 (Q0092–Q0094, Q0108.1).
2026-11-26 — Q0110 (d); Recruit repair noted. Entry 0320.
2026-11-26 — Repair rules. Entry 0321.
2026-11-26 — Recruit panel letters, Assist/Repair pointers. Entry 0322 (Q0095, Q0108.2).
2026-11-26 — Sidestep 90°/body width (infantry, hover, tracked); wheeled graze. Entry 0323.
2026-11-26 — Q0111: try 0323 sidestep with 1-sq hatch first. Entry 0324.
2026-11-26 — Enter beats Repair; Agent F Repair; Mech sidestep. Entry 0325.
2026-11-26 — Agent Construct menu. Entry 0326.
2026-11-26 — Q0096 Basic/Advanced. Entry 0327.
2026-11-26 — Q0096 Basic/Advanced (0327); placeholder+cleanup batch (0328: Q0123, Q0125, Q0112.1, Q0120.1, Q0124.5-6).
2026-11-26 — Q0124.1-4: repair ends; gliders no; expansions regen. Entry 0329.
2026-11-26 — Q0114.1/.3, Q0121.2, regen in combat. Entry 0330.
2026-11-26 — MaxHP up keeps damage. Entry 0331.
2026-11-26 — First in first trained; keep damage. Entry 0332.
2026-11-26 — Gunner firing 4 frames. Entry 0333.
2026-11-26 — Shroud rules, Disciple ungroupable. Entry 0334.
2026-11-26 — Garrison, entrances, Temple, prereqs, Shrine. Entry 0335.
2026-11-26 — Retroactive augment research; Shrine sides; Q0127 accept. Entry 0336.
2026-11-26 — Kinds, Release, Eject All; Q0128.1. Entry 0337.
2026-11-26 — All Cults upgrades retroactive; DrillUnit sidestep. Entry 0338.
2026-11-26 — Q0097 group construction. Entry 0339.
2026-11-26 — Q0098 refunds. Entry 0340.
2026-11-26 — Q0099 Recruitment Area. Entry 0341.
2026-11-26 — Q0100 Hall production. Entry 0342.
2026-11-26 — Q0129 sidestep side. Entry 0343.
2026-11-26 — Q0101.1. Entry 0344.
2026-11-26 — Q0102. Entry 0345.
2026-11-26 — Q0103 economy check. Entry 0346.
2026-11-26 — Q0104 entering pool buildings. Entry 0347.
2026-11-26 — Q0105.3-5. Entry 0348.
2026-11-26 — Q0106 evacuation. Entry 0349.
2026-11-26 — Q0107 (b). Entry 0350.
2026-11-26 — Preview shows own pending Halls. Entry 0351.
2026-11-26 — Preview counts remembered Halls. Entry 0352.
2026-11-26 — Q0119. Entry 0353.
2026-11-26 — Q0120 Sparrow flight. Entry 0354.
2026-11-26 — Sparrow grip 0.5. Entry 0355.
2026-11-26 — Q0122: Rocketman/Arsenal placeholders. Entry 0356.
2026-11-26 — Q0090. Entry 0357.
2026-11-27 — Q0091 pop-up turret details. Entry 0358.
2026-11-27 — Turrets Tunnel-only; Agent panel reverted. Entry 0359.
2026-11-27 — Turrets neutral with Tunnel. Entry 0360.
2026-11-27 — Missile platform concept. Entry 0361.
2026-11-27 — Q0130 (d). Entry 0362.
2026-11-27 — Hand of Heaven. Entry 0363.
2026-11-27 — Prayer-mobile; Jackal→Seeker. Entry 0364.
2026-11-27 — Prayer-mobile placeholders. Entry 0365.
2026-11-27 — GDO placeholder pass. Entry 0366.
2026-11-27 — Syndicate placeholder pass. Entry 0367.
2026-11-27 — GDO rule confirmations. Entry 0368.
2026-11-27 — Syndicate rule confirmations. Entry 0369.
2026-11-27 — Session end: all Director questions answered through Q0130; GDO/Syndicate placeholder passes; Cults roster: Hand of Heaven, Prayer-mobile, Seeker rename; turrets Tunnel-only and neutral with Tunnel; Cults identity check given (tensions in insights).
2026-11-27 — Doc reorg (user request): split selection_and_hud, behaviors, syndicate/tunnels; moved Cults Recruitment Hall and Storage to structures/ (restoring the lost RecruitmentHall heading). Content verified line-for-line; refs and index updated. Entry 0370.
2026-11-27 — Q0131–Q0142 arrived; Q0132/Q0138/Q0141 answered from docs (0371).
2026-11-27 — Q0131 (a). Entry 0372.
2026-11-27 — Q0133 (a). Entry 0373.
2026-11-27 — Q0134 (a) + guideline. Entry 0374.
2026-11-27 — Q0136: drop-off nearest the worker at end of mining, tie earlier id. Entry 0375.
2026-11-27 — Q0137 (a), Stop clears the lock. Entry 0376.
2026-11-27 — Q0139 (b). Entry 0377.
2026-11-27 — Q0140 (b). Entry 0378.
2026-11-27 — Q0142 (b). Entry 0379.
2026-11-27 — Q0135 partial: Storage back to 3x2. Entry 0380.
2026-11-27 — Q0135 answered: realistic opening should overtake by ~4:00; longer timeline to 6:00. Entry 0381.
2026-11-27 — Economy comparison metrics (income/s, mined, bank, spent on economy). Entry 0382.
2026-11-27 — Economy runs: all spending is economy, Generators included. Entry 0383.
2026-11-27 — Agents re-pick Tunnel every return. Entry 0384.
2026-11-27 — Q0144 (part): rerun with two-sided 8-patch line (~32 seats), One vs Two Halls, start with 1 Storage, Storages as needed. Entry 0385.
2026-11-27 — Q0143 (a). Entry 0386.
2026-11-27 — Hall placeable at map edge, off-map tiles non-Recruitable (replaces Q0099). Entry 0387. Q0145 arrived.
2026-11-27 — DC Place button with progress bar. Entry 0388.
2026-11-27 — Recruit E: Attack. Entry 0389.
2026-11-27 — Ghost entrance/exit arrows. Entry 0390.
