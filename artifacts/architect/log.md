# Architect Session Log

## 2026-03-19 — Project root cleanup

### Actions taken

1. **Moved `tests/` to `artifacts/developer/tests/`** — integration tests now live alongside Cargo.toml as part of Developer's artifact space. Updated path references in:
   - `artifacts/developer/src/shared/testing/README.md`
   - `.claude/agent-memory/developer/MEMORY.md`
   - `.claude/agent-memory/qa/MEMORY.md`
   - `features/automated_qa_system.md`

2. **Deleted legacy scripts from project root**:
   - `run_supervisor.sh` — replaced by `scripts/run_scheduler.sh`
   - `check_pipeline.sh` — replaced by scheduler logic
   - `tag_qa_steps.py` — orphaned, zero references anywhere in project

3. **Moved QA scripts to `scripts/`**:
   - `build_qa_artifact.sh` → `scripts/build_qa_artifact.sh` (updated paths to build from `artifacts/developer/`, output to `artifacts/manual_qa/qa_artifacts/`)
   - `run_qa.sh` → `scripts/run_qa.sh` (updated to run from `artifacts/developer/` for Bevy assets)

4. **Deleted legacy root directories and files**:
   - `features/` — design docs (superseded by `artifacts/designer/design/`)
   - `tickets_archive/` — old ticket format
   - `design_updates_archive/` — old message format
   - `feature_updates_archive/` — old message format
   - `analyst/` — old product analyst agent data
   - `completed_tasks/` — old task tracking
   - `developer_tasks/` — old task flow (forum topic created for remaining task)
   - `qa_tasks/` — old QA flow (forum topics created for remaining items)
   - `qa_human_review/` — old QA flow (forum topics created for remaining items)
   - `claude.md` — old project instructions (superseded by `CLAUDE.md`)
   - `claude.log` — empty file
   - `PROGRESS.md` — stale progress tracker from Feb 2026

### Result
Project root now contains only current pipeline framework files:
`agents/`, `artifacts/`, `CLAUDE.md`, `forum/`, `forum_archive/`, `framework.md`, `messages/`, `pipeline.yaml`, `scripts/`, `templates/`
