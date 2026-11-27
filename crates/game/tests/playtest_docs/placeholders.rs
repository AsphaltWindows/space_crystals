//! The placeholders list, `docs/playtest/placeholders.md` (slice M5-Ma): every `Placeholder (` mark in the comments
//! of `crates/` (`ARCHITECTURE.md > Tunables`: a number or a rule the design doesn't give is marked where it is made,
//! with the question that asks for it, if any), so that a playtest knows what is a stand-in.
//!
//! - **An entry** ([`Mark`]) is the mark's sentence, and the one before it when the mark opens its sentence, as the
//!   comment says it, with its file, the item it documents (a constant's or a field's name and value, `WAVE_SIZE =
//!   32`) and the questions it names (`Q0147`). A relative citation (`> Types`) is given its module's file.
//! - **Grouped** by faction where the path names one (a `cults/` folder, or a folder or file named after a type of
//!   the faction, `chop_shop/`), and the rest by crate and system ([`System`]).
//! - **Not kept current**: the list is rewritten only by `UPDATE_DOCS=1` ([`super::rewrite_if_updating`]); its test
//!   checks that the scan finds the marks.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sc_sim::{Faction, ObjectKind};

use super::{rewrite_if_updating, root};

/// The doc, from the repository's root.
const DOC: &str = "docs/playtest/placeholders.md";
/// Its generated part's name ([`super::markers`]).
const PART: &str = "placeholders";
/// The mark.
const MARK: &str = "Placeholder (";
/// This generator's folder, whose examples of marks aren't marks.
const OWN: &str = "crates/game/tests/playtest_docs";
/// The design folder, whose top-level entries begin a citation, as `crates/sim/tests/design_citations/` reads them.
const DESIGN: &str = "artifacts/designer/design";

/// The list's test: the scan finds marks in each crate, each with its text and its group; and the doc is rewritten
/// only while updating, however stale it is (slice M5-X1 moves the constants; slice M5-Mb regenerates it).
#[test]
fn the_placeholders_list() {
    let root = root();
    let marks = scan(&root);
    for krate in ["crates/sim/", "crates/game/", "crates/ai/"] {
        assert!(
            marks.iter().any(|mark| mark.file.starts_with(krate)),
            "no mark in {krate}"
        );
    }
    for mark in &marks {
        assert!(mark.text.contains(MARK), "{}:{}: {}", mark.file, mark.line, mark.text);
    }
    rewrite_if_updating(DOC, PART, &render(&marks));
}

/// A mark's entry: its sentence, and the one before it when the mark opens its sentence; the item it documents; its
/// questions; and no mark quoted as an example.
#[test]
fn an_entry_is_its_sentence_with_its_item() {
    let text = "//! The doc names `systems/controls/hud.md > Types`.\n\
                \n\
                /// How far a unit looks. It looks this far ahead, in its own lengths.\n\
                /// **Placeholder (not in the design)**: slice M1-G1's.\n\
                pub const LANE_LOOK: f32 = 4.0;\n\
                /// A mark mid-sentence, Placeholder (Q0099; the Hall's\n\
                /// area), and on (`> Types`): [`Foo`] here. Next sentence.\n\
                pub fn foo() {}\n\
                /// Every number is marked \"Placeholder (Q0147 point N)\".\n\
                pub mod tuning {\n    \
                    /// Placeholder (Q0147 point 2): the first wave.\n    \
                    pub const FIRST_WAVE: usize = 8;\n\
                }\n";
    let marks = marks_in("crates/x/src/lane.rs", text, &prefixes(&root()));
    let said: Vec<(Option<&str>, &str, &[String])> = (marks.iter())
        .map(|mark| (mark.item.as_deref(), mark.text.as_str(), &mark.questions[..]))
        .collect();
    let q = |questions: &[&str]| questions.iter().map(|q| (*q).to_owned()).collect::<Vec<_>>();
    assert_eq!(said.len(), 3, "{marks:?}");
    assert_eq!(
        said[0],
        (
            Some("LANE_LOOK = 4.0"),
            "It looks this far ahead, in its own lengths. Placeholder (not in the design): slice M1-G1's.",
            &q(&[])[..]
        )
    );
    assert_eq!(
        said[1],
        (
            None,
            "A mark mid-sentence, Placeholder (Q0099; the Hall's area), and on (`systems/controls/hud.md > Types`): \
             `Foo` here.",
            &q(&["Q0099"])[..]
        )
    );
    assert_eq!(
        said[2],
        (
            Some("tuning::FIRST_WAVE = 8"),
            "Placeholder (Q0147 point 2): the first wave.",
            &q(&["Q0147"])[..]
        )
    );
}

