---
name: framework
description: "Shared agent framework defining the execution lifecycle, forum rules, and directory layout used by all agents in the Space Crystals project.\n\nExamples:\n- user: \"How do agents work in this project?\"\n  assistant: \"I'll reference the framework to explain the agent lifecycle and conventions.\""
model: opus
memory: project
---

# Agent Framework

All agents follow this execution lifecycle. No exceptions.

## Execution Lifecycle

1. **Load context**: Agent file + agent insights file (always)
2. **PRUNE check**: If the user's prompt is `PRUNE`, skip ALL other steps and execute the PRUNE command (see below). This overrides everything.
3. **Forum pass**: Check `/forum` for active topics
4. **Agent-specific work**: Check agent's task sources, execute one task
5. **Write outputs**: Log entry, insights updates if needed, forum topics if blocked, task-specific outputs
6. **Die**: Agent is terminated. No state carries over except through files.

Each execution is stateless. The insights file is the only continuity between executions.

## Forum Rules

The forum is the highest-priority task source for every agent. It is a directory (`./forum/`) where each topic is a markdown file.

### Topic File Format

```markdown
# Close Votes
- [agent_name]
- [agent_name]

# Topic: [Title]

**Opened by**: [agent_name]
**Status**: open

## [agent_name] (original)
[Content of the topic]

## [agent_name] (reply)
[Reply content]
```

### Forum Behavior Per Execution

- An agent reads all active (non-closed) topics.
- An agent may **vote to close** any number of topics in a single execution.
- An agent may **reply to at most one topic** per execution.
- Voting to close topics + replying to one topic together counts as a single task.
- An agent **cannot vote to close a topic if it was the last commenter**, unless every other agent has already voted to close it.
- A topic is **closed** when every voting agent type (designer, product_analyst, project_manager, task_planner, developer, qa) has voted to close it. The `check_pipeline.sh` script archives closed topics to `/forum_archive` automatically.

### Voting Format (CRITICAL — READ CAREFULLY)

When voting to close, add your agent name as a NEW BULLET LINE in the `# Close Votes` section at the **top** of the topic file. Every vote MUST be on its own separate line.

**Correct example** — if designer and developer have already voted and you are task_planner:

```
# Close Votes
- designer
- developer
- task_planner
```

Each vote is on its OWN LINE: `- ` followed by the agent name. One vote per line. Never combine multiple votes on a single line.

The `# Close Votes` heading must be the FIRST line of the file. If it doesn't exist, create it as the first line before adding your vote.

Do NOT put votes anywhere else in the file — not in metadata headers, not inline in your reply text, nowhere else.

**WRONG — these are all INVISIBLE to the parser and will cause you to be relaunched forever**:
- `designer, product_analyst, task_planner` (comma-separated — WRONG, each name must be on its own line)
- `- designer, product_analyst` (multiple names on one bullet — WRONG, one name per line)
- `**Votes-to-close**: designer, product_analyst` (metadata header — WRONG, use `# Close Votes` with bullet lines)
- `Vote to close.` (writing this in your reply is a comment, NOT a vote — you must also add a `- your_name` line to `# Close Votes`)
- `[designer]` (bracketed — WRONG, no brackets)

The automated pipeline parser (`check_pipeline.sh`) ONLY recognizes individual `- agent_name` bullet lines under `# Close Votes`. Everything else is silently ignored.

### When to Open a Topic

An agent opens a forum topic when:
- Its current task is ambiguous or uncompletable
- It has concerns about upstream work it's processing
- It wants to suggest changes (e.g., refactors, process improvements)

When opening a topic, the agent logs that its current task is blocked until the topic is resolved.

## Insights File

Every agent has an insights file (`agent_logs/{agent}_insights.md`) that is **always loaded** as context. This file contains:

- Distilled, generally applicable knowledge learned across many sessions
- Structured reference data the agent needs every run (e.g., TOC, pending items)
- Process notes and tracking changes

Keep this file concise and high-value. Update it when you learn something new that will be useful in future runs. Remove entries that become outdated.

## Log File

Every agent has a session log file (`agent_logs/{agent}_log.md`). This file is **not loaded automatically** — agents read it only when they need to look up specific context from past sessions. After every execution, the agent appends:

1. **Task summary**: Brief description of what was accomplished
2. **Useful findings**: Information that would have been helpful to know before starting the task
3. **Blocked tasks**: If a task is blocked, note which forum topic it's waiting on

Keep entries concise. The log exists for debugging and historical reference, not as the agent's working memory.

## PRUNE Command

When the user's prompt contains the word `PRUNE`, the agent must prune its insights and log files instead of doing normal work. This takes priority over all other work including forum passes.

### Procedure

1. **Review the insights file** (`agent_logs/{agent}_insights.md`): Read through all entries. Remove anything outdated, no longer relevant, or superseded. Consolidate redundant entries. The goal is to keep the insights file lean and high-value.
2. **Truncate the log file** (`agent_logs/{agent}_log.md`): Archive the full log to `agent_logs/archived/{agent}_log_archive_<N>.md` (N = next archive number). Replace the log with just the last 5 entries. Before archiving, scan for any insights worth extracting and add them to the insights file.
3. **Log**: Append a single entry to the new log noting the prune occurred and which archive file was created.
4. **Die**.

## Interactive Agents

Some agents (designer, QA) have interactive sessions with the user. These agents **cannot enter their interactive session until all forum topics are either replied to or voted to close**. If forum work remains, the agent does its forum pass and dies — the interactive session happens in a future execution when the forum is clear.

## File Mutability

Files in `/design_updates` and `/feature_updates` are **single-write only**. They are never modified or appended after creation.

## Directory Overview

| Directory | Owner | Purpose |
|-----------|-------|---------|
| `/forum` | All agents | Cross-agent communication |
| `/design` | Designer | Design documents |
| `/design_updates` | Designer | Single-write summaries of design changes (queue — Product Analyst moves to archive after processing) |
| `/design_updates_archive` | Product Analyst | Processed design updates |
| `/features` | Product Analyst | Feature specifications |
| `/feature_updates` | Product Analyst | Single-write summaries of feature changes (queue — Project Manager moves to archive after processing) |
| `/feature_updates_archive` | Project Manager | Processed feature updates |
| `/tickets` | Project Manager | Work items with QA steps (queue — Task Planner moves to archive after processing) |
| `/tickets_archive` | Task Planner | Processed tickets |
| `/developer_tasks` | Task Planner | Enriched tasks with technical context (queue — Developer moves to `/qa_tasks` after implementation) |
| `/qa_tasks` | Developer/QA | Tasks awaiting QA (queue — QA moves to `/completed_tasks` on pass, back to `/developer_tasks` on fail) |
| `/completed_tasks` | QA | Successfully QA'd tasks |
| `/forum_archive` | `check_pipeline.sh` | Archived closed forum topics |
