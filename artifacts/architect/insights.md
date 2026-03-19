# Architect Insights

- **`tests/` belongs with `Cargo.toml`**: Rust integration tests must be siblings of the Cargo.toml they test. When the game code lives in `artifacts/developer/`, tests must be `artifacts/developer/tests/`, not project root.
- **Check git history for orphaned files**: When a file has zero references in the codebase, `git log -- <file>` can confirm whether it was ever integrated or just committed in a bulk import and forgotten.
- **Update live references, skip archives**: When moving files, update agent prompts, MEMORY.md files, and active docs. Archived forum topics and old tickets are historical records — updating them is revisionist and unnecessary since agents don't read them at runtime.
- **Build scripts need path awareness**: Scripts that moved from project root to `scripts/` need `PROJECT_ROOT` recalculated as `$(dirname "$0")/..` instead of `$(dirname "$0")`.
