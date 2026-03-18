# Ticket: QA Build Artifact Script

## Current State
QA sessions use `cargo run` to test the game. Because the developer agent runs concurrently, source code can change mid-session, causing `cargo run` to produce a different binary than the one initially tested. QA pass/fail results become unreliable.

## Desired State
A `build_qa_artifact.sh` script exists at the project root. When invoked, it:

1. Runs `cargo build --no-default-features` (produces a statically-linked binary without `bevy_dylib`, ensuring the artifact is self-contained and portable)
2. Creates a timestamped directory under `qa_artifacts/` (e.g., `qa_artifacts/2026-03-06_143022/`)
3. Copies the compiled binary (`target/debug/space_crystals`) into that directory
4. Updates a `qa_artifacts/latest` symlink to point to the new timestamped directory
5. Writes a `qa_artifacts/<timestamp>/manifest.txt` containing:
   - Git commit hash
   - Git dirty state (clean/dirty)
   - Git branch name
   - QA task filename(s) being tested (passed as script arguments)
   - Build timestamp
6. Exits with a clear error (non-zero exit code, descriptive message) if `cargo build` fails, without updating the `latest` symlink or leaving a broken artifact

Additionally:
- `qa_artifacts/` is added to `.gitignore`
- The script outputs a run instruction: run from the project root as `./qa_artifacts/latest/space_crystals` (Bevy resolves assets relative to CWD, so the binary must be launched from the project root where the `assets/` directory lives)
- The script checks whether `bevy/dynamic_linking` is enabled and warns if the build might require shared libraries alongside the binary

## Justification
Forum topic `forum/qa_build_artifact.md` (opened by operator on behalf of user). With 37+ tasks in `/qa_tasks`, QA sessions will be frequent. Concurrent developer agent execution means the binary under test can change mid-session. Frozen build artifacts are standard practice for QA reliability and prevent wasted cycles from false passes or unnecessary re-tests.

Key technical decisions from forum discussion:
- **Debug profile** (not release) — QA tests functionality, not performance; debug builds are faster and preserve debug info
- **Static build via `--no-default-features`** — project uses `bevy/dynamic_linking` by default; building without it produces a self-contained binary (developer recommendation, Option A)
- **Run from project root** — Bevy asset paths resolve relative to CWD (product_analyst, task_planner, developer all confirmed)

## QA Steps
1. Run `./build_qa_artifact.sh some_task_name` from the project root
2. Verify the script runs `cargo build` and completes without error
3. Check that a timestamped directory exists under `qa_artifacts/` containing the `space_crystals` binary
4. Check that `qa_artifacts/latest` is a symlink pointing to the timestamped directory
5. Read `qa_artifacts/latest/manifest.txt` and verify it contains: commit hash, dirty state, branch, task name (`some_task_name`), and a timestamp
6. Run `./qa_artifacts/latest/space_crystals` from the project root — verify the game launches and assets load correctly (window opens, no asset-loading errors in console)
7. Run `./build_qa_artifact.sh` again — verify a second timestamped directory is created and `latest` symlink is updated
8. Simulate a build failure (e.g., introduce a syntax error in `main.rs`), run the script, verify it exits with a non-zero code and an error message, and that the `latest` symlink still points to the previous successful build
9. Undo the syntax error
10. Verify `qa_artifacts/` is listed in `.gitignore`

## Expected Experience
- Step 1: Terminal shows cargo compilation output followed by a success message indicating where the artifact was saved
- Step 3: `ls qa_artifacts/` shows a timestamped directory (e.g., `2026-03-06_143022/`) containing a `space_crystals` binary
- Step 4: `ls -la qa_artifacts/latest` shows a symlink arrow pointing to the timestamped directory
- Step 5: `cat qa_artifacts/latest/manifest.txt` shows structured metadata including "some_task_name" as the task
- Step 6: The game window opens normally — no "asset not found" errors or panics in the terminal
- Step 7: `ls qa_artifacts/` now shows two timestamped directories; `latest` points to the newer one
- Step 8: Terminal shows a build error message from the script (not raw cargo output); `latest` symlink is unchanged from step 7
- Step 10: `grep qa_artifacts .gitignore` returns a match
