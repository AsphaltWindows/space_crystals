//! (Slice M5-X2, plan decision T66) The data files: the rules and the client's feel tunables as JSON files, tuned while
//! the game runs. "Modules now, data files at playtest" (design change 0282: "Once testing starts we want to
//! experiment and make changes rapidly and accurately"): the Rust defaults stay the committed rules, with their
//! citations and tests, and the files are a tuning layer over them (`docs/tuning.md`).
//!
//! - **Export** (`--export-data <dir>`, [`export()`]): the built-in rules and tuning written as files that mirror their
//!   modules ([`layout`]), pretty-printed in their fields' order ([`ordered`]).
//! - **Load** ([`load()`]): each `.json` file of a directory is a merge patch over its built-in counterpart
//!   ([`patch`]); an unknown file or member, a value of the wrong kind, a number out of its sign, a change of
//!   structure (`Ruleset::structural_diff`) or a number the game isn't made for (`Ruleset::check`) is refused, with
//!   the file and the member.
//! - **Live** (`--tuning <dir>`, [`live`]): the directory read before the first tick, its rules given to the game as a
//!   `DebugCommand::SetRules` in the first tick's inputs, its client tuning as the `ClientTuning` resource; then
//!   polled every half second ([`poll`]), each change read again and sent; F8 restarts the game with it. The HUD's top
//!   bar says "Tuned: N files" while files change something, and the feedback line says what was refused.
//!
//! The sim never reads a file: the client reads, the rules travel in the command, and a replay of a tuned game plays
//! it back with no directory. The goldens never read one.

pub mod export;
pub mod layout;
pub mod live;
pub mod load;
pub mod ordered;
pub mod patch;
pub mod poll;

#[cfg(test)]
mod tests;

pub use export::export;
pub use live::{DataPlugin, TuningData};
pub use load::{Data, Refusal, load};
