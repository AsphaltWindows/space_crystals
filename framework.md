# Agent Pipeline Framework

> **Read-only for all agents.** This file is framework-owned and must never be modified by any agent, including the pipeline_builder. Changes to this file are made only by the framework maintainer.

## Overview

A pipeline framework where agents communicate through markdown messages and maintain their own artifact spaces. A shared forum provides cross-pipeline communication with consensus-based resolution.

## Core Concepts

### Pipeline Nodes

The pipeline consists of **nodes** — named units that consume messages, maintain artifacts, and produce messages. There are two kinds of nodes:

- **Agents** — LLM-powered nodes invoked via Claude Code with a system prompt (`.claude/agents/{name}.md`). Agents can participate in forum discussions and vote to close topics.
- **Script nodes** — shell scripts that run without LLM involvement. They have the same pipeline infrastructure (artifacts, message inboxes) but cannot participate in forum discussions. Their `close_vote_required` is always `false`.

### Agents

Three types:

- **Source**: Interactive with user. Produces messages for downstream agents. Has interactive and non-interactive mode.
- **Processing**: Autonomous. Consumes messages, updates own artifacts, produces messages. Has interactive and non-interactive mode.
- **Sink**: Interactive with user. Consumes messages. Has interactive and non-interactive mode.

### Node Definition

Each pipeline node is defined inline in `pipeline.yaml` under the `nodes` section. Each node entry specifies:
- Name (unique identifier, snake_case; also used as directory names)
- Type (source | processing | sink)
- Description/role
- Command — the shell command the scheduler executes to launch the node
- Consumed message types (with priority, lower number = higher priority)
- Produced message types (with target consumers)
- Interactive mode description (agent nodes; source/sink required, processing optional)
- Non-interactive mode description (agent nodes)

Optional flags:
- `scheduled: false` — node is never launched by the scheduler (user-invoked only)
- `close_vote_required: false` — node's vote is not needed to close forum topics

The `command` field is what the scheduler runs to launch the node. It executes from the project root with `NODE_ROOT` and `NODE_NAME` environment variables set. Both agent nodes and script nodes use this same mechanism — the scheduler does not need to know the node type.

Standard command patterns:
- Agent nodes: `claude -p "Run in non-interactive mode" --allowedTools "Read,Write,Edit,Glob,Grep,Bash" --agent .claude/agents/{name}.md`
- Script nodes: `scripts/{name}.sh "$NODE_ROOT" "$NODE_NAME"`

### Script Nodes

A script node is a processing node that runs a shell script instead of an LLM. It participates in the same pipeline infrastructure — it has its own artifacts directory, messages inbox, and is scheduled by `run_scheduler.sh` — but requires no LLM invocation or agent prompt file.

The script is responsible for:
- Checking `messages/{name}/{message-type}/pending/` for pending messages across its consumed types
- Processing messages and moving them through `pending/` → `active/` → `done/`
- Using `scripts/send_message.sh` to send output messages to downstream consumers
- Writing to its own `artifacts/{name}/` directory

Script nodes always have `close_vote_required: false` and `type: processing`.

### Framework Agents

The framework provides two built-in agents that exist **outside the pipeline**. They are not listed in `pipeline.yaml`, are never scheduled, and their close-votes are never required for forum topic closure.

- **Operator**: The user's direct interface into the system. Translates user concerns, questions, and directives into forum topics that pipeline nodes can act on. Defined in `.claude/agents/operator.md`.
- **Pipeline Builder**: The meta-agent responsible for designing and maintaining the pipeline. Creates and modifies pipeline nodes, maintains `pipeline.yaml`, and generates agent prompts and scripts. Defined in `.claude/agents/pipeline_builder.md`.

Framework agents have their own artifacts directories (`artifacts/operator/`, `artifacts/pipeline_builder/`) and can participate in forum discussions, but they are not pipeline nodes and must not appear in `pipeline.yaml`.

### Artifacts

