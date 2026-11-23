# Discussion: Data-Driven Entity Definitions via YAML

## Metadata
- **Created by**: operator
- **Created**: 2026-03-25T12:00:00Z
- **Status**: open

## Close Votes
VOTE:task_splitter
VOTE:designer
VOTE:developer
VOTE:automatic_qa
VOTE:task_planner
VOTE:software_architect

## Discussion

### [operator] 2026-03-25T12:00:00Z

The user wants to open a team discussion about a potential philosophical shift in how entities (units, buildings, and other game objects) are defined in the codebase.

**The idea:** Instead of hardcoding entity definitions in Rust, move them to external YAML data files. This would cover:

- **Object stats** (health, damage, speed, costs, etc.)
- **Type definitions** (unit types, building types)
- **Behaviors** (what actions/behaviors an entity can perform)
- **Command panel options** (what buttons/commands appear for each entity type)
- **State machines** (state definitions and transitions)

**Why this is being raised:** The user is not committed to this direction — they want to hear what the team thinks before deciding. This is an open discussion, not a directive.

**Questions for the team to consider:**

1. **Designer**: How would data-driven definitions affect the design workflow? Would YAML files make it easier to iterate on unit/building designs without touching code?
2. **Software Architect**: What are the architectural tradeoffs? Consider: hot-reloading during development, mod support potential, compile-time safety vs runtime flexibility, Bevy's ECS patterns and how data-driven definitions interact with them, serialization/deserialization overhead, and whether this is the right abstraction level for our current stage of development.
3. **Developer**: What's the practical impact on the current codebase? Given the already-large files (command_panel.rs, core.rs, behaviors.rs, faction.rs), would externalizing definitions help with code size and maintainability, or add complexity?

**Key tensions to discuss:**
- Rust's compile-time guarantees vs. runtime data loading
- Development velocity now vs. flexibility later
- Scope of what goes into data files — everything listed above, or a subset?
- Whether Bevy has existing patterns or plugins for this (e.g., `bevy_common_assets`, RON files, etc.)

This is a discussion topic. All agents with relevant perspectives should weigh in before any decisions are made.

### [automatic_qa] 2026-03-25T04:59:20Z

From an automated QA perspective, a few testing considerations:

1. **Runtime validation becomes critical**: Moving definitions from Rust to YAML trades compile-time safety for runtime flexibility. This means we'd need comprehensive validation tests that load and verify all YAML definitions on every test run — catching typos, missing fields, invalid enum variants, etc. that the compiler currently catches for free.

2. **Test fixture management**: Currently tests reference Rust types directly. With YAML-driven definitions, tests would either need to load the YAML files (coupling tests to data files) or maintain separate test fixtures (risk of divergence). Either way, test infrastructure needs updating.

3. **Regression detection**: The current 1800+ unit tests and 300+ integration tests rely on hardcoded types. A migration would require updating many tests. I'd recommend a phased approach where both paths coexist during transition so the test suite stays green throughout.

4. **Positive for QA**: If entity definitions are data-driven, we could potentially generate test cases from the YAML definitions themselves — e.g., automatically testing that every defined unit type can be spawned, every command panel entry resolves to a valid action, etc. This could significantly improve automated test coverage.

Overall, I support this direction if accompanied by a strong validation layer that runs as part of `cargo test`. The key risk is silent runtime failures replacing compile-time errors.

### [developer] 2026-03-25T04:59:29Z

From the implementation side, here's my assessment:

**Current pain points that data-driven definitions could help with:**

1. **objects.rs (1312 lines) and utils.rs (1397 lines)** are essentially big lookup tables — ObjectType definitions, spawn functions, stat constants. These are the most natural candidates for externalization since they're already declarative in nature (mapping ObjectEnum → stats/components).

2. **Command panel grid layouts** (grid_layout.rs) are another declarative mapping — which buttons appear for which entity in which state. This is currently a massive match statement that grows with every new entity type.

3. **Adding a new unit type currently touches 5-10 files**: ObjectEnum variant, ObjectType entry, spawn function, command panel grid, behavior registration, possibly faction setup. A data file could consolidate the declarative parts.

**Practical concerns:**

1. **Compile-time safety is genuinely valuable here.** Typos in YAML stat names become runtime errors. Rust enums currently guarantee exhaustive matching — if you add a new ObjectEnum variant, the compiler tells you every match that needs updating. This has caught real bugs during development. We'd need a build-time validation step or proc macros to preserve this.

2. **Not everything is declarative.** Behaviors, state machines, and system logic are procedural Rust code. The spawn functions don't just set stats — they compose ECS components with specific Bevy types (Transform, Mesh, Material). Drawing the line between 'data' and 'logic' would be the hardest design decision.

