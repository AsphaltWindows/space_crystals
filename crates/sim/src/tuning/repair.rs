//! Repair (slice M5-N8, plan decision T83; `crate::repair`): how a repair's time and price follow from what built its
//! target. Who repairs what, and whether they pay, is each repairer's def (`defs::WorkerDef::repairs`).

/// "**Placeholder rate and cost** (designer, 2026-11-26): repairing from 0 to full HP takes **half the target's build
/// time** with one repairer" (`factions/cults/units/recruit.md > Repair`, `factions/syndicate/units/agent.md >
/// Repair`): the share of its build time that one repairer takes to restore a target's whole MaxHP. **Placeholder**
/// (design change 0321).
pub const REPAIR_TIME_SHARE: f32 = 0.5;

/// The same placeholder's "and costs **half its price** in the resource it was bought with, paid gradually as HP is
/// restored": the share of its price that restoring a target's whole MaxHP costs. **Placeholder** (design change
/// 0321).
pub const REPAIR_COST_SHARE: f32 = 0.5;
