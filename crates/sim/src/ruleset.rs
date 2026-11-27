//! The ruleset (plan decision T66, slice M5-X1): every number the sim reads, in one value: the object types' defs,
//! the unit bases, the factions' own rules and the systems' tunables. The design's numbers are the built-in ruleset
//! ([`Ruleset::BUILTIN`]), made from the def modules' defs and the tuning modules' `BUILTIN`s, with their citations;
//! a game may replace it, between two ticks, by `DebugCommand::SetRules` (slice M5-X1b; the data files' tuning layer,
//! slice M5-X2), but not its structure ([`Ruleset::structural_diff`]).
//!
//! - **Reading.** `kind.def()`, `base.def()`, `tech.def()` and `ability.def()` read the current rules, and a tunable is
//!   `rules().tuning.<system>.<field>`. A hot loop reads its tunables once, before the loop, and hands its helpers
//!   what they need (`elevation::sight_range_by`): each read of [`rules`] is an atomic load and a branch, and a number
//!   the compiler can't see costs what a constant didn't (slice M5-X1: 2 to 5 % of the perf tests' ticks before the
//!   hoisting; the ElevationModifier read for every tile of the sight stamp, most of `movement_perf`'s).
//!
//! - **Generations.** A ruleset in use is a generation of the process, numbered: the built-in rules are generation 0,
//!   a `static` ([`Ruleset::BUILTIN`]), and every other is leaked, so that every `&'static` def handed out stays valid
//!   forever, into [`GENERATIONS`], at most [`GENERATION_COUNT`] in all. [`rules`] reads the current one, with no
//!   lock and no `unsafe`: one atomic load, and for another generation than the built-in one a `OnceLock`'s.
//!   `ObjectKind::def`, `UnitBase::def`, `Tech::def` and `AreaAbility::def` read it.
//! - **Determinism.** The sim names its generation (`Sim::rules`) and makes it the current one at the start of every
//!   tick ([`select`]), so it reads one generation for a whole tick and a change applies at a tick boundary, in
//!   command order: `SetRules` installs its ruleset as the command is applied, and the next tick starts on it. The
//!   ruleset travels inside the command, so a replay reproduces a tuned game. The state hash doesn't include the
//!   rules. A reader on another thread (the client's) sees an old or a new generation, both valid.
//! - **One sim at a time on other rules.** The current generation is the process's, so two sims stepping at once on
//!   two generations would read each other's. The game runs one sim; the tests run many, all on the built-in rules,
//!   and only one test program installs others, its tests one at a time (`tests/rules.rs`).

mod structure;
mod table;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use serde::{Deserialize, Serialize};

pub use structure::StructuralChange;
pub use table::{Key, Table};

use crate::defs::{FactionRules, ObjectDef, ObjectKind, UnitBase, UnitBaseDef};
use crate::tuning::{self, Tuning};

/// Every object type's def, by kind.
pub type Defs = Table<ObjectKind, ObjectDef, { ObjectKind::COUNT }>;

/// Every unit base's def, by base.
pub type Bases = Table<UnitBase, UnitBaseDef, { UnitBase::COUNT }>;

/// Every number the sim reads (module docs). In files: `defs` and `bases` are maps by kind and by base, the rest
/// structs by field name, each serialized as its Rust type is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ruleset {
    /// The object types' defs (`crate::defs`).
    pub defs: Defs,
    /// The unit bases' defs (`systems/units.md > UnitBase`).
    pub bases: Bases,
    /// The factions' own rules: the caps, the Cults' augmentation table, techs and abilities.
    pub factions: FactionRules,
    /// The systems' tunables (`crate::tuning`).
    pub tuning: Tuning,
}

impl Ruleset {
    /// The design's numbers: every def module's def, every base's, every faction's rules and every tuning module's
    /// `BUILTIN`. Generation 0.
    pub const BUILTIN: Ruleset = Ruleset {
        defs: Table::new(ObjectKind::BUILTIN_DEFS),
        bases: Table::new(UnitBase::BUILTIN_DEFS),
        factions: FactionRules::BUILTIN,
        tuning: tuning::BUILTIN,
    };

    /// The built-in rules ([`Ruleset::BUILTIN`]), to change.
    pub fn builtin() -> Ruleset {
        BUILTIN.clone()
    }

    /// The built-in rules, generation 0's.
    #[inline]
    pub fn builtin_ref() -> &'static Ruleset {
        &BUILTIN
    }

    /// The def of `kind` in these rules.
    #[inline]
    pub fn def(&self, kind: ObjectKind) -> &ObjectDef {
        &self.defs[kind]
    }

    /// Where `other`'s structure first differs from these rules' ([`structure`]'s rule: every field that isn't a
    /// number, and the sizes of the footprints and Silhouettes); `None` when only numbers differ, so that `other` can
    /// replace these rules in a running game.
    pub fn structural_diff(&self, other: &Ruleset) -> Option<StructuralChange> {
        structure::Skeleton::of(self).diff(&structure::Skeleton::of(other))
    }
}