3. **Bevy ecosystem:** RON is more idiomatic for Bevy than YAML (bevy_common_assets supports both, but RON has first-class serde support and is used by Bevy's scene format). If we go this route, RON > YAML for Bevy projects.

4. **Codebase scale (49K lines):** We're large enough that the boilerplate of adding entities is noticeable but not yet unmanageable. The recent file splits (command_panel/, behaviors/, faction/) helped more with maintainability than data-driven would, because the complexity is in the *logic*, not the *data*.

**My recommendation — phased approach:**

- **Phase 1 (high value, low risk):** Extract numeric stats (HP, damage, speed, costs, build times) into data files. Keep spawn functions and component composition in Rust. This is ~70% of the boilerplate win with ~20% of the risk.
- **Phase 2 (if Phase 1 works well):** Command panel grid layouts — these are pure data mappings.
- **Phase 3 (evaluate later):** Behavior/state machine definitions — only if we find a clean abstraction boundary.

I would NOT recommend trying to externalize everything at once. The current 1871+ test suite depends heavily on compile-time guarantees, and a big-bang migration would be risky. Start with stats, prove the pattern, then expand.

### [task_splitter] 2026-03-25T04:59:33Z

From a task decomposition and work-scoping perspective, here are my observations:

**Impact on task splitting:** A data-driven approach would significantly change how I decompose feature requests. Currently, adding a new unit/structure is a single self-contained task (define ObjectEnum variant, type data, spawn function, interface state, tests — all in Rust). With YAML definitions, new-entity tasks would split into: (1) YAML definition authoring, and (2) any custom behavior/system code that can't be data-driven. The first category becomes trivial; the second remains the real work.

**What I've seen in the codebase so far:** After processing ~20+ feature requests, clear patterns emerge:
- **Stat definitions** (HP, armor, costs, damage, ranges, timing) — highly repetitive across units/structures. These are prime candidates for externalization. Every unit task I split involves the same boilerplate: `type_data`, `attack_data`, spawn function assembling identical component bundles.
- **Command panel layouts** — each ObjectInterfaceState match arm is a grid of slots with hotkeys. Very data-driven already in spirit, just expressed in Rust match arms. Good candidate.
- **Behaviors and systems** — these are logic-heavy (pathfinding, phase state machines, combat cycles). NOT good candidates. These need Rust's type system and compile-time checks.
- **State machines** — mixed. State *definitions* could be data-driven, but transition *logic* is inherently code.

**Timing concern:** We have a large backlog of foundational features already processed and implemented. Introducing this migration now would create a massive refactoring wave — every existing unit, structure, and interface would need migration tasks. I'd estimate 15-25 migration tasks on top of whatever new features are incoming. This is the kind of change that's cheaper to do either very early (before much exists) or as a deliberate milestone (freeze features, migrate, then resume).

**My recommendation for scoping:** If the team decides to proceed, I'd suggest a phased approach that I could split cleanly:
1. **Phase 1**: Stats/costs only (ObjectType data, AttackData, construction costs) — lowest risk, highest boilerplate reduction
2. **Phase 2**: Command panel layouts — moderate complexity, big maintainability win
3. **Phase 3**: Spawn function component assembly — requires a registry/factory pattern
4. Skip behaviors and state machine logic — keep in Rust

Each phase would be a separate architecture_request I could decompose into concrete tasks. Trying to do it all at once would be extremely difficult to split into safe, independently-implementable tasks.

### [designer] 2026-03-25T04:59:34Z

From the design workflow perspective, here's how I see the tradeoffs:

**What would map well to data files:**

The design documents already define entities in a highly structured, attribute-driven format. Look at how a Peacekeeper is specified in `gdo_objects.md` — it's essentially a flat list of typed fields: MaxHP, PointArmor, UnitBase, AttackAttributes with sub-fields, etc. The same pattern holds for structures (PowerPlant, Barracks) and across all factions. This format is already very close to what a YAML definition would look like. Stats, costs, timing values, sight ranges, attack attributes, unit base attributes — all of these are pure data with no logic.

**What concerns me:**

1. **Behaviors and state machines are where the complexity lives.** The design docs describe behaviors like SupplyChopper's automated pickup/dropoff cycle, the Armory's training queue consumption mechanic, or Tunnel Network pathfinding — these involve conditional logic, multi-step sequences, and interactions between multiple systems. Pushing these into YAML risks either (a) creating a mini scripting language in YAML that's harder to reason about than Rust, or (b) forcing behaviors into a rigid template that can't express the variety we need as factions diverge.

2. **Command panel layouts** are semi-structured but tightly coupled to ObjectInterfaceState transitions (see `control_system.md`). Each button maps to state changes, commands, or mode switches. This might be data-driveable but the state machine driving it is non-trivial.

3. **Design iteration speed today vs. tomorrow.** Right now the bottleneck isn't 'changing a number requires recompilation' — it's that we're still defining what the numbers should be (see the many TBD values across Cults and Syndicate objects). Data files shine when you're tuning known parameters. We're still in the phase of discovering what parameters exist.

**My recommendation — subset approach:**

If this moves forward, I'd suggest a phased scope:
- **Phase 1 (high value, low risk):** Object stats — HP, armor, costs, speeds, timing values, sight ranges, attack attributes. These are pure data, already structured identically across all objects in the design docs, and would benefit most from hot-reload tuning.
- **Phase 2 (moderate value, moderate risk):** Type definitions and command panel button layouts — what commands appear, slot assignments, keybindings per object type.
- **Defer:** Behaviors and state machines. Keep these in Rust where the type system can catch errors and where the logic can be as expressive as needed.

This maps cleanly to how the design docs are organized: the attribute blocks are data, the behavioral descriptions are logic.

### [task_planner] 2026-03-25T04:59:52Z

From a task planning perspective, I can offer concrete observations about how entity data is currently distributed across the codebase and what externalization would mean practically.

**Current state of entity definitions (evidence from codebase investigation):**

1. **Unit stats** are already well-isolated in `artifacts/developer/src/game/units/types/unit_data.rs`. Each unit type has a dedicated `*_type_data()` function returning `UnitTypeData` (HP, armor, faction, unit base) and a `*_attack_data()` function returning `AttackAttributesData` (damage, range, cooldowns). Plus associated constants (`AGENT_MINING_DURATION`, `AGENT_CRYSTAL_CARRY`, etc). This is the easiest win for externalization — these are pure data, no logic.

2. **Command panel layouts** are hardcoded in `grid_layout.rs` via `get_grid_slot_action()` — a giant match on `ObjectInterfaceState` mapping (row, col) pairs to `CommandButtonAction`s. This is ~140 lines of pure data-as-code. Externalizing this would meaningfully shrink the command panel module.

3. **Command availability** is logic-driven in `UnitCommand::is_available()` (commands.rs:48) — it takes capability flags (`has_attack`, `can_target_ground`, `can_reverse`, `is_syndicate`) and returns bool. This is harder to externalize because it encodes conditional logic, not just data.

4. **Behavior/state machine definitions** are deeply embedded in system implementations (behaviors.rs, faction.rs). These are the hardest to externalize — they're Bevy ECS systems with queries, not declarative data.

**My recommendation — phased approach:**

- **Phase 1 (high value, low risk):** Externalize unit stats (UnitTypeData, AttackAttributesData, cost constants). These are already structured as pure data. RON or YAML files would work. The `*_type_data()` functions become thin wrappers loading from a registry.

- **Phase 2 (medium value, medium risk):** Externalize command panel grid layouts. The `get_grid_slot_action()` match arms could become a data table mapping (state, row, col) -> action.

- **Phase 3 (evaluate later):** Behavior definitions and state machines. These require a DSL or scripting layer — much more complex. I'd defer this until we have more unit types and the current approach is clearly limiting.

**Practical concern from my role:** I plan tasks by mapping developer_tasks to specific files, patterns, and integration points. Today, adding a new unit type means touching unit_data.rs, commands.rs, grid_layout.rs, and faction.rs in predictable ways. Data-driven definitions would change this pattern significantly — I'd need to understand the registry/loading system instead. This is fine if the new pattern is well-documented, but the transition period will temporarily make task planning harder.

**On timing:** With only 3 unit types defined so far, we're not yet at the pain point where hardcoded definitions are a bottleneck. Phase 1 is worth doing soon as a foundation, but I'd caution against a full data-driven overhaul until we're adding units frequently enough to feel the friction.

### [software_architect] 2026-03-25T05:00:18Z

## Architectural Analysis: Data-Driven Entity Definitions

I've investigated the current codebase thoroughly. Here's my assessment covering tradeoffs, feasibility, and a recommended phased approach.

### Current State of Entity Definitions

Entity data is currently scattered across several locations, all hardcoded in Rust:

1. **Unit stats** (`unit_data.rs`): `peacekeeper_type_data()`, `agent_type_data()`, `guard_type_data()` — each returns a `UnitTypeData` struct with HP, armor, silhouette, faction, unit_base. Attack stats via `*_attack_data()` returning `AttackAttributesData`. Constants for costs, carry amounts, durations.

2. **Structure stats** (`structures.rs`): Costs and build durations embedded in match arms inside `DeploymentCenterState::construction_cost()`, `BarracksState::production_cost()`, `HeadquartersState::production_cost()`. Each production building has its own `*State` component with duplicated queue/cancel logic.

3. **Command panel layout** (`grid_layout.rs`): Hardcoded `(row, col) => CommandButtonAction` match arms per `ObjectEnum` variant. Button enablement rules in `helpers.rs` with per-type match logic.

4. **Spawning** (`utils.rs`, faction files): Entity assembly via manual component insertion, each unit type has its own spawn function.

5. **Behavior availability**: Implicitly determined by which components are attached at spawn time and which systems match those components.

There are currently ~20 `ObjectEnum` variants. The pattern is: every new unit or structure requires touching 4-6 files, adding enum variants, match arms, spawn functions, command panel entries, and stat functions.

### Tradeoff Analysis

**Arguments FOR data-driven (YAML/RON):**
- **Designer velocity**: Stats and costs become editable without recompilation. At ~50K LOC, compile times are already non-trivial.
- **Reduced code duplication**: The production building pattern (Barracks, Headquarters, Armory all have nearly identical queue/cancel/cost logic with different values) is a prime candidate. Data files would collapse these into one generic system parameterized by data.
- **File size reduction**: The oversized files we've been splitting (faction.rs, command_panel helpers) are large precisely *because* they enumerate every entity type. Data-driven definitions would eliminate most match arms.
- **Mod support potential**: A future consideration but architecturally significant.
- **Hot-reload during development**: Bevy's asset system supports this. Change a stat, see it in-game without restart.

**Arguments AGAINST (or for caution):**
- **Loss of compile-time safety**: Rust's exhaustive match on `ObjectEnum` currently guarantees that every entity type is handled everywhere it needs to be. Moving to string keys or data IDs means runtime errors instead of compile errors when a definition is incomplete.
- **Complexity at our stage**: We have ~20 entity types. The pain threshold for hardcoding is real but manageable. A full data-driven system is a significant architectural investment.
- **Bevy ECS tension**: Components are Rust types. You can't define new component *types* in YAML — only parameterize existing ones. The data layer would need to be "which components to attach and with what values," not "define arbitrary new components."
- **State machine complexity**: Behaviors involve system logic (pathfinding, combat phases, construction sequences). These can't be meaningfully externalized to data. The *availability* of behaviors could be data-driven, but the *implementation* stays in Rust.
- **Testing disruption**: Our 1,920+ unit tests and 318 integration tests reference hardcoded functions like `peacekeeper_type_data()`. A full migration would require updating all of them.

### My Recommendation: Phased, Scoped Approach

I recommend **not** doing a wholesale migration to YAML. Instead, a targeted two-phase approach:

**Phase 1: Data Registry Pattern (Rust-side, no external files)**

Create a centralized `EntityRegistry` resource that maps `ObjectEnum` → stat bundles. This is a *precondition* for Phase 2 and delivers immediate value:

```
// Conceptual — not actual implementation
pub struct EntityDefinition {
    pub object_type: ObjectEnum,
    pub faction: FactionEnum,
    pub stats: UnitTypeData,           // or StructureTypeData
    pub attack: Option<AttackAttributesData>,
    pub cost: Option<StructureCost>,
    pub command_slots: Vec<CommandSlotDef>,
    pub producible_by: Vec<ObjectEnum>,
}
```

This eliminates the scattered match arms and the production state duplication (Barracks/HQ/Armory all become "production building with config X"). Compile-time safety is preserved because the registry is populated in Rust. This also directly addresses our file-size concerns better than the ongoing file-split work.

**Phase 2: External data files (optional, later)**

Once the registry pattern is stable, the *population* of the registry can optionally move to RON or YAML files loaded via Bevy's asset system. This becomes a much smaller change because all systems already read from the registry, not from scattered hardcoded functions.

I'd recommend **RON over YAML** for Bevy projects — RON is Rust-native, has first-class serde support, Bevy uses it internally, and `bevy_common_assets` exists for custom asset types. No extra dependency needed.

### Scope Assessment

- **What SHOULD be data-driven**: Stats (HP, damage, speed, costs, durations), command panel layout, production recipes, unit control costs, build radii, power values
- **What should STAY in Rust**: System logic (combat phases, pathfinding, construction sequences), component type definitions, state machine transitions, ECS system implementations
- **Gray area**: Behavior *availability* per unit type (e.g., "can gather resources", "can build tunnels") — could be data-driven as capability tags, but the systems themselves stay in code

### Timing

Phase 1 (registry pattern) aligns well with our existing refactoring work. The production building deduplication I've already tracked as a deferred priority is essentially a subset of this. I'd suggest completing the current file-split work first, then pursuing the registry as the next major architectural initiative.

Phase 2 should wait until we have significantly more entity types (30+) or an explicit modding/hot-reload requirement from design.

### [operator] 2026-03-25T12:30:00Z

**Direction update from the user** — after reading the team's feedback, the user agrees that abandoning Rust's type safety for external YAML files is probably the wrong move. The compiler catching exhaustive matches, typos, and missing fields is too valuable to give up.

However, the user still sees value in making entity definitions **more declarative** — just within Rust itself. The new question for the team:

**Can we design a system of declarative Rust struct types (or builder patterns, or macro-driven registrations) that give us the organizational benefits of data-driven design while keeping compile-time safety?**

Think along the lines of:
- A single struct type (or small family of structs) that fully describes a unit or building — stats, costs, command panel layout, available commands, etc.
- Entity definitions that read as declarations rather than scattered logic across 5-10 files
- Adding a new unit type ideally means filling out one struct/definition in one place, not touching many files

This is still an open discussion. The team should consider:
1. What Rust patterns best support this? (const structs, builder pattern, declarative macros, proc macros, trait-based registration?)
2. How does this interact with Bevy's ECS — can a declarative definition drive component composition at spawn time?
3. What's the right scope? Stats + costs + command panel layout seem like the consensus "easy wins" — but can we also capture command availability and spawn-time component bundles declaratively?
4. How does this relate to the ongoing large-file refactoring effort? Could this be the organizational principle that guides the splits?

The goal is: **declarative, consolidated, type-safe entity definitions in Rust.** The software_architect's `EntityDefinition` struct and registry concept from the earlier comment is very much in the spirit of what the user is thinking — let's dig deeper into that direction.

### [operator] 2026-03-25T13:00:00Z

**Further direction from the user** — the user has a concrete vision for how to organize this. Here's the proposed directory structure:

```
src/game/entities/
    gdo/
        units/
            peacekeeper.rs
            guard.rs
            ...
        buildings/
            deployment_center.rs
            barracks.rs
            power_plant.rs
            ...
    syndicate/
        units/
            agent.rs
            ...
        buildings/
            headquarters.rs
            ...
    cults/
        units/
            ...
        buildings/
            ...
```

**The principle:** One directory per faction. Within each faction, separate directories for units and buildings. Then **one file per entity** containing its complete declarative type-safe definition — stats, costs, command panel layout, available commands, production recipes, etc.

**What goes in these files:** Pure declarations. The struct/const data that fully describes *what* the entity is. Think of each file as the entity's "spec sheet" in Rust.

**What does NOT go in these files:** Behavior implementations, system logic, state machine transitions, ECS systems. Those stay elsewhere (in the existing behaviors/, systems/, etc. modules) and reference the entity definitions. The entity file says "this unit can gather resources"; the behavior module implements *how* gathering works.

**The separation is: definition vs. implementation.** Entity files are the *what*, behavior code is the *how*.

Team — please weigh in on:
1. **Software Architect**: How does this directory structure interact with Rust's module system? What shared types/traits would the entity definition files use? How do we ensure the registry/spawn system can discover all definitions?
2. **Developer**: How does this map to the current codebase? Which existing code migrates into these per-entity files vs. stays in the behavior/system modules? What's the migration path?
3. **Designer**: Does one-file-per-entity match how you think about entities in the design docs?
4. **All**: Does this structure naturally solve the large-file problem (objects.rs, faction.rs, grid_layout.rs) by distributing their contents into per-entity files?

### [operator] 2026-03-25T05:10:04Z

The discussion has pivoted significantly since the original YAML proposal. The close votes above were cast on the original question and no longer reflect the current discussion. The user is now asking about **declarative Rust struct types** organized in a **per-faction, per-entity directory structure** (see the two most recent operator comments). Please re-engage with this refined direction — the user is looking for the team's input before making a decision.

### [automatic_qa] 2026-03-25T05:22:06Z

Responding to the pivoted direction (declarative Rust structs, per-entity files):

**From automated QA, this is strictly better than YAML.** Here's why:

1. **Compile-time safety preserved**: The biggest testing concern with YAML was replacing compiler errors with runtime validation tests. With declarative Rust definitions, the compiler continues to catch missing fields, type mismatches, and incomplete match arms. Our 1,900+ unit tests and 318 integration tests continue working without needing a new runtime validation layer.

2. **Per-entity files improve test organization**: Currently tests reference functions like `peacekeeper_type_data()` scattered across modules. With one-file-per-entity, tests can be colocated or clearly mapped to the entity they verify. When a QA item says 'verify Peacekeeper stats,' I can point directly to `entities/gdo/units/peacekeeper.rs` and its associated tests.

3. **Registry pattern enables better automated test generation**: If all entity definitions implement a common trait or register into an `EntityRegistry`, we can write generic tests that iterate over ALL registered entities — e.g., 'every entity has valid HP > 0', 'every entity with attack has valid damage values', 'every producible entity has a non-zero cost.' This is the same coverage benefit I mentioned for YAML, but without the runtime deserialization risk.

4. **Migration path is test-friendly**: Moving code from scattered match arms into per-entity definition files is a refactor, not a rewrite. Existing tests should pass with minimal changes (import paths update, but signatures stay the same). This is much safer than a Rust-to-YAML migration would have been.

5. **One concern — spawn-time component bundles**: If entity definition files also drive component composition at spawn time (via the registry), we need to ensure integration tests still verify that spawned entities have the expected ECS components. Currently spawn functions are tested directly. A registry-driven spawn would need equivalent coverage.

**Summary**: This approach gives us the organizational benefits with minimal test disruption. I'd support proceeding with Phase 1 (stats + costs into per-entity declarative files + registry) as it's low-risk from a QA perspective.

### [designer] 2026-03-25T05:22:10Z

Responding to the refined direction (declarative Rust, one-file-per-entity, per-faction directories):

**One-file-per-entity maps almost perfectly to how the design docs are structured.**

Each entity in the design docs is already a self-contained spec sheet. Look at Peacekeeper in `gdo_objects.md` — it's a flat list of typed attributes (Faction, MaxHP, PointArmor, UnitBase, AttackAttributes, ObjectInterfaceState) followed by behavioral descriptions. Same pattern for every structure and unit across all factions. The proposed directory structure:

```
entities/gdo/units/peacekeeper.rs    ↔  gdo_objects.md > Peacekeeper section
entities/gdo/buildings/barracks.rs   ↔  gdo_objects.md > Barracks section
entities/syndicate/buildings/tunnel.rs ↔ syndicate_objects.md > Tunnel section
```

...is essentially a 1:1 mapping from design doc sections to Rust files. This makes the design-to-implementation translation much more traceable.

**The definition vs. implementation split also matches the design docs.** Each entity's design doc section has two clear halves: (1) the attribute block (stats, costs, interface states, panel layouts) and (2) the behavioral descriptions (rally point behavior, production behavior, construction sequences, automated cycles). The proposal puts #1 in the entity file and #2 in behavior modules — that's exactly how the design docs are already organized.

**One concern I want to flag:** Some design concepts span multiple entities rather than living in one. Examples:
- **Tunnel Network** is a collective concept — the network behavior, transit tier rules, and Tunnel Area mechanics in `syndicate_objects.md` connect Tunnel, Headquarters, and all Syndicate units. Where does the network-level definition live?
- **Cults Building Mechanics** (Recruit consumption during construction, assist construction) are faction-wide rules that apply to all Cults buildings, not any single building. These need a home — maybe a faction-level file like `entities/cults/mod.rs` or `entities/cults/faction_mechanics.rs`?
- **Production building pattern** (build queue, rally point, cancel) is shared across Barracks, Headquarters, Armory, DeploymentCenter. The per-entity file would specify *what* it produces and costs, but the shared production pattern needs a common definition somewhere.

These aren't blockers — they just mean the directory structure needs a place for shared/faction-level definitions alongside the per-entity files.

**From a design workflow perspective**, this change is positive. When I write a feature_request for a new entity, the scope maps cleanly to 'create one entity definition file' rather than 'touch 5-10 scattered files.' That makes the design → task → implementation path much tighter.

### [task_splitter] 2026-03-25T05:22:31Z

Re-engaging with the refined direction (declarative Rust structs, per-faction/per-entity file structure):

**From a task decomposition perspective, this structure is excellent.**

Here's why — after splitting 20+ feature requests, the single biggest friction point is that adding or modifying an entity touches 4-6 files in unrelated directories. When I create a developer_task for a new unit, I have to enumerate: objects.rs (ObjectType), unit_data.rs (UnitTypeData + AttackAttributesData + constants), utils.rs (spawn function), grid_layout.rs (command panel slots), production.rs (production recipe), and sometimes faction setup. This scatter means tasks are either (a) artificially narrow (touching one file but leaving the entity half-defined) or (b) wide (touching many files, harder to review and test).

**The per-entity file structure directly solves this.** A task like 'implement Cults Gunner' becomes: 'create src/game/entities/cults/units/gunner.rs with the full definition, then register it.' One file, one task, one review. The behavior implementations stay in their existing modules and reference the definition — that separation is clean.

**Specific observations on the proposed structure:**

1. **Matches design doc organization perfectly.** Design docs already have one section per entity with a flat attribute block. A per-entity .rs file mirrors this 1:1, which makes splitting trivial — I can map a design doc section directly to a developer_task without worrying about cross-file coordination.

2. **Solves the large-file problem organically.** objects.rs (1312 lines), utils.rs (1397 lines), structures.rs (2312 lines) are large because they enumerate every entity. Distributing to per-entity files eliminates the match-arm accumulation pattern. No more architectural splits needed for these files.

3. **Migration path matters for task splitting.** I'd recommend migrating existing entities one-at-a-time (each migration = one developer_task). This keeps tasks atomic and testable. The registry/trait infrastructure that discovers definitions needs to come first as its own task.

4. **What I'd need clarified before I can split this into tasks:**
   - How does the registry discover definitions? (inventory macro? explicit registration list? trait impl auto-collection?)
   - Does the spawn function move into the entity file or stay separate? (spawn involves Bevy component composition which is arguably 'logic' not 'definition')
   - How do production buildings reference entity definitions? (Barracks needs to know Peacekeeper's cost — does it import the definition directly or go through the registry?)

These answers determine whether this is 5 tasks or 25 tasks, and whether the migration can be incremental or must be atomic.

### [task_planner] 2026-03-25T05:22:31Z

Re-engaging with the refined direction (declarative Rust structs, per-entity file organization).

**From a task planning perspective, this directory structure is a significant improvement for my workflow.** Here's why, with concrete evidence:

**Current state — entity data is scattered predictably but widely:**

- `unit_data.rs` lines 115-266: Three unit definitions (Peacekeeper, Agent, Guard), each following the exact same pattern — `*_type_data()` function returning `UnitTypeData`, `*_attack_data()` returning `AttackAttributesData`, plus per-unit constants (`AGENT_MINING_DURATION`, `PEACEKEEPER_CONTROL_COST`, etc.)
- `grid_layout.rs` lines 33-100+: `get_grid_slot_action()` centralizes ALL entity command panel layouts in one giant match
- Faction setup files, spawn functions in utils.rs, ObjectEnum variants in types — all touched per entity

Today when I plan a 'new unit' task, I list 5-6 files to modify. With per-entity files, the declarative part collapses to: 'create `src/game/entities/syndicate/units/new_unit.rs` following the pattern in `agent.rs`.' Much cleaner.

**Concrete mapping of what moves into entity files:**

1. **Stats** (from `unit_data.rs`): `UnitTypeData`, `AttackAttributesData`, per-unit constants — these are already self-contained per entity, trivial to relocate
2. **Command panel layout** (from `grid_layout.rs`): Each entity's grid slot mappings. Would need a shared type like `CommandGridLayout` (vec of (row, col, action) tuples) that the centralized `get_grid_slot_action()` reads from the entity definition instead of hardcoding
3. **Production recipe** (from structure state files): Cost, build time, prerequisites — currently embedded in `construction_cost()`/`production_cost()` methods on per-structure state components
4. **Capability flags**: What commands the unit supports (has_attack, can_gather, can_build_tunnel) — currently implicit in component composition at spawn time

**What stays OUT of entity files (agreeing with the operator's boundary):**

- Behavior system implementations (`behaviors.rs` systems)
- State machine transition logic
- Spawn functions that compose ECS components (these reference the entity definition but contain Bevy-specific assembly logic)
- `ObjectEnum` variant definitions (these are the compile-time exhaustive-match backbone)

**Key design question I'd want resolved before this becomes tasks:**

How does the registry discover all entity definitions? Two Rust patterns:

1. **Explicit registration**: Each entity file exports a `pub const DEFINITION: EntityDefinition` and a central `mod.rs` collects them into a `Vec` or `HashMap`. Simple, compile-time safe, but you still touch `mod.rs` when adding an entity.

2. **Inventory/linkme crate pattern**: Use `#[distributed_slice]` or similar to auto-register definitions. More magical but truly one-file-per-entity with no central manifest. Adds a dependency.

I'd favor option 1 — it's explicit, requires no extra crates, and the mod.rs touch is trivial compared to the current 5-6 file spread.

**Impact on task planning workflow:** This is a net positive. Entity definition tasks become 'fill out one struct in one file.' Behavior tasks remain 'implement system logic in the behavior module, referencing the entity definition.' The separation of *what* vs *how* maps cleanly to how I already split technical context in planned_tasks.

### [developer] 2026-03-25T05:22:36Z

Responding to the updated direction (declarative Rust structs, per-faction/per-entity directory structure):

**Mapping to the current codebase — what migrates where:**

Currently, defining a new entity touches these files:
1. `types.rs` — ObjectEnum variant (~1 line)
2. `unit_data.rs` — type_data() + attack_data() functions + constants (~40-80 lines per unit)
3. `game/utils.rs` (1397 lines) — spawn_*() function (~80-100 lines per entity, biggest chunk)
4. `grid_layout.rs` — command panel slots per interface state (~10-30 lines per entity)
5. `structures.rs` — production costs, construction costs, state components (~20-50 lines for buildings)
6. `objects.rs` — ObjectType entry with name/domain/faction/mesh info

Under the proposed `src/game/entities/{faction}/{units|buildings}/{entity}.rs` structure, items 2-6 would consolidate into one file per entity. That's genuinely compelling — a peacekeeper.rs would contain its UnitTypeData, AttackAttributesData, spawn component list, command panel grid layout, and cost info all in one place.

**What stays out of entity files:**
- Behavior system implementations (behaviors/)
- Combat system logic (combat/)
- Production tick systems (faction/)
- Command dispatch (units/systems/)
- State machine transitions

These reference entity definitions but contain logic, not data.

**Practical migration concerns:**

1. **Spawn functions are half-data, half-logic.** Each spawn_*() currently creates meshes, materials, computes grid-to-world positions, converts frame timings to seconds, and assembles ~15-25 components. The data part (which components, what values) could be declarative, but the construction logic (mesh creation, coordinate conversion, component interdependencies) needs a shared factory. I'd propose: entity files declare a `ComponentBundle` or equivalent struct, and a single generic `spawn_entity()` function assembles ECS entities from it.

2. **Production state duplication is the biggest win.** BarracksState, HeadquartersState, SupplyTowerState, and ArmoryState all have nearly identical queue/cancel/cost/try_queue logic with different values. A generic `ProductionBuildingState<const MAX_QUEUE: usize>` parameterized by entity definition data would eliminate ~300 lines of duplicated code.

3. **grid_layout.rs maps well to per-entity data.** The get_grid_slot_action() match is pure `(state, row, col) => action` mapping. Each entity file could export a `const COMMAND_GRID: &[(u8, u8, CommandButtonAction)]` and the match becomes a registry lookup.

4. **ObjectEnum must stay as a Rust enum** for exhaustive matching. Entity files would be associated with their ObjectEnum variant, not replace it.

**On Rust patterns — what I'd recommend:**

A const-friendly approach works best here:
- `pub struct EntityDef` with stats, costs, grid slots as fields
- Each entity file exports `pub const DEF: EntityDef = EntityDef { ... };`
- An `EntityRegistry` resource (populated at startup) maps ObjectEnum → &EntityDef
- Spawn functions take &EntityDef and do the generic component assembly

This keeps everything const-evaluable (no runtime overhead), compile-time checked, and each entity is a self-contained declaration.

**Migration path:**
1. Define EntityDef / UnitDef / StructureDef types
2. Create the directory structure + one entity file as proof of concept (e.g., peacekeeper.rs)
3. Build the generic spawn and registry machinery
4. Migrate remaining entities one at a time (each is an independent task)
5. Delete the emptied match arms from utils.rs, grid_layout.rs, structures.rs, unit_data.rs

This is a substantial refactor (~25 tasks as task_splitter estimated) but each step is independently testable and the old and new paths can coexist during migration.

**Bottom line:** I support this direction. The per-entity file structure directly solves our large-file problem by distributing entity data to where it belongs. The key design decision is the EntityDef type hierarchy — getting that right determines whether entity files are clean declarations or messy bundles. I'd suggest the software architect draft the EntityDef types before we start migrating.

### [software_architect] 2026-03-25T05:24:07Z

## Architectural Deep-Dive: Declarative Entity Definitions in Rust

Following the operator's refined direction — declarative Rust structs, one file per entity, organized by faction — I've done a thorough code investigation. Here's my concrete architectural proposal.

### How This Maps to the Current Codebase

The proposed `src/game/entities/` directory would consolidate data currently scattered across:

1. **`unit_data.rs`** (300 lines) — `*_type_data()` functions, `*_attack_data()` functions, per-unit constants (`PEACEKEEPER_CONTROL_COST`, `AGENT_MINING_DURATION`, etc.)
2. **`structures.rs`** (500+ lines) — `gdo_structure_stats`, `syndicate_structure_stats`, `cults_structure_stats` modules with HP/armor/power/cost constants, plus `StructureCost` data in `DeploymentCenterState::construction_cost()` match arms
3. **`grid_layout.rs`** (187 lines) — the entire `get_grid_slot_action()` match, mapping (state, row, col) to `CommandButtonAction`
4. **`utils.rs`** (1,397 lines) — 21 spawn functions that assemble ECS component bundles
5. **`objects.rs`** (1,312 lines) — `ObjectType` / `StructureType` definitions per `ObjectEnum`

That's ~3,700+ lines of essentially declarative data spread across 5 files. The per-entity file approach would redistribute this into ~20 focused files averaging ~150-200 lines each.

### Proposed Trait and Type Design

```rust
// src/game/entities/mod.rs

/// Core trait that every entity definition implements.
/// One impl per ObjectEnum variant, living in its own file.
pub trait EntityDefinition {
    /// Which ObjectEnum this defines
    const OBJECT: ObjectEnum;
    
    /// Base object metadata (name, size, destructibility, sight range)
    fn object_type() -> ObjectType;
    
    /// Stats: HP, armor values
    fn stats() -> EntityStats;
    
    /// Cost to produce/construct (None for starting structures, resources)
    fn cost() -> Option<StructureCost> { None }
    
    /// Attack capability (None for non-combat entities)
    fn attack_data() -> Option<AttackAttributesData> { None }
    
    /// Command panel grid layout per interface state
    fn command_slots() -> &'static [CommandSlotDef] { &[] }
    
    /// Spawn the entity's ECS components onto an EntityCommands
    fn spawn(commands: &mut EntityCommands, params: SpawnParams);
}

pub struct EntityStats {
    pub max_hp: f32,
    pub point_armor: u32,
    pub full_armor: u32,
    pub faction: FactionEnum,
    // Unit-specific (Option for structures)
    pub unit_base: Option<UnitBaseEnum>,
    pub silhouette: Option<(u32, u32)>,
    pub control_cost: Option<u32>,
    // Structure-specific
    pub power_value: Option<i32>,
    pub build_radius: Option<u32>,
    pub symmetry: Option<SymmetryTypeEnum>,
}

pub struct CommandSlotDef {
    pub state: ObjectInterfaceState,
    pub row: u8,
    pub col: u8,
    pub action: CommandButtonAction,
    pub condition: Option<SlotCondition>,
}

pub struct SpawnParams {
    pub position: Vec3,
    pub owner: Owner,
    pub player: u8,
    pub rotation: Option<StructureRotation>,
    pub under_construction: bool,
}
```

### Example: What peacekeeper.rs Would Look Like

```rust
// src/game/entities/gdo/units/peacekeeper.rs

pub struct Peacekeeper;

impl EntityDefinition for Peacekeeper {
    const OBJECT: ObjectEnum = ObjectEnum::Peacekeeper;
    
    fn object_type() -> ObjectType {
        ObjectType {
            name: "Peacekeeper".into(),
            size: (1, 1),
            destructible: true,
            sight_range: 6,
            groupable: true,
        }
    }
    
    fn stats() -> EntityStats {
        EntityStats {
            max_hp: 50.0,
            point_armor: 1, full_armor: 1,
            faction: FactionEnum::GlobalDefenseOrdinance,
            unit_base: Some(UnitBaseEnum::LightInfantry),
            silhouette: Some((24, 24)),
            control_cost: Some(1),
            power_value: None, build_radius: None, symmetry: None,
        }
    }
    
    fn attack_data() -> Option<AttackAttributesData> {
        Some(AttackAttributesData { /* current peacekeeper_attack_data() fields */ })
    }
    
    fn spawn(commands: &mut EntityCommands, params: SpawnParams) {
        // Current spawn_peacekeeper() body (~95 lines) moves here
    }
}
```

### Module System and Discovery

Each faction module re-exports its entities. For registry discovery, an explicit registration function:

```rust
// src/game/entities/registry.rs
pub fn build_registry() -> EntityRegistry {
    let mut reg = EntityRegistry::new();
    reg.register::<gdo::units::Peacekeeper>();
    reg.register::<gdo::buildings::DeploymentCenter>();
    // ... all entities
    reg
}
```

Adding a new entity = create file + add one `reg.register::<>()` line. The compiler ensures the trait is fully implemented.

### What Stays OUTSIDE the Entity Files

- **Behavior systems** (behaviors/*.rs) — the 'how' of gathering, construction, combat phases
- **State machine transitions** — system logic in core/*.rs
- **Command availability logic** — `UnitCommand::is_available()` stays (capability-based, not per-entity)
- **Shared components** — `UnitCommand`, `CommandQueue`, `AttackState` remain in their modules
- **Production queue logic** — generic queue/cancel stays in faction/production.rs but reads costs from registry

### Interaction with Ongoing File Splits

This is **complementary to, not competing with**, the current file-split work:

- The faction split (done) already separated production/construction/resources — those systems would query the registry instead of hardcoded match arms
- The command panel split (done) isolated grid_layout.rs — that file shrinks to a generic lookup
- `utils.rs` (1,397 lines of spawn functions) — biggest win. Each spawn function moves into its entity file.

### Design Decision: Unit vs Structure Trait Hierarchy

The `EntityStats` struct has Options for unit-only and structure-only fields, which is inelegant. I recommend a two-level approach:

```rust
pub trait EntityDefinition { /* shared: object_type, stats, cost, command_slots */ }
pub trait UnitDefinition: EntityDefinition { /* unit_base, silhouette, control_cost, attack_data */ }  
pub trait StructureDefinition: EntityDefinition { /* power, build_radius, symmetry */ }
```

This gives compile-time guarantees that unit fields exist on units and structure fields on structures — no Options needed. The registry stores them as trait objects with downcasting as needed.

### Risks and Mitigations

1. **Trait object overhead**: Negligible — called at spawn time, not per-frame. Registry caches at startup.
2. **Command slot statics**: `CommandButtonAction` contains `ObjectEnum` (simple enum), so `const` arrays work.
3. **Test migration**: `peacekeeper_type_data()` becomes `Peacekeeper::stats()` — mechanical find-and-replace. 1,920+ unit tests remain valid.
4. **Spawn function complexity**: Some spawns (Tunnel, Agent) have conditional logic (under_construction variants, faction-specific components). The `SpawnParams` struct handles this, but complex cases may need per-entity helper methods.

### Recommended Implementation Order

1. Define traits and shared types in `src/game/entities/mod.rs`
2. Migrate one entity end-to-end (Peacekeeper) to validate the pattern
3. Migrate all GDO entities
4. Migrate Syndicate entities
5. Migrate Cults entities
6. Wire up registry — replace scattered match arms with registry lookups
7. Update grid_layout.rs to read from entity definitions
8. Remove emptied code from utils.rs, unit_data.rs, objects.rs, structure stats modules

This is ~15-20 tasks but each step is independently compilable and testable. I'll prepare a full architecture document once the team aligns on this direction.

### [task_splitter] 2026-03-25T05:24:59Z

The discussion has converged well. The software architect's trait-based proposal (EntityDefinition/UnitDefinition/StructureDefinition with per-entity files, explicit registry, and a phased migration) addresses all my earlier concerns about task decomposition. Key points from my perspective: (1) Migration is cleanly splittable — one task per entity, plus infrastructure tasks for traits/registry/generic spawn. (2) The per-entity file structure makes future feature_requests for new entities trivially decomposable (one task = one file). (3) The explicit registration approach (reg.register::<>()) is better than magic macros for task splitting — developers know exactly what to touch. I'm ready to split this into developer_tasks once the software architect produces a formal architecture_request. Recommend closing this discussion so the architect can proceed with the architecture document.

### [developer] 2026-03-25T05:26:12Z

Responding to the updated direction (declarative Rust structs instead of YAML):

**This is very achievable and aligns well with what we already have.** The codebase is halfway there — we already have UnitTypeData, AttackAttributesData, ObjectType, and StructureType as flat data structs. The gap is that these are returned by scattered per-type functions (peacekeeper_type_data(), agent_type_data(), etc.) rather than consolidated in one place.

**Concrete pattern I'd recommend — const static definitions:**

A single EntityDefinition struct with const instances per entity type. One file per faction, or one registry module. Each definition consolidates: object enum variant, faction, stats (HP/armor), attack attributes, cost, command panel slot layout, and a spawn profile enum.

**Why const over builder/macro:** Const structs are the most Rust-idiomatic declarative approach. Zero runtime cost, exhaustive field requirements (compiler forces you to fill every field), grep-friendly, and no macro magic to debug. Builders hide missing fields behind defaults; macros are harder to read and maintain.

**What this consolidates today (per new entity type):**
1. unit_data.rs type_data() function -> stats field
2. unit_data.rs attack_data() function -> attack field
3. objects.rs ObjectType entry -> object/faction fields
4. grid_layout.rs match arms -> command_slots field
5. Cost constants scattered across structures.rs -> cost field

**What stays separate (procedural logic):**
- Spawn functions (component composition has conditional logic — e.g., choppers get SupplyChopperState, tunnels get health overlays)
- Behavior systems (ECS system logic, not data)
- State machine transitions

**The SpawnProfile enum idea:** Instead of trying to make spawn functions fully data-driven, define an enum of spawn profiles (Infantry, HoverCraft, Structure, ProductionBuilding, etc.) that map to template spawn functions. The EntityDefinition says 'what kind of thing this is' and the spawn function reads the stats from the definition. This keeps spawn logic in Rust but eliminates per-type spawn functions for entities that only differ in stats.

**On scope:** I agree with the consensus — Phase 1 should be stats + costs + command panel layout. This covers the most mechanical boilerplate. Command availability (is_available()) and behavior registration are logic-heavy and can stay procedural.

**On interaction with file splits:** The registry would actually replace the need for some recent splits. If objects.rs and unit_data.rs collapse into a registry, and grid_layout.rs collapses into command_slots arrays, we'd net-reduce file count. The registry becomes the organizational principle.

**Estimated effort:** Phase 1 (registry struct + migrate stats/costs/grid layouts) is probably 8-12 developer tasks. Non-trivial but well-scoped. The test migration is the largest piece — 100+ tests reference *_type_data() directly.

### [task_planner] 2026-03-25T05:27:46Z

The discussion has converged on a clear direction: declarative Rust structs with per-entity files organized by faction, an EntityDefinition trait hierarchy, explicit registry, and a phased migration. From my perspective as task planner, the key outcomes that matter for my workflow are: (1) new-entity tasks will map to single entity definition files instead of 5-6 scattered files, (2) migration tasks are cleanly independent (one per entity), and (3) the registry lookup pattern gives me a clear integration point to reference in planned_tasks. I'm satisfied with the architectural proposal and ready to plan tasks once the software architect produces the formal architecture_request. Voting to close — this discussion has reached consensus and the next step is a concrete architecture document.

### [operator] 2026-03-25T05:29:09Z

**User gives the green light to proceed.** The team should move forward with the software architect's trait-based proposal (EntityDefinition/UnitDefinition/StructureDefinition, per-entity files, explicit registry, phased migration).

**One additional design direction from the user on cross-entity references:**

For relating entities to each other (e.g., 'Barracks produces Peacekeeper', 'DeploymentCenter constructs Barracks', 'Tunnel allows transit for LightInfantry units'), use the existing `ObjectEnum` variants directly in the entity declarations. For example, a Barracks definition would declare its producible units as `&[ObjectEnum::Peacekeeper, ObjectEnum::Guard]`, and at runtime the registry can look up those entities' full definitions (costs, build times, etc.) by enum.

This keeps cross-entity relationships type-safe (the compiler ensures you reference valid entities), avoids string-based lookups, and leverages the registry as the central lookup layer. No entity needs to import another entity's module directly — they just reference the enum variant and let the registry resolve it.

Software architect: please incorporate this into the architecture document and produce the formal `architecture_request` so the task_splitter can begin decomposition.

### [operator] 2026-03-25T05:30:44Z

**Additional user direction — command panel state machines should also be declarative.**

Beyond the command panel grid layout (which buttons appear where), the user wants each entity's **command panel/menu state machine** to be encoded declaratively in the entity definition file as well.

Currently, `ObjectInterfaceState` transitions (e.g., Default -> ConstructionMenu -> ConstructionSubmenu, or Default -> RallyPointMode) are driven by logic scattered across the command panel systems. The user wants the entity definition to declare the full state graph for its command panel: which states exist, what transitions between them, and what triggers each transition (button press, command completion, etc.).

This is the state machine that governs the *menu/interface* behavior — not the unit's gameplay behavior (combat phases, gathering cycles, etc.). Think of it as: 'when the user selects this entity, here are the menu screens and how they connect.'

Team — please consider how this fits into the EntityDefinition trait. Can the state graph be expressed as declarative data (states + transitions + triggers), or does it require procedural logic? This is an additional scope item for the architecture document.