/// A mark: where it is, what it documents, and its sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Mark {
    /// Its file, from the repository's root.
    file: String,
    /// Its line, from 1.
    line: usize,
    /// The item its doc comment documents, if a constant's or a field's: its name, in its inline modules, with its
    /// value when short.
    item: Option<String>,
    /// Its sentence (module docs).
    text: String,
    /// The questions it names, in order.
    questions: Vec<String>,
}

/// Every mark in `crates/`, file by file in path order.
fn scan(root: &Path) -> Vec<Mark> {
    let mut files = Vec::new();
    rust_files(&root.join("crates"), &mut files);
    files.sort();
    let prefixes = prefixes(root);
    let mut marks = Vec::new();
    for file in files {
        let relative = file
            .strip_prefix(root)
            .expect("under the root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&file).unwrap_or_else(|e| panic!("{relative}: {e}"));
        marks.extend(marks_in(&relative, &text, &prefixes));
    }
    marks
}

/// The `.rs` files under `dir`, but in build output, hidden folders and this generator's ([`OWN`]).
fn rust_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
    for entry in entries.map(|entry| entry.expect("a folder entry")) {
        let (path, name) = (entry.path(), entry.file_name().to_string_lossy().into_owned());
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            if name != "target" && !name.starts_with('.') && !path.ends_with(OWN) {
                rust_files(&path, files);
            }
        } else if name.ends_with(".rs") {
            files.push(path);
        }
    }
}

/// What a design citation begins with: the design folder's top-level entries, its own path, and `docs/`
/// (`crates/sim/tests/design_citations/main.rs`'s rule).
fn prefixes(root: &Path) -> Vec<String> {
    let design = root.join(DESIGN);
    let entries = fs::read_dir(&design).unwrap_or_else(|e| panic!("{}: {e}", design.display()));
    let mut prefixes: Vec<String> = entries
        .map(|entry| entry.expect("a design folder entry"))
        .map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect();
    prefixes.extend([format!("{DESIGN}/"), "docs/".to_owned()]);
    prefixes
}

/// The kind of a comment line: `//!`, `///` or `//`; `None` for a line that isn't one.
fn comment(line: &str) -> Option<&'static str> {
    let line = line.trim_start();
    if line.starts_with("//!") {
        Some("//!")
    } else if line.starts_with("///") && !line.starts_with("////") {
        Some("///")
    } else if line.starts_with("//") {
        Some("//")
    } else {
        None
    }
}