Located in `/artifacts/{agent-name}/`.

- Agent is the **sole writer** to its artifact directory
- All other agents have **read-only** access
- Internal organization is up to the owning agent
- Used to maintain the agent's domain state

### Insights

Each agent maintains an `artifacts/{agent-name}/insights.md` file.

- **Loaded at startup** — the agent reads this file before doing any work
- **Written post-task** — after completing a task that required significant investigation, the agent appends an insight if it discovered something specific that would have helped it find the right path earlier
- Insights should be concise and actionable — not a log of what happened, but a lesson learned
- Agents should consult their insights during work to avoid repeating past mistakes

### Session Log

Each agent maintains a `artifacts/{agent-name}/log.md` file.

- **Written at end of session** — before exiting, the agent appends a timestamped summary of what it did during the session
- **Not loaded at startup** — the agent is aware the log exists and may read it if needed, but does not load it automatically
- Summaries should be brief: what work was found, what actions were taken, what was produced

### Messages

Located in `/messages/{agent-name}/{message-type}/` (organized by **consuming** agent and **message type**).

- Each message is a single `.md` file
- Single producer, single consumer
- Each consumed message type has its own subdirectory with the full lifecycle: `pending/` → `active/` → `done/`
- The scheduler determines work availability by checking for any `pending/` files across all message-type subdirectories
- The agent is responsible for moving messages through the lifecycle (pending → active → done)
- Agents process message types in priority order (lower number = higher priority in their `consumes` list)

Message filename format: `{producing-agent}-{message-name}.md`

The `{message-name}` is a short descriptive slug for the specific message (e.g., `add_reviewer_agent`, `fix_parser_bug`). The message type is encoded in the directory path, not the filename. Timestamps are not needed in the filename — messages are write-only and file creation time serves as the timestamp.

### Message Types

All message types are registered in the `message_types` section of `pipeline.yaml`. Each entry has a name (snake_case), description, and path to a template file. The registry is the canonical list of types — agents' `consumes` and `produces` entries reference these by name. Message type names are used as directory names, so snake_case is required.

### Message Templates

Each message type has a template file in `templates/messages/{message-type}.md`. Templates document the expected content structure for that message type. Agents use `scripts/send_message.sh` to create messages (which handles metadata and file placement); the templates serve as guidance for what content to pass.

### Forum

Located in `/forum/open/` and `/forum/closed/`.

- Any agent can create a topic during task execution
- Used for problems, ambiguities, or cross-pipeline communication
- **Highest priority** for all agents — checked before messages
- A topic is closed only when every pipeline node with `close_vote_required: true` has voted to close it
- Any new comment **clears all existing close-votes**
- Agents should execute work in response to forum topics when the issue falls under their responsibility

Forum topic filename format: `{timestamp}-{creating-agent}-{slug}.md`

### Forum Topic Structure

```markdown
# Topic Title

## Metadata
- **Created by**: {agent-name}
- **Created**: {ISO-8601 timestamp}
- **Status**: open

## Close Votes
<!-- ONE VOTE PER LINE: VOTE:{agent-name} -->

## Discussion

### [{agent-name}] {ISO-8601 timestamp}

{comment content}
```

### Message Structure

```markdown
# {Title}

## Metadata
- **From**: {producing-agent-name}
- **To**: {consuming-agent-name}

## Content

{message content}
```

The message type is determined by the directory the message lives in. The creation timestamp is the file's filesystem creation time.

### No-Work Investigation

When an agent is launched by the scheduler in non-interactive mode but cannot find any work to do, something is wrong — the scheduler only launches agents when it detects pending work.

