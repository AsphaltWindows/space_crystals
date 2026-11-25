//! Design citations (`ARCHITECTURE.md > The design is the spec`; slice M4-R2, design change 0282, which split each
//! faction's units and structures into a file per object): every design path the code and the docs cite exists, so a
//! design file that is renamed, split or removed fails here, naming each file and line that still cite it; and every
//! citation names its file, as the rule asks ([`rule`]: a relative ``> Section`` only in a module whose doc names
//! exactly one file, no bare file name).
//!
//! - **Scanned**: `crates/**/*.rs`, `docs/**/*.md` and `ARCHITECTURE.md`, from the repository's root.
//! - **A citation** is a path that begins with an entry of the design folder's top level (`core/`, `systems/`,
//!   `factions/`, `archive/`, `README.md`: whatever `artifacts/designer/design/` holds), with that folder's own path,
//!   or with `docs/`, where nothing that can be part of a path comes before it (so `crates/sim/src/core/` is none). It
//!   runs as far as a path's characters go (letters, digits, `_`, `-`, `.`, `/`), less a period that ends a sentence,
//!   and on across a line break in a comment or a doc, when it breaks after a `/`.
//! - **It must exist**: a design path in `artifacts/designer/design/`, the others from the repository's root (a
//!   `docs/` path in the repository's `docs/`); a file for a path that ends in `.md`, a folder for one that ends in
//!   `/`. Any other path (a module's `systems/panel.rs`) cites nothing.
//! - **The sections** after a path (`core/map.md > Elevation > Map resources only on flat terrain`) are not checked:
//!   the design names many of them by bold paragraphs, not headings. The ignored
//!   `report_cited_sections_that_are_not_headings` lists the ones that aren't headings of the file they cite:
//!   `cargo test -p sc_sim --test design_citations -- --ignored --nocapture`.
//!
//! It runs in the sim crate, in the fast `cargo test -p sc_sim` loop, without Bevy.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

mod rule;

/// The design folder, from the repository's root.
const DESIGN: &str = "artifacts/designer/design";

/// The repository's root, two levels above this crate.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// What a citation begins with, and the folder its path is relative to: the design's top-level entries, in the design
/// folder; the design folder's own path and `docs/`, in the repository's root.
struct Prefixes(Vec<(String, PathBuf)>);

impl Prefixes {
    fn of(root: &Path) -> Self {
        let design = root.join(DESIGN);
        let entries = fs::read_dir(&design).unwrap_or_else(|e| panic!("{}: {e}", design.display()));
        let mut prefixes: Vec<(String, PathBuf)> = entries
            .map(|entry| entry.expect("a design folder entry"))
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
                (if is_dir { format!("{name}/") } else { name }, design.clone())
            })
            .collect();
        prefixes.push((format!("{DESIGN}/"), root.to_path_buf()));
        prefixes.push(("docs/".to_owned(), root.to_path_buf()));
        prefixes.sort();
        Self(prefixes)
    }

    /// The folder of the path that begins `text`, if it is a citation.
    fn base(&self, text: &[u8]) -> Option<&Path> {
        let (_, base) = self.0.iter().find(|(prefix, _)| text.starts_with(prefix.as_bytes()))?;
        Some(base)
    }
}

/// A path cited in a text: the line it begins on (from 1), the path, the folder it is relative to, and where it
/// begins and ends in the text.
#[derive(Clone, Debug, PartialEq)]
struct Cited<'a> {
    line: usize,
    path: String,
    base: &'a Path,
    start: usize,
    end: usize,
}

impl Cited<'_> {
    /// Whether what it names exists: a file for a `.md` path, a folder for one that ends in `/`. Any other path cites
    /// nothing, and passes.
    fn exists(&self) -> bool {
        let at = self.base.join(&self.path);
        if self.path.ends_with(".md") {
            at.is_file()
        } else if self.path.ends_with('/') {
            at.is_dir()
        } else {
            true
        }
    }
}

fn is_path_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.' | b'/')
}

/// Where the text goes on after a line break at `at`: past the next line's indentation, its comment marker (`///`,
/// `//!`, `//`) and the indentation after it. `None` unless `at` is a line break.
fn continuation(text: &[u8], at: usize) -> Option<usize> {
    if text.get(at) != Some(&b'\n') {
        return None;
    }
    let blank = |text: &[u8], mut i: usize| {
        while text.get(i).is_some_and(|c| *c == b' ' || *c == b'\t') {
            i += 1;
        }
        i
    };
    let mut i = blank(text, at + 1);
    if let Some(marker) = ["///", "//!", "//"]
        .iter()
        .find(|m| text[i..].starts_with(m.as_bytes()))
    {
        i = blank(text, i + marker.len());
    }
    Some(i)
}

