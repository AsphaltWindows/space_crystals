//! The Colonists' computer player (`factions/colonists/overview.md`): none yet. Slice A2c writes it here and sets
//! `HOOK`; until then the command line refuses a computer player for a Colonists player (`crate::AiError::NoAi`).

use super::Hook;

/// None yet (module docs).
pub(super) const HOOK: Option<Hook> = None;
