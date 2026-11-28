//! The citation rule (`ARCHITECTURE.md > The design is the spec`): a relative ``> Section`` only in a module whose doc
//! names exactly one file, and no bare design file name (``economy.md > Beacon``) anywhere.
//!
//! - **What is read**: a Rust file's comments (line and block comments; its strings, raw strings and character
//!   literals are code), and a markdown file but its fenced code blocks. A double-backtick code span shows an example
//!   (``> A > B``) and is not read.
//! - **A relative citation** is a code span that begins with `>` and a space or a line break. It is allowed only in a
//!   Rust file whose module doc (its leading `//!` lines) cites exactly one `.md` path ([`super::citations`]), a
//!   design file or a `docs/` one: its sections are that file's. A markdown file has no module doc, so it allows none.
//! - **A bare name** is a code span that begins with a design file's name rather than its path. The design's top-level
//!   files are paths of their own (`README.md`), and the repository's root files (`ARCHITECTURE.md`) aren't the
//!   design's.

use std::collections::BTreeSet;
use std::fs;
use std::ops::Range;
use std::path::Path;

use super::{DESIGN, Prefixes, citations, root, scanned_files};

/// The comments of a Rust text: line comments, to the line's end, and block comments, nested. Strings, raw strings and
/// character literals are skipped, so that what they hold never starts a comment.
fn comments(text: &[u8]) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        let in_word = i > 0 && (text[i - 1].is_ascii_alphanumeric() || text[i - 1] == b'_');
        if rest.starts_with(b"//") {
            let end = rest.iter().position(|&c| c == b'\n').map_or(text.len(), |n| i + n);
            found.push(i..end);
            i = end;
        } else if rest.starts_with(b"/*") {
            let (mut depth, mut j) = (1, i + 2);
            while j < text.len() && depth > 0 {
                if text[j..].starts_with(b"/*") {
                    (depth, j) = (depth + 1, j + 2);
                } else if text[j..].starts_with(b"*/") {
                    (depth, j) = (depth - 1, j + 2);
                } else {
                    j += 1;
                }
            }
            found.push(i..j);
            i = j;
        } else if let Some(end) = raw_string_end(text, i).filter(|_| !in_word) {
            i = end;
        } else if rest[0] == b'"' {
            i += 1;
            while i < text.len() && text[i] != b'"' {
                i += if text[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
        } else if rest[0] == b'\'' {
            i += char_literal_len(rest).unwrap_or(1);
        } else {
            i += 1;
        }
    }
    found
}

/// Where the raw string that begins at `i` ends (`r"…"`, `r#"…"#`, `br"…"`), if one does.
fn raw_string_end(text: &[u8], i: usize) -> Option<usize> {
    let r = i + usize::from(text[i] == b'b');
    if text.get(r) != Some(&b'r') {
        return None;
    }
    let hashes = text[r + 1..].iter().take_while(|&&c| c == b'#').count();
    let open = r + 1 + hashes;
    if text.get(open) != Some(&b'"') {
        return None;
    }
    let close: Vec<u8> = std::iter::once(b'"').chain(std::iter::repeat_n(b'#', hashes)).collect();
    let at = text[open + 1..].windows(close.len()).position(|w| w == close)?;
    Some(open + 1 + at + close.len())
}

/// The length of the character literal that begins `text` (`'a'`, `'\n'`, `'\u{2014}'`, `'—'`), unless the quote
/// begins a lifetime or a label (`'a`, `'outer:`).
fn char_literal_len(text: &[u8]) -> Option<usize> {
    let first = *text.get(1)?;
    if first == b'\\' {
        let close = text.get(3..)?.iter().take(10).position(|&c| c == b'\'')?;
        return Some(3 + close + 1);
    }
    let width = match first {
        b'\'' => return None,
        0..0x80 => 1,
        0xF0.. => 4,
        0xE0.. => 3,
        _ => 2,
    };
    (text.get(1 + width) == Some(&b'\'')).then_some(2 + width)
}

