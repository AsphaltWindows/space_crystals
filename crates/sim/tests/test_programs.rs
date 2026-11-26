//! Every test file is in a test program (slice M5-T). Each crate's integration tests are modules of the few programs
//! its `Cargo.toml` lists as `[[test]]` targets, with `autotests = false`, so a file that no program includes would
//! never be compiled, let alone run. This fails for the sim, the computer players (slice A1) and the client, naming
//! each such file and the line that includes it (for a folder, its root alone), and each file that two programs
//! include, whose tests would run twice.
//!
//! It follows the module declarations from each program's root as the compiler does: in a root, a `mod.rs` or a file
//! included by a `#[path]`, `mod foo;` is `foo.rs` (or `foo/mod.rs`) beside it; in any other file, `bar.rs`, it is in
//! `bar/`; and `#[path = "p"] mod foo;` is `p`, beside the file that declares it. It reads the declarations that
//! begin a line, and fails on one inside an inline module, which it doesn't follow.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn every_sim_test_file_is_in_a_program() {
    let root = root();
    let problems = problems(&root.join("crates/sim"), &root);
    assert!(problems.is_empty(), "the sim's test files:\n{}", problems.join("\n"));
}

#[test]
fn every_client_test_file_is_in_a_program() {
    let root = root();
    let problems = problems(&root.join("crates/game"), &root);
    assert!(problems.is_empty(), "the client's test files:\n{}", problems.join("\n"));
}

#[test]
fn every_computer_player_test_file_is_in_a_program() {
    let root = root();
    let problems = problems(&root.join("crates/ai"), &root);
    assert!(
        problems.is_empty(),
        "the computer players' test files:\n{}",
        problems.join("\n")
    );
}

/// The check on a crate made up for it, in a folder of its own: two programs, one by its `path` and one by its
/// `name`, a folder included by a `#[path]` with a module in it and one in that; and a file left out at each level, a
/// folder left out (named by its root alone, not by the modules in it), and a file that both programs include.
#[test]
fn it_names_each_file_in_no_program_or_in_two() {
    let dir = std::env::temp_dir().join(format!("sc_test_programs_{}", std::process::id()));
    let manifest = "[package]\nname = \"x\"\nautotests = false\n\n[[test]]\nname = \"one\"\npath = \"tests/one.rs\"\n\n\
                    [dev-dependencies]\n\n[[test]]\nname = \"two\"\n";
    let files = [
        ("Cargo.toml", manifest),
        (
            "tests/one.rs",
            "//! mod not_a_module;\n\nmod a;\n#[path = \"b/main.rs\"]\nmod b;\nmod both;\n",
        ),
        ("tests/two.rs", "mod both;\n"),
        ("tests/a.rs", ""),
        ("tests/b/main.rs", "mod c;\n"),
        ("tests/b/c.rs", "pub(crate) mod d;\n"),
        ("tests/b/c/d.rs", ""),
        ("tests/both.rs", ""),
        ("tests/lost.rs", ""),
        ("tests/lost_folder/main.rs", "mod inside;\n"),
        ("tests/lost_folder/inside.rs", "mod deeper;\n"),
        ("tests/lost_folder/inside/deeper.rs", ""),
        ("tests/b/c/lost.rs", ""),
    ];
    for (file, text) in files {
        let path = dir.join(file);
        fs::create_dir_all(path.parent().expect("a folder")).expect("a folder for the made-up crate");
        fs::write(&path, text).expect("a file of the made-up crate");
    }
    let found = problems(&dir, &fs::canonicalize(&dir).expect("the made-up crate"));
    fs::remove_dir_all(&dir).expect("the made-up crate removed");
    let roots = "tests/one.rs or tests/two.rs";
    assert_eq!(
        found,
        [
            "tests/both.rs: included by two test programs, tests/one.rs and tests/two.rs, so its tests would run twice"
                .to_owned(),
            "tests/b/c/lost.rs: in no test program, so its tests never run; declare it, `mod lost;`, in the module of \
             its folder, `c/`"
                .to_owned(),
            format!("tests/lost.rs: in no test program, so its tests never run; add `mod lost;` to {roots}"),
            format!(
                "tests/lost_folder/main.rs: in no test program, so its tests never run; add `#[path = \
                 \"lost_folder/main.rs\"] mod lost_folder;` to {roots}"
            ),
        ]
    );
}

