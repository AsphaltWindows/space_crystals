# Space Crystals RTS

A real-time strategy game built with Bevy 0.17.

# Project Overview

Space Crystals is an RTS game currently in early development. The project uses the Bevy game engine and is written in Rust.

# Project Structure

- **Source code**: `artifacts/developer/` (Cargo.toml, src/, etc.)
- **Design documents**: `artifacts/designer/design/*.md`
- **Agent framework**: `framework.md`
- **Pipeline manifest**: `pipeline.yaml`
- **Agent definitions**: `agents/{name}/agent.yaml`
- **Agent prompts**: `.claude/agents/{name}.md`
- **Message inboxes**: `messages/{agent}/{message_type}/pending|active|done/`
- **Message templates**: `templates/messages/{message_type}.md`

# Agent Framework

See `framework.md` for the full shared lifecycle, forum rules, and directory layout.

Key concepts:
- **Forum** (`forum/`): Highest-priority task source for all agents. Cross-agent communication via topic files.
- **Artifacts** (`artifacts/{name}/`): Each agent's persistent state. Sole writer, others read-only.
- **Messages**: Agents communicate via markdown messages routed through per-consumer inboxes using `scripts/send_message.sh`.
- **Insights**: Each agent's persistent memory between executions (`artifacts/{name}/insights.md`).
- **Single task per execution**: An agent does at most one unit of work per run, then exits.

# Pipeline

```
Designer (interactive, 1 session -> N feature_requests)
    |
    v feature_request
Task Splitter (1 -> M developer_tasks + forward feature_request + manifest)
    |                          |
    | developer_task           | feature_request + feature_tasks
    v                          v
Task Planner (1:1)      Completion Aggregator (script node)
    | planned_task              ^ task_completion
    v                          |
Developer (1:1) ---------------+
                               | when all done:
                               v qa_item
                        QA Router (script node)
                        |---> Manual QA (interactive sink)
                        +---> Automatic QA (autonomous)
                               |
                               v feature_request (on failure, _r{N})
                        back to Task Splitter
```

## Agents

### Designer (source)
Interactive. Collaborates with user on game design. Produces `feature_request` messages with QA instructions. Design docs live in `artifacts/designer/design/`.

### Task Splitter (processing)
Decomposes `feature_request` into `developer_task` messages. Reads design docs and codebase for informed splits. Forwards original feature_request and a `feature_tasks` manifest to the completion aggregator.

### Task Planner (processing)
Enriches `developer_task` with codebase context (files, patterns, dependencies), producing `planned_task` messages.

### Developer (processing)
Implements `planned_task` messages. Writes code and tests in `artifacts/developer/`. Produces `task_completion` markers.

### Completion Aggregator (script node)
Tracks task completions against feature_tasks manifests. When all tasks for a feature are done, produces a `qa_item`.

### QA Router (script node)
Routes `qa_item` to manual_qa or automatic_qa based on `artifacts/qa_router/auto_capabilities.txt`.

### Manual QA (sink)
Interactive. Walks user through QA steps. On failure, produces a scoped `feature_request` rework back to task_splitter.

### Automatic QA (processing)
Autonomous. Runs automated QA checks. On failure, produces a scoped `feature_request` rework back to task_splitter.

### Operator (source, utility)
Not scheduled. User's direct interface to inject concerns into the pipeline via forum topics. Close-vote not required.

### Pipeline Builder (source, utility)
Maintains the pipeline framework itself. Creates/modifies agent definitions, pipeline configuration, and scripts.

## Scheduler

`scripts/run_scheduler.sh` — runs in a continuous loop (interval configured by `scheduler_interval` in `pipeline.yaml`). Detects work for each node (forum topics or pending messages) and launches them.

# Technical Stack

- **Language**: Rust (Edition 2021)
- **Game Engine**: Bevy 0.17
- **Build System**: Cargo

# currentDate
Today's date is 2026-03-18.