/// Every path `text` cites ([`Prefixes`]), in order.
fn citations<'p>(text: &str, prefixes: &'p Prefixes) -> Vec<Cited<'p>> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let (mut line, mut i) = (1, 0);
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            line += 1;
        }
        let begins = i == 0 || !is_path_char(bytes[i - 1]);
        let Some(base) = begins.then(|| prefixes.base(&bytes[i..])).flatten() else {
            i += 1;
            continue;
        };
        let (first, mut path, mut j) = (line, String::new(), i);
        loop {
            let from = j;
            while j < bytes.len() && is_path_char(bytes[j]) {
                j += 1;
            }
            path.push_str(&text[from..j]);
            // A path broken after a `/` at a line's end goes on at the next line's start.
            let next = continuation(bytes, j).filter(|&n| bytes.get(n).is_some_and(|c| is_path_char(*c)));
            match next.filter(|_| path.ends_with('/')) {
                Some(next) => {
                    line += 1;
                    j = next;
                }
                None => break,
            }
        }
        while path.ends_with('.') {
            path.pop();
        }
        found.push(Cited {
            line: first,
            path,
            base,
            start: i,
            end: j,
        });
        i = j;
    }
    found
}

/// The sections a citation names after its path (``> A > B``, across line breaks in a comment), up to the backtick that
/// closes the code span the path opens; none for a path that opens no code span, or whose span doesn't soon close.
fn sections(text: &str, cited: &Cited) -> Vec<String> {
    let bytes = text.as_bytes();
    if cited.start == 0 || bytes[cited.start - 1] != b'`' {
        return Vec::new();
    }
    let mut span = Vec::new();
    let mut i = cited.end;
    while i < bytes.len() && bytes[i] != b'`' {
        match continuation(bytes, i) {
            Some(next) => {
                span.push(b' ');
                i = next;
            }
            None => {
                span.push(bytes[i]);
                i += 1;
            }
        }
        if span.len() > 400 {
            return Vec::new();
        }
    }
    let span = String::from_utf8_lossy(&span);
    let Some(rest) = span.trim_start().strip_prefix('>') else {
        return Vec::new();
    };
    rest.split(" > ")
        .map(|section| section.trim().to_owned())
        .filter(|section| !section.is_empty())
        .collect()
}