/// The repository's root, canonical, so that the files found by declaration and by listing compare equal.
fn root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::canonicalize(&root).unwrap_or_else(|e| panic!("{}: {e}", root.display()))
}

/// `file`'s path from `root`, to show.
fn shown(file: &Path, root: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).display().to_string()
}

/// What keeps a test of the crate in `dir` from running exactly once: each `.rs` file in its `tests/` that none of its
/// test programs includes, or that two include, and anything wrong with how it lists them. Paths are shown from
/// `root`.
fn problems(dir: &Path, root: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    let roots = programs(dir, root, &mut problems);
    // Each file a program includes, and the root of that program.
    let mut included: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    for program in &roots {
        for file in modules(program, root, &mut problems) {
            if let Some(other) = included.insert(file.clone(), program.clone()) {
                problems.push(format!(
                    "{}: included by two test programs, {} and {}, so its tests would run twice",
                    shown(&file, root),
                    shown(&other, root),
                    shown(program, root)
                ));
            }
        }
    }
    let roots: Vec<String> = roots.iter().map(|program| shown(program, root)).collect();
    let tests = fs::canonicalize(dir.join("tests")).unwrap_or_else(|e| panic!("{}/tests: {e}", dir.display()));
    for file in rs_files(&tests) {
        if !included.contains_key(&file) && !behind_left_out(&tests, &file, &included) {
            let line = inclusion(&tests, &file, &roots.join(" or "));
            problems.push(format!(
                "{}: in no test program, so its tests never run; {line}",
                shown(&file, root)
            ));
        }
    }
    problems
}

/// The roots of the crate's test programs: the `path` of each of its `[[test]]` targets, else Cargo's default for its
/// `name`. It needs `autotests = false`, without which Cargo would build every file as a program of its own as well.
fn programs(dir: &Path, root: &Path, problems: &mut Vec<String>) -> Vec<PathBuf> {
    let manifest = dir.join("Cargo.toml");
    let text = fs::read_to_string(&manifest).unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
    if !text.lines().any(|line| line.trim() == "autotests = false") {
        problems.push(format!(
            "{}: no `autotests = false`, so Cargo would build every test file as a program of its own as well, and \
             its tests would run twice",
            shown(&manifest, root)
        ));
    }
    // Each `[[test]]` target's name and path, as far as they are given.
    let mut targets: Vec<(Option<String>, Option<String>)> = Vec::new();
    let mut in_target = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_target = line == "[[test]]";
            if in_target {
                targets.push((None, None));
            }
        } else if in_target && let Some(target) = targets.last_mut() {
            if let Some(name) = string_value(line, "name") {
                target.0 = Some(name.to_owned());
            } else if let Some(path) = string_value(line, "path") {
                target.1 = Some(path.to_owned());
            }
        }
    }
    if targets.is_empty() {
        problems.push(format!("{}: no `[[test]]` target", shown(&manifest, root)));
    }
    let mut roots = Vec::new();
    for target in targets {
        let path = match target {
            (_, Some(path)) => dir.join(path),
            (Some(name), None) if dir.join(format!("tests/{name}.rs")).exists() => dir.join(format!("tests/{name}.rs")),
            (Some(name), None) => dir.join(format!("tests/{name}/main.rs")),
            (None, None) => {
                problems.push(format!(
                    "{}: a `[[test]]` target without a name",
                    shown(&manifest, root)
                ));
                continue;
            }
        };
        match fs::canonicalize(&path) {
            Ok(path) => roots.push(path),
            Err(e) => problems.push(format!("{}: {e}", shown(&path, root))),
        }
    }
    roots
}

/// The value of `key = "value"`, if `line` is that.
fn string_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let value = line.strip_prefix(key)?.trim_start().strip_prefix('=')?.trim_start();
    value.strip_prefix('"')?.split('"').next()
}