/// The marks of one file, `file`, whose text is `text`.
fn marks_in(file: &str, text: &str, prefixes: &[String]) -> Vec<Mark> {
    let lines: Vec<&str> = text.lines().collect();
    let header: String = (lines.iter().take_while(|line| line.starts_with("//!")))
        .map(|line| line.trim_start_matches("//!"))
        .collect::<Vec<_>>()
        .join("\n");
    let cited = module_file(&header, prefixes);
    let mut modules: Vec<(usize, String)> = Vec::new();
    let mut marks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let indent = line.len() - line.trim_start().len();
        if let Some(name) = inline_module(line) {
            modules.push((indent, name.to_owned()));
        } else if line.trim() == "}" && modules.last().is_some_and(|(at, _)| *at == indent) {
            modules.pop();
        }
        let Some(kind) = comment(line) else {
            i += 1;
            continue;
        };
        let start = i;
        while i < lines.len() && comment(lines[i]) == Some(kind) {
            i += 1;
        }
        let block = Block::of(&lines[start..i], kind);
        let item = (kind != "//!")
            .then(|| lines[i..].iter().find(|line| !line.trim_start().starts_with("#[")))
            .flatten()
            .and_then(|line| item(line))
            .map(|item| {
                let path: Vec<&str> = modules.iter().map(|(_, name)| name.as_str()).collect();
                [&path[..], &[item.as_str()]].concat().join("::")
            });
        for at in block.marks() {
            let text = block.sentence(at);
            let questions = questions(&text);
            marks.push(Mark {
                file: file.to_owned(),
                line: start + 1 + block.line_of(at),
                item: item.clone(),
                text: cleaned(&text, cited.as_deref()),
                questions,
            });
        }
    }
    marks
}

/// The name of an inline module that a line opens, `pub mod army {`.
fn inline_module(line: &str) -> Option<&str> {
    let line = line.trim();
    let line = line.strip_prefix("pub ").unwrap_or(line);
    line.strip_prefix("mod ")?.strip_suffix(" {")
}

/// The item a doc comment documents, from its line: a constant's or a static's name, with its value when the line
/// gives it and it is short (`LANE_LOOK = 4.0`); a field's, with its value or type (`search_radius: 6.0`). `None` for
/// any other item.
fn item(line: &str) -> Option<String> {
    let mut line = line.trim();
    if let Some(rest) = line.strip_prefix("pub") {
        line = match rest.strip_prefix('(') {
            Some(inside) => inside.split_once(')')?.1,
            None => rest,
        }
        .trim_start();
    }
    let short = |value: &str| value.len() <= 40;
    if let Some(rest) = line.strip_prefix("const ").or_else(|| line.strip_prefix("static ")) {
        let (name, rest) = rest.split_once(':')?;
        let value = rest.split_once(" = ").and_then(|(_, value)| value.strip_suffix(';'));
        return Some(match value.filter(|value| short(value)) {
            Some(value) => format!("{name} = {value}"),
            None => name.to_owned(),
        });
    }
    let (name, value) = line.split_once(": ")?;
    let field = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    let value = value.strip_suffix(',').filter(|_| field)?;
    Some(if short(value) {
        format!("{name}: {value}")
    } else {
        name.to_owned()
    })
}

/// A comment block's text, its lines joined: by a space, but none after a `/`, where a citation breaks
/// (`crates/sim/tests/design_citations/main.rs`), with where each line starts and where each paragraph and each list
/// item begins, which ends a sentence.
struct Block {
    text: String,
    starts: Vec<usize>,
    breaks: Vec<usize>,
}

impl Block {
    fn of(lines: &[&str], kind: &str) -> Self {
        let (mut text, mut starts, mut breaks, mut blank) = (String::new(), Vec::new(), Vec::new(), false);
        for line in lines {
            let body = line.trim_start().trim_start_matches(kind).trim();
            let item = body.starts_with("- ")
                || body.starts_with("* ")
                || body
                    .split_once(". ")
                    .is_some_and(|(n, _)| !n.is_empty() && n.bytes().all(|c| c.is_ascii_digit()));
            if !text.is_empty() && !text.ends_with('/') && !body.is_empty() {
                text.push(' ');
            }
            if !body.is_empty() && !text.is_empty() && (blank || item) {
                breaks.push(text.len());
            }
            blank = body.is_empty();
            starts.push(text.len());
            text.push_str(body);
        }
        Self { text, starts, breaks }
    }

    /// The line (from 0) of the block that `at` is on.
    fn line_of(&self, at: usize) -> usize {
        self.starts.iter().rposition(|&start| start <= at).unwrap_or(0)
    }

