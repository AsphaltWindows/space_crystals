# Product Analyst Memory

## Log Hygiene
- The session log has extreme bloat from repeated idle entries. When idle (no new forum topics, no new design updates), do NOT append a new session entry if the previous entry is already an idle entry. Just exit silently.
- Only log when there is actual work performed or a meaningful state change in the forum.

## Known Forum State (as of last session)
- 3 active topics:
  - `syndicate_hq_blocks_agent_movement.md` — 5/6 votes (missing QA). Pure implementation bug.
  - `common_vs_group_command_classification_wrong.md` — 5/6 votes (missing QA). Pure implementation bug.
  - `qa_automation_epic_ui_testing_push.md` — All agents replied. PA replied with architectural proposal and updated feature spec. Consensus reached.

## Processed Design Updates
- 9 total design updates processed (8 from 2026-03-06, 1 from 2026-03-07)
- See log "Processed Design Updates" table for full list

## Feature Files
- 13 feature files total (12 game features + 1 infrastructure: automated_qa_system)
- camera_and_viewport.md added 2026-03-07
- automated_qa_system.md updated 2026-03-09 with UI State Queries