/// What the rule reads of `text`, at the same offsets: a Rust file's comments, or a markdown file but its fenced code
/// blocks, everything else blanked but the line breaks; and in it each double-backtick code span blanked too.
fn prose(text: &str, rust: bool) -> String {
    let bytes = text.as_bytes();
    let read = if rust {
        comments(bytes)
    } else {
        let (mut read, mut start, mut fenced) = (Vec::new(), 0, false);
        for line in bytes.split_inclusive(|&c| c == b'\n') {
            let fence = line.trim_ascii_start().starts_with(b"```") || line.trim_ascii_start().starts_with(b"~~~");
            fenced ^= fence;
            if !fenced && !fence {
                read.push(start..start + line.len());
            }
            start += line.len();
        }
        read
    };
    let mut out: Vec<u8> = bytes.iter().map(|&c| if c == b'\n' { c } else { b' ' }).collect();
    for part in read {
        out[part.clone()].copy_from_slice(&bytes[part]);
    }
    let ticks = |out: &[u8], from: usize| out.get(from..)?.windows(2).position(|w| w == b"``").map(|n| from + n);
    let mut from = 0;
    while let Some(open) = ticks(&out, from) {
        let Some(close) = ticks(&out, open + 2) else {
            break;
        };
        for c in &mut out[open..close + 2] {
            if *c != b'\n' {
                *c = b' ';
            }
        }
        from = close + 2;
    }
    String::from_utf8(out).expect("whole comments and lines, the rest blanked: still UTF-8")
}

/// The `.md` files the module doc of a Rust file's `prose` (its leading `//!` lines) cites, but the architecture docs
/// (`docs/architecture/`), which are rules for the code and not a file whose sections a relative citation names.
fn module_files(prose: &str, prefixes: &Prefixes) -> BTreeSet<String> {
    let end = prose
        .split_inclusive('\n')
        .take_while(|line| line.starts_with("//!"))
        .map(str::len)
        .sum();
    let design = format!("{DESIGN}/");
    citations(&prose[..end], prefixes)
        .into_iter()
        .filter(|cited| cited.path.ends_with(".md") && !cited.path.starts_with("docs/architecture/"))
        .map(|cited| cited.path.strip_prefix(&design).unwrap_or(&cited.path).to_owned())
        .collect()
}

/// Each citation of `text` the rule forbids: a relative one where none is allowed, and a code span that begins with
/// one of the `bare` names. As its line (from 1) and the code span's text, up to 60 bytes of its first line.
fn misplaced_in(text: &str, rust: bool, prefixes: &Prefixes, bare: &BTreeSet<String>) -> Vec<(usize, String)> {
    let prose = prose(text, rust);
    let bytes = prose.as_bytes();
    let relative_allowed = rust && module_files(&prose, prefixes).len() == 1;
    let mut found = Vec::new();
    for (i, _) in bytes.iter().enumerate().filter(|(_, c)| **c == b'`') {
        if (i > 0 && bytes[i - 1] == b'`') || bytes.get(i + 1) == Some(&b'`') {
            continue;
        }
        let span = &bytes[i + 1..];
        let relative = span.starts_with(b">") && matches!(span.get(1), None | Some(b' ' | b'\n'));
        let name_len = span
            .iter()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.'))
            .count();
        let named = &prose[i + 1..i + 1 + name_len];
        if (relative && !relative_allowed) || bare.contains(named) {
            let end = span
                .iter()
                .take(60)
                .position(|&c| c == b'`' || c == b'\n')
                .unwrap_or(span.len().min(60));
            let line = 1 + bytes[..i].iter().filter(|&&c| c == b'\n').count();
            found.push((line, String::from_utf8_lossy(&span[..end]).into_owned()));
        }
    }
    found
}