    /// Where the marks begin: not one quoted as an example, after a `"` or a backtick.
    fn marks(&self) -> Vec<usize> {
        let text = &self.text;
        (text.match_indices(MARK).map(|(at, _)| at))
            .filter(|&at| !text[..at].ends_with(['"', '`']))
            .collect()
    }

    /// Where the sentences end: after a `.`, `!` or `?` outside parentheses and code, followed by a space and a
    /// capital, a parenthesis, a quote, a backtick or bold, or by the end; and where a paragraph or a list item begins.
    fn ends(&self) -> Vec<usize> {
        let bytes = self.text.as_bytes();
        let (mut depth, mut code, mut ends) = (0usize, false, Vec::new());
        for (i, &c) in bytes.iter().enumerate() {
            if self.breaks.contains(&i) {
                (depth, code) = (0, false);
                ends.push(i);
            }
            match c {
                b'`' => code = !code,
                b'(' | b'[' if !code => depth += 1,
                b')' | b']' if !code => depth = depth.saturating_sub(1),
                b'.' | b'!' | b'?' if !code && depth == 0 => {
                    let ends_here = match bytes.get(i + 1..i + 3) {
                        Some([b' ', after]) => after.is_ascii_uppercase() || b"(\"`*".contains(after),
                        _ => i + 1 == bytes.len(),
                    };
                    if ends_here {
                        ends.push(i + 1);
                    }
                }
                _ => {}
            }
        }
        ends
    }

    /// The sentence of the mark at `at`, and the one before it in its paragraph when the mark opens its sentence (the
    /// bold that marks it aside), which says what it is about; without its list item's dash or a trailing `;`.
    fn sentence(&self, at: usize) -> String {
        let ends = self.ends();
        let begin_of = |at: usize| ends.iter().rev().find(|&&end| end <= at).copied().unwrap_or(0);
        let begin = begin_of(at);
        let end = ends.iter().find(|&&end| end > at).copied().unwrap_or(self.text.len());
        let opens = self.text[begin..at].trim().trim_matches('*').is_empty();
        let paragraph = begin == 0 || self.breaks.contains(&begin);
        let begin = if opens && !paragraph {
            begin_of(begin - 1)
        } else {
            begin
        };
        let sentence = self.text[begin..end].trim();
        let sentence = sentence
            .strip_prefix("- ")
            .or_else(|| sentence.strip_prefix("* "))
            .unwrap_or(sentence);
        sentence.trim_end_matches([';', ',', ':']).to_owned()
    }
}

/// The questions a text names, `Q0147`, each once, in order.
fn questions(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut found: Vec<String> = Vec::new();
    for (at, _) in text.match_indices('Q') {
        let digits = bytes
            .get(at + 1..at + 5)
            .is_some_and(|d| d.iter().all(u8::is_ascii_digit));
        let alone = (at == 0 || !bytes[at - 1].is_ascii_alphanumeric())
            && bytes.get(at + 5).is_none_or(|c| !c.is_ascii_alphanumeric());
        let question = text[at..].chars().take(5).collect::<String>();
        if digits && alone && !found.contains(&question) {
            found.push(question);
        }
    }
    found
}

/// The one design file a module's doc cites, which its relative citations (`> Types`) are in, as the citation rule
/// reads it (`crates/sim/tests/design_citations/rule.rs`); `None` unless there is exactly one.
fn module_file(header: &str, prefixes: &[String]) -> Option<String> {
    let path_char = |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.' | b'/');
    let bytes = header.as_bytes();
    let mut files: Vec<String> = Vec::new();
    for at in 0..bytes.len() {
        let free = header.is_char_boundary(at) && (at == 0 || !path_char(bytes[at - 1]));
        if !free || !prefixes.iter().any(|prefix| header[at..].starts_with(prefix.as_str())) {
            continue;
        }
        let len = bytes[at..].iter().take_while(|&&c| path_char(c)).count();
        let path = header[at..at + len].trim_end_matches('.');
        let path = path.strip_prefix(&format!("{DESIGN}/")).unwrap_or(path);
        if path.ends_with(".md") && !files.iter().any(|file| file == path) {
            files.push(path.to_owned());
        }
    }
    match &files[..] {
        [file] => Some(file.clone()),
        _ => None,
    }
}

