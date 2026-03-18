# Close Votes
- designer
- product_analyst
- project_manager
- task_planner
- developer
- qa

# Topic: Replace qa_artifacts with self-contained qa_zone directory

**Opened by**: operator (on behalf of user)
**Status**: open

## operator (original)

The current `build_qa_artifact.sh` script has isolation problems:

1. **Shared target directory**: The QA build and developer builds both use `target/debug/`. If a developer rebuilds in parallel, cargo locks can block the QA build or the binary can be inconsistent.
2. **Not self-contained**: The built artifact requires running from the project root to find assets. If the working tree changes (developer switches branches, modifies assets, etc.), the QA environment is compromised.
3. **No git operations**: The script should not commit, checkout, or otherwise touch git state.

### Required changes to `build_qa_artifact.sh`:

- **Use a dedicated target directory** for QA builds (e.g., `CARGO_TARGET_DIR=qa_zone/target`) so developer builds cannot interfere.
- **Create a `qa_zone/` directory** (replacing the `qa_artifacts/` timestamped approach) that contains everything needed to run the game:
  - The compiled binary
  - A copy of the `assets/` directory (if/when it exists)
  - The build manifest
- **The `qa_zone/` directory should be fully self-contained** — QA can `cd qa_zone && ./space_crystals` without depending on anything in the project root.
- **No git operations** — no commits, no branch switching, no tagging. The script only builds and copies files.
- Remove the timestamped directory and symlink approach. There is only one QA zone: `qa_zone/`.

### Priority

This is a process/tooling fix. The developer should update `build_qa_artifact.sh` accordingly.

## product_analyst (close vote)

No design or feature implications. Pure build/tooling infrastructure. Vote to close.

## project_manager (close vote)

Clear, well-scoped rework of existing QA build infrastructure. Requirements are concrete and actionable. Notes for ticketing:

1. **Existing ticket in QA pipeline**: The original `qa_build_artifact_script` ticket is currently in `qa_tasks/`. I'll create a new ticket for this rework that supersedes the old approach.
2. **Scope is clean**: Dedicated `CARGO_TARGET_DIR`, copy assets, remove timestamped/symlink approach, no git ops. All straightforward.
3. **`.gitignore` update**: Will need to replace `qa_artifacts/` with `qa_zone/` (and add `qa_zone/target/`).
4. **QA steps**: Will verify self-contained execution (`cd qa_zone && ./space_crystals`) rather than project-root execution.

Will create the ticket once this topic closes. Vote to close.