The agent should:
1. **Investigate** — check its pending messages, open forum topics, and any other expected work sources to understand why the scheduler thought work existed but the agent cannot find it
2. **Attempt low-impact self-unblocking** — if the cause is simple (e.g., a malformed message filename, a message it already processed but didn't move to done), fix it
3. **Escalate if needed** — if the cause is unclear or complex, open a forum topic describing what happened so other agents can help diagnose the issue
4. **Log the incident** — record what happened in the session log regardless of outcome

## Pipeline

Defined in `pipeline.yaml`. This file contains **only pipeline nodes** — framework agents (operator, pipeline_builder) are not listed here. The scheduler reads this to determine:
- Which message types exist in the system
- Which nodes exist and their types
- What each node consumes (and priority)
- What each node produces
- How to route messages

The `message_types` section defines all message types used in the pipeline. Each type has a name and description. This serves as the canonical registry — the `consumes` and `produces` entries on nodes reference these types by name.

The `nodes` section lists all pipeline nodes (agents and script nodes). Each `produces` entry includes a `to` field listing the consuming nodes. This is the authoritative routing table — the producer writes to `messages/{consumer}/{message-type}/pending/` for each node in the `to` list. The `consumes` entries on receiving nodes are the inverse view. The pipeline_builder uses both to create the per-type inbox directories (`messages/{node}/{message-type}/{pending,active,done}/`).

### Pipeline Upgrade

When the framework is upgraded, `pipeline.yaml` may need migration to match the current framework version. The pipeline_builder is responsible for this migration. Key changes to check:
- The `nodes` section replaces the former `agents` section — rename the key if the old name is present
- Framework agents (operator, pipeline_builder) must not appear in `pipeline.yaml` — remove any such entries
- All entries in `nodes` should be pipeline nodes only
- Node definitions are now inline in `pipeline.yaml` — if separate `agent.yaml` or `node.yaml` files exist in `agents/{name}/`, migrate their contents into `pipeline.yaml` and remove the files
- Each node must have a `command` field — the `script:` field is no longer used; migrate script nodes by setting `command: scripts/{name}.sh "$NODE_ROOT" "$NODE_NAME"`

## Scheduler

`scripts/run_scheduler.sh` — long-running process that continuously monitors the pipeline for work.

The scheduler runs in an infinite loop. The interval between passes is configured by `scheduler_interval` in `pipeline.yaml` (in seconds, default 20). Each pass:
1. For each scheduled node, skip if already running (PID lock file check)
2. If `close_vote_required` is not false: check open forum topics — if any topic lacks the node's close-vote, the node has work
3. Check `messages/{node-name}/*/pending/` — if any files exist in any message-type subdirectory, the node has work
4. If work exists, read `command` from the node's entry in `pipeline.yaml` and execute it in the background

Nodes are **never started speculatively**. The scheduler guarantees work exists before launch, but does not tell the node what to do — the node is responsible for finding and processing its own work.

## Scripts

- `scripts/run_scheduler.sh` — orchestrator
- `scripts/send_message.sh <from> <to> <message-type> <message-name> <content>` — creates a message in the recipient's inbox. Validates the message type exists and the recipient consumes it. Agents and script nodes should use this instead of writing message files directly.
- `scripts/add_comment.sh <topic-file> <agent-name> <comment-text>` — appends comment to forum topic, clears all close-votes
- `scripts/vote_close.sh <topic-file> <agent-name>` — adds close-vote to forum topic

## Directory Structure

```
/
├── artifacts/{node-name}/        # node-owned state (agents and script nodes)
├── messages/{node-name}/         # per-consumer inboxes
│   └── {message-type}/          # subdirectory per consumed message type
│       ├── pending/
│       ├── active/
│       └── done/
├── forum/
│   ├── open/
│   └── closed/
├── .claude/agents/
│   └── {agent-name}.md           # agent prompt (Claude Code format, agent nodes only)
├── scripts/
│   ├── run_scheduler.sh
│   ├── send_message.sh
│   ├── add_comment.sh
│   ├── vote_close.sh
│   └── {script-node-name}.sh     # script node entry points
├── templates/
│   └── messages/{message-type}.md  # per-type message templates
├── pipeline.yaml                 # pipeline manifest
└── framework.md                  # this file
```
