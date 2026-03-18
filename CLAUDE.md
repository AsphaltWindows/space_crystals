# Space Crystals RTS

A real-time strategy game built with Bevy 0.17.

# Project Overview

Space Crystals is an RTS game currently in early development. The project uses the Bevy game engine and is written in Rust.

# Agents

Agent definitions live in `./.claude/agents/`. All agents follow the shared framework defined in `agents/framework.md`.

Agents are stateless workers. Each execution: load context, do one task, write outputs, die. The only continuity between executions is through files (logs, forum, task artifacts).

## Agent Framework

See `agents/framework.md` for the full shared lifecycle, forum rules, and directory layout.

Key concepts:
- **Forum** (`/forum`): Highest-priority task source for all agents. Cross-agent communication via topic files.
- **Logs**: Each agent has a persistent log file that is always loaded. The agent's only memory between executions.
- **Single task per execution**: An agent does at most one unit of work per run.

## Available Agent Types

### Designer
**File**: `agents/designer.md` | **Log**: `./agent_logs/designer_log.md`

Interactive agent. Collaboratively develops game design with the user. Maintains `/design/*.md` files and writes `/design_updates` summaries after each session.

### Product Analyst
**File**: `agents/product_analyst.md` | **Log**: `./agent_logs/product_analyst_log.md`

Processes `/design_updates` into `/features` specifications. Writes `/feature_updates` for each feature modified. Skeptical — flags inconsistencies via the forum.

### Project Manager
**File**: `agents/project_manager.md` | **Log**: `./agent_logs/project_manager_log.md`

Processes `/feature_updates` into `/tickets` with QA steps and expected user experience. Also creates technical/refactor tickets when forum discussions warrant them.

### Task Planner
**File**: `agents/task_planner.md` | **Log**: `./agent_logs/task_planner_log.md`

Enriches `/tickets` with codebase context and dependency info, producing developer-ready tasks in `/developer_tasks`. Suggests refactors via the forum.

### Developer
**File**: `agents/developer.md` | **Log**: `./agent_logs/developer_log.md`

Implements `/developer_tasks` (respecting dependency order), writes unit tests, maintains code organization rules. Moves completed tasks to `/qa_tasks`.

### QA
**File**: `agents/qa.md` | **Log**: `./agent_logs/qa_log.md`

Interactive agent. Walks the user through QA steps from `/qa_tasks`. Pass moves to `/completed_tasks`. Fail annotates and returns to `/developer_tasks`.

## Pipeline

```
Designer ──> /design_updates ──> Product Analyst ──> /design_updates_archive
                                       │
                                 /feature_updates ──> Project Manager ──> /feature_updates_archive
                                                            │
                                                      /tickets ──> Task Planner ──> /tickets_archive
                                                                         │
                                                                   /developer_tasks ──> Developer ──> /qa_tasks
                                                                                                         │
                                                                                                      QA ──> /completed_tasks
```

Each pipeline directory is a queue. Presence of a file = unprocessed work. After processing, agents move the input file to the corresponding archive directory.

All agents communicate laterally via `/forum`.

## Supervisor (Shell Script)

**Script**: `run_supervisor.sh` (uses `check_pipeline.sh`) | **Log**: `./agent_logs/supervisor_log.md`

Pure shell script orchestrator — not an agent. Runs on a 30s loop. `check_pipeline.sh` checks pipeline directories for files — presence means work is pending. The supervisor script then archives closed forum topics and launches agents as needed. Does not schedule interactive sessions (QA, designer, operator — those are user-initiated). Designer and QA are launched in forum-only mode when they have pending forum votes.

## Utility Agents

These agents do not count for forum close-votes but can read forum topics.

### Operator
**File**: `agents/operator.md` | **Log**: `./agent_logs/operator_log.md`

Interactive agent. Gives the user an organizational view of the project. Can create forum topics on behalf of the user to steer project direction.

# Technical Stack

- **Language**: Rust (Edition 2021)
- **Game Engine**: Bevy 0.17
- **Build System**: Cargo