/// The files scanned, from the repository's root, in order: `crates/**/*.rs`, `docs/**/*.md`, `ARCHITECTURE.md`.
fn scanned_files(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, dir: &Path, extension: &str, files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(root.join(dir)) else {
            return;
        };
        for entry in entries.map(|entry| entry.expect("a folder entry")) {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = dir.join(&name);
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if !name.starts_with('.') && name != "target" {
                    walk(root, &path, extension, files);
                }
            } else if name.ends_with(extension) {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(root, Path::new("crates"), ".rs", &mut files);
    walk(root, Path::new("docs"), ".md", &mut files);
    files.push(PathBuf::from("ARCHITECTURE.md"));
    files.sort();
    files
}

/// Each citation in the scanned files whose path doesn't exist, as `file:line: path`, and how many paths are cited.
fn missing(root: &Path) -> (Vec<String>, usize) {
    let prefixes = Prefixes::of(root);
    let (mut missing, mut count) = (Vec::new(), 0);
    for file in scanned_files(root) {
        let text = fs::read_to_string(root.join(&file)).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        for cited in citations(&text, &prefixes) {
            count += 1;
            if !cited.exists() {
                missing.push(format!("{}:{}: {}", file.display(), cited.line, cited.path));
            }
        }
    }
    (missing, count)
}

#[test]
fn every_cited_design_path_exists() {
    let (missing, count) = missing(&root());
    assert!(
        count > 1500,
        "only {count} cited paths found: is the scan looking in the right place?"
    );
    assert!(
        missing.is_empty(),
        "{} cited paths don't exist (design paths are relative to `{DESIGN}/`, `docs/` ones to the repository):\n{}",
        missing.len(),
        missing.join("\n")
    );
}

/// The check on a text of its own: a planted bad path is named with its line, and what isn't a citation passes.
#[test]
fn a_planted_bad_path_is_named_with_its_line() {
    let prefixes = Prefixes::of(&root());
    // Put together here, so that this file itself cites no bad path.
    let (factions, docs) = ("factions", "docs");
    let removed = format!("{factions}/gdo/units.md");
    let (nowhere, none) = (format!("{factions}/colonists/nowhere.md"), format!("{docs}/none.md"));
    let text = format!(
        "//! Good (`{factions}/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`, `core/map.md`).\n\
         /// Bad: `{removed} > Peacekeeper`.\n\
         /// Broken (`{factions}/colonists/\n\
         /// economy.md > Beacon`) and `{factions}/colonists/\n\
         /// nowhere.md`; in a sentence, see {factions}/gdo/overview.md. And `docs/movement.md > 3`, `{none}`.\n\
         Not citations: crates/sim/src/core/ and command_ui/systems/panel.rs. A folder: `{factions}/syndicate/`.\n"
    );
    let cited = citations(&text, &prefixes);
    let paths: Vec<(usize, &str)> = cited.iter().map(|c| (c.line, c.path.as_str())).collect();
    assert_eq!(
        paths,
        [
            (1, "factions/gdo/units/peacekeeper.md"),
            (1, "core/map.md"),
            (2, removed.as_str()),
            (3, "factions/colonists/economy.md"),
            (4, nowhere.as_str()),
            (5, "factions/gdo/overview.md"),
            (5, "docs/movement.md"),
            (5, none.as_str()),
            (6, "factions/syndicate/"),
        ]
    );
    let bad: Vec<(usize, &str)> = cited
        .iter()
        .filter(|c| !c.exists())
        .map(|c| (c.line, c.path.as_str()))
        .collect();
    assert_eq!(bad, [(2, removed.as_str()), (4, nowhere.as_str()), (5, none.as_str())]);
    let sections: Vec<Vec<String>> = cited.iter().map(|c| sections(&text, c)).collect();
    assert_eq!(sections[0], ["Riot Shield (toggled ability)"]);
    assert_eq!(sections[3], ["Beacon"]);
    assert!(sections[5].is_empty(), "not in a code span");
}

/// The headings of a markdown file, without their `#`s, bold markers and a trailing colon.
fn headings(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.starts_with('#'))
        .map(|line| {
            let heading = line.trim_start_matches('#').trim().replace("**", "");
            heading.trim_end_matches(':').to_owned()
        })
        .collect()
}

/// Whether `heading` is the section `cited`: the same, or the same followed by its value or a qualifier
/// (`MaxHP - 50`, `Riot Shield (toggled ability)`, `ObjectInterfaceState: BasicCombatUnitInterfaceState`,
/// `ObjectInterfaceState[Barracks]`, a numbered `3. Replanning`).
fn names(heading: &str, cited: &str) -> bool {
    heading == cited
        || heading.strip_prefix(cited).is_some_and(|rest| {
            [" - ", " (", ":", "[", ". ", " —"]
                .iter()
                .any(|separator| rest.starts_with(separator))
        })
}

/// A report, not a gate: each cited section that isn't a heading of the file it cites, marked "nowhere" when its
/// name isn't in the file at all (renamed, or gone), else named by a bold paragraph or in a line of the text.
#[test]
#[ignore = "a report: cargo test -p sc_sim --test design_citations -- --ignored --nocapture"]
fn report_cited_sections_that_are_not_headings() {
    let root = root();
    let prefixes = Prefixes::of(&root);
    let mut files: BTreeMap<PathBuf, Option<String>> = BTreeMap::new();
    let (mut checked, mut not_headings, mut nowhere) = (0, 0, 0);
    for file in scanned_files(&root) {
        let text = fs::read_to_string(root.join(&file)).expect("a scanned file");
        for cited in citations(&text, &prefixes) {
            if !cited.path.ends_with(".md") {
                continue;
            }
            let at = cited.base.join(&cited.path);
            let Some(cited_text) = files.entry(at.clone()).or_insert_with(|| fs::read_to_string(&at).ok()) else {
                continue;
            };
            let known = headings(cited_text);
            for section in sections(&text, &cited) {
                checked += 1;
                if known.iter().any(|heading| names(heading, &section)) {
                    continue;
                }
                not_headings += 1;
                let found = cited_text.contains(&section);
                nowhere += usize::from(!found);
                let note = if found { "" } else { "  (nowhere)" };
                println!("{}:{}: {} > {section}{note}", file.display(), cited.line, cited.path);
            }
        }
    }
    println!("{checked} cited sections, {not_headings} not headings of their file, {nowhere} of them nowhere in it");
}