/// The files of a program: its root, then the modules it includes, and theirs, by their declarations.
fn modules(program: &Path, root: &Path, problems: &mut Vec<String>) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    // Each file still to read, and whether the modules it declares without a `#[path]` sit beside it (a root, a
    // `mod.rs` or a file included by a `#[path]`) rather than in a folder named after it.
    let mut pending = vec![(program.to_path_buf(), true)];
    while let Some((file, beside)) = pending.pop() {
        let (file, text) = match fs::canonicalize(&file).and_then(|file| Ok((fs::read_to_string(&file)?, file))) {
            Ok((text, file)) => (file, text),
            Err(e) => {
                problems.push(format!("{}: {e}", shown(&file, root)));
                continue;
            }
        };
        if files.contains(&file) {
            problems.push(format!("{}: included twice by one test program", shown(&file, root)));
            continue;
        }
        let folder = file.parent().expect("a file in a folder").to_path_buf();
        let children = match beside {
            true => folder.clone(),
            false => folder.join(file.file_stem().expect("a file name")),
        };
        let mut path = None;
        for (n, line) in text.lines().enumerate() {
            if let Some(value) = path_attribute(line) {
                path = Some(value.to_owned());
                continue;
            }
            let Some(name) = declaration(line) else {
                continue;
            };
            if line.starts_with(char::is_whitespace) {
                problems.push(format!(
                    "{}:{}: `mod {name};` inside an inline module, which this check doesn't follow",
                    shown(&file, root),
                    n + 1
                ));
                continue;
            }
            let child = match path.take() {
                Some(path) => (folder.join(path), true),
                None if children.join(name).join("mod.rs").exists() => (children.join(name).join("mod.rs"), true),
                None => (children.join(format!("{name}.rs")), false),
            };
            pending.push(child);
        }
        files.push(file);
    }
    files
}

/// The path of a `#[path = "…"]` attribute, if `line` is one.
fn path_attribute(line: &str) -> Option<&str> {
    let attribute = line.trim().strip_prefix("#[")?.strip_suffix(']')?;
    string_value(attribute.trim(), "path")
}

/// The name of the module that `line` declares out of line (`mod foo;`, any visibility before it), if it does.
fn declaration(line: &str) -> Option<&str> {
    let mut rest = line.trim_start();
    if let Some(public) = rest.strip_prefix("pub") {
        rest = public.trim_start();
        if rest.starts_with('(') {
            rest = rest.split_once(')')?.1.trim_start();
        }
    }
    let rest = rest.strip_prefix("mod")?;
    let name = rest.trim_start().split(';').next()?.trim_end();
    let declared = rest.starts_with(char::is_whitespace)
        && rest.contains(';')
        && !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    declared.then_some(name)
}

/// Every `.rs` file in `dir` and its folders, canonical, in name order.
fn rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut folders = vec![dir.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let entries = fs::read_dir(&folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display()));
        for entry in entries.map(|entry| entry.expect("a folder entry")) {
            let path = entry.path();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                folders.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(fs::canonicalize(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())));
            }
        }
    }
    files.sort();
    files
}

/// Whether `file`, a file of `tests` that no program includes, is left out only because the file that would declare it
/// is too: a module of a folder whose root (`main.rs`, `mod.rs`, or the file named after the folder) exists and no
/// program includes. That file is the one to name, and its modules are checked once it is in.
fn behind_left_out(tests: &Path, file: &Path, included: &BTreeMap<PathBuf, PathBuf>) -> bool {
    let Some(mut folder) = file.parent() else {
        return false;
    };
    // A folder's own root is declared by the module of the folder around it.
    if file
        .file_name()
        .is_some_and(|name| name == "main.rs" || name == "mod.rs")
    {
        let Some(outer) = folder.parent() else {
            return false;
        };
        folder = outer;
    }
    if folder == tests || !folder.starts_with(tests) {
        return false;
    }
    let declaring = [
        folder.join("main.rs"),
        folder.join("mod.rs"),
        folder.with_extension("rs"),
    ];
    declaring
        .iter()
        .any(|root| root != file && root.exists() && !included.contains_key(root))
}

/// The declaration that would include `file`, a file of `tests` (a crate's `tests/`) that no program includes: in a
/// program's root (`roots`) for a test file, `tests/foo.rs`, or a folder's root, `tests/foo/main.rs`; in the module of
/// its folder for any other.
fn inclusion(tests: &Path, file: &Path, roots: &str) -> String {
    let relative = file.strip_prefix(tests).unwrap_or(file);
    let parts: Vec<String> = relative
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect();
    let stem = |name: &str| name.strip_suffix(".rs").unwrap_or(name).to_owned();
    match parts.as_slice() {
        [name] => format!("add `mod {};` to {roots}", stem(name)),
        [folder, main] if main == "main.rs" => format!("add `#[path = \"{folder}/main.rs\"] mod {folder};` to {roots}"),
        [.., folder, name] => format!(
            "declare it, `mod {};`, in the module of its folder, `{folder}/`",
            stem(name)
        ),
        [] => "it is no file".to_owned(),
    }
}
