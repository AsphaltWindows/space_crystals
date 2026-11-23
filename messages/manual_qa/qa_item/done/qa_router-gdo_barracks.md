# gdo_barracks

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# gdo-barracks

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Barracks structure and its interface as defined in `artifacts/designer/design/gdo_objects.md`.

GDO infantry production structure. Produces infantry units from a build queue. Units exit from one of the short sides (B side).

**Entity Type:** Structure Type
**Faction:** GlobalDefenseOrdinance

**Stats:**
- Size: 3x2 grid units
- SymmetryType: ABAC (two distinct short sides, two identical long sides)
- MaxHP: 300
- PointArmor: 1
- FullArmor: 6
- SightRange: 4 grid units
- Destructible: true
- Groupable: true
- BuildRadiusExtension: 2 grid units
- Power: -30 (consumes power)

**BarracksInstanceState:**
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 5)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

**RallyPoint behavior:**
When a unit finishes production and spawns: if RallyPoint is set, issue default right-click command resolved against RallyPoint target (ground=Move, enemy=Attack, friendly/neutral=Move to object). If None, unit spawns with no command. If RallyPoint references destroyed object, reset to None.

**Produces:**
- Peacekeeper: 50 Space Crystals, 80 frames (5 seconds)

**ObjectInterfaceState[Barracks]:**

DefaultState right-click:
- Ground: SetRallyPoint to location
- Object: SetRallyPoint to object

Immediate commands:
- **Q: Build Peacekeeper** — deducts 50 crystals, adds to queue. Requires queue < 5 and sufficient crystals and Unit Control.
- **X: Cancel Production** — removes last queue entry, full refund. Requires non-empty queue.

Target commands:
- **C: Set Rally Point** — enters AwaitingTarget[SetRallyPoint]. Left-click sets rally point, returns to DefaultState.

**Production:** Constructed by DeploymentCenter for 200 Space Crystals, 160 frames (10 seconds). Follows ConstructionHP Rule.

## QA Instructions

1. Build a Barracks from DC — verify 200 crystals deducted, 10-second construction.
2. Verify it occupies 3x2 grid units with ABAC symmetry (short sides differ).
3. Press Q to queue a Peacekeeper — verify 50 crystals deducted and unit appears in queue.
4. Queue 5 Peacekeepers — verify Q becomes unavailable (max queue reached).
5. Press X to cancel — verify last queued unit removed and 50 crystals refunded.
6. Wait for production to complete — verify Peacekeeper spawns at Side B.
7. Set rally point (C, then click ground) — verify next produced unit auto-moves to rally point.
8. Set rally point on enemy — verify produced unit auto-attacks the enemy.
9. Right-click ground from Barracks DefaultState — verify rally point is set.
10. Destroy the rally point target — verify rally point resets to None.
11. Verify Barracks consumes 30 power (Power: -30).
12. Verify BuildRadiusExtension of 2 grid units expands GDO build area.
