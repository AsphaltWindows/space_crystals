# entity-registry-costs

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Replace hardcoded production/construction cost lookups with EntityRegistry-based cost lookups.

**Current state:** Costs are hardcoded in match arms:
- `DeploymentCenterState::construction_cost(object)` matches ObjectEnum variants to StructureCost values
- `BarracksState::production_cost(object)` matches ObjectEnum variants to StructureCost values
- `HeadquartersState::production_cost(object)` matches ObjectEnum variants to StructureCost values
- Similar patterns in other production building states (CultsArmory, RecruitmentCenter, SupplyTower)

**After this task:** Cost lookups go through the registry:
1. Add a `cost(&self, object: &ObjectEnum) -> Option<StructureCost>` convenience method to EntityRegistry
2. Replace `construction_cost()` and `production_cost()` match arms with registry.cost() calls
3. Update the production queue systems (`try_queue`, construction start) to use registry cost lookups
4. Update cancellation refund logic to use registry costs

**Callers to update:**
- DeploymentCenterState construction_cost() and cancellation_refund()
- BarracksState production_cost()
- HeadquartersState production_cost()
- Any other production building cost lookups
- UI display of costs (if cost is shown in command panel tooltips or HUD)

**Tests:**
- Registry cost lookup returns identical values to current match-based lookups for all producible entities
- Registry returns None for non-producible entities (resources, starting structures)
- Production queue try_queue still correctly deducts costs
- Cancellation refund returns correct values
- All existing production/construction tests pass

**Reference:** Architecture doc Phase 6 (Wire Registry Consumers)