/// A mark's text for the doc: without bold, its links as code (`[`Foo`]` as `Foo`), a relative citation in its module's
/// file `cited` (in quotes without one), and nothing a Markdown viewer would read as a tag.
fn cleaned(text: &str, cited: Option<&str>) -> String {
    let text = text.replace("**", "");
    let mut out = String::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find(['`', '[', '<']) {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        if let Some(inside) = rest.strip_prefix("[`")
            && let Some(close) = inside.find("`]")
        {
            out.push_str(&format!("`{}`", &inside[..close]));
            rest = &inside[close + 2..];
            if let Some(link) = rest.strip_prefix('(')
                && let Some(close) = link.find(')')
            {
                rest = &link[close + 1..];
            }
        } else if let Some(inside) = rest.strip_prefix('`')
            && let Some(close) = inside.find('`')
        {
            let span = &inside[..close];
            match (span.strip_prefix("> "), cited) {
                (Some(section), Some(file)) => out.push_str(&format!("`{file} > {section}`")),
                (Some(section), None) => out.push_str(&format!("\"{section}\"")),
                (None, _) => out.push_str(&format!("`{span}`")),
            }
            rest = &inside[close + 1..];
        } else if rest.starts_with('<') {
            out.push_str("&lt;");
            rest = &rest[1..];
        } else {
            out.push_str(&rest[..1]);
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

/// Where a mark is listed: its faction's section, or its crate's and its system's.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum System {
    Faction(Faction),
    /// A crate's area (by the doc's order: the rules, the client, the computer players) and a system's name.
    Area(u8, String),
}

/// A faction's name, as the HUD says it.
fn faction_name(faction: Faction) -> &'static str {
    match faction {
        Faction::Gdo => "GDO",
        Faction::Syndicate => "Syndicate",
        Faction::Cults => "Cults",
        Faction::Colonists => "Colonists",
    }
}

/// The faction a path names: a folder named after it (`defs/cults/`), or a folder or file named after one of its types
/// (`tests/chop_shop/`, `tests/shrine.rs`).
fn faction_of(file: &str) -> Option<Faction> {
    let parts: Vec<&str> = file.trim_end_matches(".rs").split('/').collect();
    let named = |part: &str| {
        let faction = Faction::ALL.iter().find(|&&f| faction_name(f).to_lowercase() == part);
        faction.copied().or_else(|| {
            let kind = ObjectKind::ALL.iter().find(|kind| snake(kind.def().name) == part);
            kind.and_then(|kind| kind.def().faction)
        })
    };
    parts.iter().skip(2).find_map(|part| named(part))
}

/// A type's name as a module's: "Chop Shop" as `chop_shop`.
fn snake(name: &str) -> String {
    let words: Vec<String> = (name.split(|c: char| !c.is_ascii_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    words.join("_")
}

/// Where a mark is listed. A rule's or the client's: its faction's section where its path names one ([`faction_of`]),
/// else its crate's area and the most specific part of its path that names a system ([`system_name`]), else the first
/// after `src/` or `tests/` (the folders `tuning/` and `sim/` only group a crate's files). A computer player's: their
/// area, by the module of its number (`army`, `gdo`) or the faction its path names.
fn system_of(mark: &Mark) -> System {
    let area = match mark.file.split('/').nth(1) {
        Some("sim") => 0,
        Some("game") => 1,
        _ => 2,
    };
    let faction = faction_of(&mark.file);
    if area < 2
        && let Some(faction) = faction
    {
        return System::Faction(faction);
    }
    let parts: Vec<&str> = mark.file.trim_end_matches(".rs").split('/').skip(3).collect();
    let parts: Vec<&str> = parts
        .into_iter()
        .filter(|part| !matches!(*part, "tuning" | "sim" | "tests"))
        .collect();
    let module = mark
        .item
        .as_deref()
        .and_then(|item| item.split_once("::"))
        .map(|(module, _)| module);
    let name = match (module.filter(|_| area == 2), faction.filter(|_| area == 2)) {
        (Some(module), _) => title(module),
        (None, Some(faction)) => faction_name(faction).to_owned(),
        (None, None) => (parts.iter().rev().find_map(|part| system_name(part)).map(str::to_owned))
            .unwrap_or_else(|| parts.first().map_or_else(|| "Other".to_owned(), |part| title(part))),
    };
    System::Area(area, name)
}

/// A module's name as a heading: a faction's as the HUD says it, any other's in words.
fn title(module: &str) -> String {
    if let Some(&faction) = Faction::ALL.iter().find(|&&f| faction_name(f).to_lowercase() == module) {
        return faction_name(faction).to_owned();
    }
    let words = module.replace('_', " ");
    let mut chars = words.chars();
    chars
        .next()
        .map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
}

/// The systems that a part of a path names.
fn system_name(part: &str) -> Option<&'static str> {
    Some(match part {
        "movement" | "mover" | "paths" => "Movement",
        "air" | "flight" | "glider" => "Aircraft",
        "combat" | "combat_effects" => "Combat",
        "repair" | "repairs" => "Repair",
        "holding" => "Buildings that hold units",
        "production" | "producers" => "Production",
        "construction" | "spot_work" | "placement" => "Construction and placement",
        "casts" | "buffs" | "energy" | "shroud" => "Spells and energy",
        "augmentation" => "Augmentation",
        "orders" => "Orders",
        "defs" => "Definitions",
        "input" => "Input",
        "camera" => "The camera",
        "feedback" => "Feedback",
        "hud" => "The HUD",
        "command_ui" => "Commands and panels",
        "object_render" | "door_arrows" => "Drawing",
        "editor" => "The map editor",
        "viewer_events" => "What a player sees",
        _ => return None,
    })
}

/// The list: each faction's marks, then each crate's by system, every entry with its file.
fn render(marks: &[Mark]) -> String {
    let mut groups: BTreeMap<System, Vec<&Mark>> = BTreeMap::new();
    for mark in marks {
        groups.entry(system_of(mark)).or_default().push(mark);
    }
    let areas = [
        "The rules (`crates/sim`)",
        "The client (`crates/game`)",
        "The computer players (`crates/ai`)",
    ];
    let mut out = format!("{} marks.\n", marks.len());
    let mut area = None;
    for (system, marks) in &groups {
        let heading = match system {
            System::Faction(faction) => format!("## {} ({})", faction_name(*faction), marks.len()),
            System::Area(index, name) => {
                if area != Some(*index) {
                    area = Some(*index);
                    out.push_str(&format!("\n## {}\n", areas[usize::from(*index)]));
                }
                format!("### {name} ({})", marks.len())
            }
        };
        out.push_str(&format!("\n{heading}\n\n"));
        for mark in marks {
            out.push_str(&entry(mark));
        }
    }
    out
}

/// A mark's line in the list: its questions, its item (by its name alone: its heading names its module), its text and
/// its file.
fn entry(mark: &Mark) -> String {
    let questions = match &mark.questions[..] {
        [] => String::new(),
        questions => format!("**{}** ", questions.join(", ")),
    };
    let name = |item: &str| item.rsplit("::").next().unwrap_or(item).to_owned();
    let item = mark
        .item
        .as_ref()
        .map_or_else(String::new, |item| format!("`{}`: ", name(item)));
    format!("- {questions}{item}{} (`{}`)\n", mark.text, mark.file)
}