/// The most generations a process installs (module docs): the built-in rules and 63 others. A `SetRules` beyond them
/// is refused (`Reject::RulesFull`); a ruleset equal to one installed is that one again.
pub const GENERATION_COUNT: usize = 64;

/// Generation 0, the built-in rules.
static BUILTIN: Ruleset = Ruleset::BUILTIN;

/// The installed generations but the built-in one (module docs), from 1; slot 0 stays empty.
static GENERATIONS: [OnceLock<&'static Ruleset>; GENERATION_COUNT] = [const { OnceLock::new() }; GENERATION_COUNT];

/// The current generation, which [`rules`] reads: an installed one's number.
static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// Taken while a generation is installed, so that two installs don't take the same number.
#[expect(
    dead_code,
    reason = "slice M5-X1b: `DebugCommand::SetRules` installs generations and the sim selects its own"
)]
static INSTALLING: Mutex<()> = Mutex::new(());

/// A generation of rules (module docs): its number in [`GENERATIONS`]. A sim's own, not part of its state: saved as
/// the ruleset itself when it isn't the built-in one (`Sim::rules`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Generation(u8);

impl Generation {
    /// The built-in rules ([`Ruleset::builtin`]).
    pub const BUILTIN: Generation = Generation(0);

    /// Whether it is the built-in rules.
    pub fn is_builtin(&self) -> bool {
        *self == Generation::BUILTIN
    }

    /// Its rules.
    pub fn rules(self) -> &'static Ruleset {
        installed(usize::from(self.0))
    }
}

/// The current rules (module docs): those of the generation the sim stepping now selected, the built-in ones until
/// one does.
#[inline]
pub fn rules() -> &'static Ruleset {
    // Relaxed: the number orders nothing else. Another generation's rules are published by its `OnceLock`, which
    // `installed` reads with its own ordering; an Acquire here would keep the compiler from keeping other values in
    // registers across every def read, in the sim's hottest loops.
    match CURRENT.load(Ordering::Relaxed) {
        0 => &BUILTIN,
        current => installed(current),
    }
}

/// The rules of generation `number`: an installed one's (`select` takes only a number `install` returned).
#[cold]
#[inline(never)]
fn installed(number: usize) -> &'static Ruleset {
    match GENERATIONS[number % GENERATION_COUNT].get() {
        Some(rules) => rules,
        None => &BUILTIN,
    }
}

/// Makes `generation` the current one (the sim, at the start of each tick: module docs).
#[inline]
#[expect(
    dead_code,
    reason = "slice M5-X1b: `DebugCommand::SetRules` installs generations and the sim selects its own"
)]
pub(crate) fn select(generation: Generation) {
    let number = usize::from(generation.0);
    if CURRENT.load(Ordering::Relaxed) != number {
        CURRENT.store(number, Ordering::Release);
    }
}

/// The generation of `ruleset`: the one installed with equal rules, or a new one; `None` when all
/// [`GENERATION_COUNT`] are taken by others.
#[expect(
    dead_code,
    reason = "slice M5-X1b: `DebugCommand::SetRules` installs generations and the sim selects its own"
)]
pub(crate) fn install(ruleset: &Ruleset) -> Option<Generation> {
    let _installing = INSTALLING.lock().unwrap_or_else(PoisonError::into_inner);
    if *ruleset == BUILTIN {
        return Some(Generation::BUILTIN);
    }
    for (number, slot) in GENERATIONS.iter().enumerate().skip(1) {
        let generation = Generation(u8::try_from(number).expect("64 generations"));
        match slot.get() {
            Some(rules) if **rules == *ruleset => return Some(generation),
            Some(_) => {}
            None => {
                let rules: &'static Ruleset = Box::leak(Box::new(ruleset.clone()));
                slot.set(rules).expect("the slot is free while installing");
                return Some(generation);
            }
        }
    }
    None
}

/// Whether [`install`] would find `ruleset` a generation: one is free, or one has equal rules.
#[expect(
    dead_code,
    reason = "slice M5-X1b: `DebugCommand::SetRules` installs generations and the sim selects its own"
)]
pub(crate) fn installable(ruleset: &Ruleset) -> bool {
    let _installing = INSTALLING.lock().unwrap_or_else(PoisonError::into_inner);
    *ruleset == BUILTIN || (GENERATIONS.iter().skip(1)).any(|slot| slot.get().is_none_or(|rules| **rules == *ruleset))
}

#[cfg(test)]
mod tests;