/// The names a bare citation would give: each design file's, but the design's top-level files (`README.md`, a path of
/// its own) and the repository's root ones (`ARCHITECTURE.md`).
fn bare_names(root: &Path, prefixes: &Prefixes) -> BTreeSet<String> {
    fn md_names(dir: &Path, deep: bool, names: &mut BTreeSet<String>) {
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        for entry in entries.map(|entry| entry.expect("a folder entry")) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if deep {
                    md_names(&entry.path(), deep, names);
                }
            } else if name.ends_with(".md") {
                names.insert(name);
            }
        }
    }
    let (mut names, mut root_files) = (BTreeSet::new(), BTreeSet::new());
    md_names(&root.join(DESIGN), true, &mut names);
    md_names(root, false, &mut root_files);
    names.retain(|name| prefixes.base(name.as_bytes()).is_none() && !root_files.contains(name));
    names
}

/// Each citation in the scanned files that the rule forbids, as `file:line: span`.
fn misplaced(root: &Path) -> Vec<String> {
    let prefixes = Prefixes::of(root);
    let bare = bare_names(root, &prefixes);
    let mut found = Vec::new();
    for file in scanned_files(root) {
        let text = fs::read_to_string(root.join(&file)).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        let rust = file.extension().is_some_and(|extension| extension == "rs");
        for (line, span) in misplaced_in(&text, rust, &prefixes, &bare) {
            found.push(format!("{}:{line}: `{span}`", file.display()));
        }
    }
    found
}

#[test]
fn every_citation_names_its_file() {
    let found = misplaced(&root());
    assert!(
        found.is_empty(),
        "{} citations don't name their file: write the path in full, `path/file.md > Section` (a relative one only in \
         a module whose doc names exactly one file):\n{}",
        found.len(),
        found.join("\n")
    );
}

/// The rule on texts of its own: what it forbids is named with its line; strings, examples and the sections of a
/// module of one file pass. And the bare names are the design's, less its top-level files.
#[test]
fn a_planted_misplaced_citation_is_named_with_its_line() {
    let prefixes = Prefixes::of(&root());
    let bare = BTreeSet::from(["economy.md".to_owned()]);
    let found = |text, rust| -> Vec<String> {
        let spans = misplaced_in(text, rust, &prefixes, &bare);
        spans
            .into_iter()
            .map(|(line, span)| format!("{line}: {span}"))
            .collect()
    };
    let one_file = r##"//! The shield (`factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`).

/// `> During the toggle`, and across a line (`>
/// Two unit types`).
fn f() -> (&'static str, char) {
    ("`economy.md` in a string", '"') // `> A section of that file`
}
"##;
    assert_eq!(found(one_file, true), Vec::<String>::new());
    let two_files = r##"//! Vision (`core/vision.md`) and the map (`core/map.md > Elevation`).

/// `> Explored`: of which file? Nor this one (`>
/// Visible`). ``> Shown`` is an example; `README.md`, `ARCHITECTURE.md > Tunables` and `core/map.md` are paths.
// A bare name: `economy.md > Beacon`.
const URL: &str = "http://host/`> not a comment`";
const QUOTE: char = '"'; /* `> In a block comment` */
const RAW: &str = r#"a quote " and // `> not a comment either`"#;
fn f<'a>(s: &'a str) -> &'a str { s } // `economy.md`
"##;
    let expected = [
        "3: > Explored",
        "3: >",
        "5: economy.md > Beacon",
        "7: > In a block comment",
        "9: economy.md",
    ];
    assert_eq!(found(two_files, true), expected);
    let doc = r##"A relative `> Section`, and an example: ``> Section``.
```text
`> In a fence`
```
`economy.md`; `core/map.md > Elevation` and `README.md` are paths.
"##;
    assert_eq!(found(doc, false), ["1: > Section", "5: economy.md"]);
    let names = bare_names(&root(), &prefixes);
    assert!(names.len() > 20 && !names.contains("README.md"), "{names:?}");
}
