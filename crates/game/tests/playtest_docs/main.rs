//! The playtest kit's generated docs (slice M5-Ma; `docs/playtest/README.md`): the controls of every faction's panels
//! and the game's keys, which the panel templates and the bindings table give (`controls`, `docs/playtest/controls.md`),
//! and the placeholders the code marks (`placeholders`, `docs/playtest/placeholders.md`). Each doc has its generated
//! part between two markers that name it ([`markers`]); the text around them is the doc's own, written by hand.
//!
//! - **Regenerating**: `UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::` rewrites the
//!   generated parts of both docs, `playtest_docs::controls` or `playtest_docs::placeholders` the one.
//! - **The controls doc can't drift**: its test fails when the doc isn't what the templates and the bindings give,
//!   naming the line that differs and the command that regenerates it ([`keep_current`]).
//! - **The placeholders list may**: slice M5-X1 moves the constants and their docs, so the list is regenerated at the
//!   milestone's end (slice M5-Mb), and its test only checks that the scan works ([`rewrite_if_updating`]).
//!
//! The tests read the variable, as the goldens read `SC_BLESS`, and set nothing: the tests share one process.

mod controls;
mod placeholders;

use std::fs;
use std::path::{Path, PathBuf};

/// The repository's root, two levels above this crate.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Whether this run rewrites the docs' generated parts: `UPDATE_DOCS=1`.
fn updating() -> bool {
    std::env::var("UPDATE_DOCS").is_ok_and(|value| value == "1")
}

/// The lines that open and close a doc's generated part `name`: HTML comments, which a Markdown viewer doesn't show.
fn markers(name: &str) -> (String, String) {
    (
        format!("<!-- generated: {name}, begin; regenerate it, don't edit it -->"),
        format!("<!-- generated: {name}, end -->"),
    )
}

/// `doc` with its generated part `name` replaced by `generated`, which ends with a line break: a blank line after the
/// opening marker, `generated`, and a blank line before the closing one. `Err` if the doc lacks a marker.
fn spliced(doc: &str, name: &str, generated: &str) -> Result<String, String> {
    let (begin, end) = markers(name);
    let missing = |marker: &str| format!("no marker line `{marker}`");
    let start = doc.find(&begin).ok_or_else(|| missing(&begin))? + begin.len();
    let stop = start + doc[start..].find(&end).ok_or_else(|| missing(&end))?;
    Ok(format!("{}\n\n{generated}\n{}", &doc[..start], &doc[stop..]))
}

/// The first line where `new` differs from `old`, by its number in `new` (from 1), with both versions of it.
fn first_difference(old: &str, new: &str) -> String {
    let (mut old_lines, mut new_lines) = (old.lines(), new.lines());
    for number in 1.. {
        match (old_lines.next(), new_lines.next()) {
            (Some(was), Some(is)) if was == is => continue,
            (None, None) => break,
            (was, is) => {
                let shown =
                    |line: Option<&str>| line.map_or_else(|| "(nothing)".to_owned(), |line| format!("`{line}`"));
                return format!(
                    "line {number}: the doc has {}, the code gives {}",
                    shown(was),
                    shown(is)
                );
            }
        }
    }
    "line breaks only".to_owned()
}

/// Why the doc at `path` (from the repository's root), whose text is `doc`, needs its generated part `name` rewritten
/// as `generated`, naming the command that does it, `regenerate`; `None` when it is current.
fn stale(path: &str, doc: &str, name: &str, generated: &str, regenerate: &str) -> Option<String> {
    let new = match spliced(doc, name, generated) {
        Ok(new) if new == doc => return None,
        Ok(new) => new,
        Err(problem) => return Some(format!("{path}: {problem}")),
    };
    Some(format!(
        "{path} is out of date with the code ({}).\nRegenerate it, and commit it with your change:\n\n    \
         {regenerate}\n",
        first_difference(doc, &new)
    ))
}

/// Keeps the generated part `name` of the doc at `path` (from the repository's root) current: rewrites it as
/// `generated` while updating ([`updating`]), and otherwise fails if it differs, naming the line and `regenerate`.
fn keep_current(path: &str, name: &str, generated: &str, regenerate: &str) {
    let file = root().join(path);
    let doc = fs::read_to_string(&file).unwrap_or_else(|e| panic!("{path}: {e}"));
    let Some(problem) = stale(path, &doc, name, generated, regenerate) else {
        return;
    };
    let new = spliced(&doc, name, generated).unwrap_or_else(|_| panic!("{problem}"));
    if updating() {
        fs::write(&file, new).unwrap_or_else(|e| panic!("{path}: {e}"));
    } else {
        panic!("{problem}");
    }
}

/// Rewrites the generated part `name` of the doc at `path` as `generated` while updating ([`updating`]); else
/// nothing, however stale it is.
fn rewrite_if_updating(path: &str, name: &str, generated: &str) {
    if !updating() {
        return;
    }
    let file = root().join(path);
    let doc = fs::read_to_string(&file).unwrap_or_else(|e| panic!("{path}: {e}"));
    let new = spliced(&doc, name, generated).unwrap_or_else(|problem| panic!("{path}: {problem}"));
    if new != doc {
        fs::write(&file, new).unwrap_or_else(|e| panic!("{path}: {e}"));
    }
}

/// The markers' splice: the text around the generated part stays, the part is replaced, and a doc without its
/// markers is refused; a stale part names the first line that differs and the command.
#[test]
fn a_generated_part_is_spliced_between_its_markers() {
    let (begin, end) = markers("table");
    let doc = format!("# Title\n\nBy hand.\n\n{begin}\n\nold\n\n{end}\n\nAfter.\n");
    let new = spliced(&doc, "table", "new\n").unwrap();
    assert_eq!(
        new,
        format!("# Title\n\nBy hand.\n\n{begin}\n\nnew\n\n{end}\n\nAfter.\n")
    );
    assert_eq!(
        spliced(&new, "table", "new\n").unwrap(),
        new,
        "spliced again, unchanged"
    );
    assert!(spliced("# Title\n", "table", "new\n").is_err());
    assert_eq!(stale("doc.md", &new, "table", "new\n", "regenerate"), None);
    let problem = stale("doc.md", &doc, "table", "new\n", "UPDATE_DOCS=1 cargo test").unwrap();
    assert!(
        problem.contains("line 7: the doc has `old`, the code gives `new`"),
        "{problem}"
    );
    assert!(problem.contains("    UPDATE_DOCS=1 cargo test\n"), "{problem}");
}
