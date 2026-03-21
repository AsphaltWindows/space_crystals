# hq-production-interface

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# hq-production-interface

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Fix: Headquarters should show production commands, not unit commands. As defined in `artifacts/designer/design/syndicate_objects.md` under 'ObjectInterfaceState[Headquarters]'.

The Headquarters is a unit-producing structure. Its ObjectInterfaceState should display production commands in DefaultState:

**DefaultState commands:**

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Agent** — deducts 100 Space Crystals, adds Agent to BuildQueue. Available if queue < 5 and sufficient crystals.
- **W: Build Guard** — deducts 125 Space Crystals, adds Guard to BuildQueue. Available if queue < 5 and sufficient crystals.
- **X: Cancel Production** — removes last BuildQueue entry, refunds full cost. Available if queue not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point** — enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets rally point (CommandIssuingTransition, returns to DefaultState).

The HQ must NOT display unit movement/combat commands (Move, Attack, Stop, etc.) — it is a structure, not a unit.

**Note:** Some of this functionality may already be partially implemented. Downstream agents should check the current codebase state.

## QA Instructions

1. Select a Headquarters (underground expansion).
2. Verify the command panel shows Q (Build Agent), W (Build Guard), X (Cancel Production), C (Set Rally Point).
3. Verify NO unit commands (Move, Attack, Stop, Patrol, etc.) are displayed.
4. Press Q — verify Agent is added to build queue and 100 Space Crystals are deducted.
5. Press W — verify Guard is added to build queue and 125 Space Crystals are deducted.
6. Press X — verify last queue entry is removed and cost is refunded.
7. Queue 5 units — verify Q and W become unavailable (queue full).
8. Right-click ground while HQ is selected — verify a rally point is set.
9. Press C, then left-click ground — verify rally point is set via AwaitingTarget flow.
