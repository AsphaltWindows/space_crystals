# Space Crystals RTS

A real-time strategy game in early development, written in Rust (Edition 2021) with the Bevy 0.17 engine (Cargo build).

# Project Structure

- **Design documents (the spec)**: `artifacts/designer/design/**/*.md` — index: `artifacts/designer/design/README.md`. Cross-references are written `path/file.md > Section`.
- **Old prototype code**: `artifacts/developer/` — built by a retired agent pipeline; the game is being rebuilt from scratch, so treat it as reference only. The new code location is recorded in `artifacts/director/insights.md` once decided.
- **Agent prompts**: `.claude/agents/{designer,director}.md`
- **Per-agent artifacts**: `artifacts/{designer,director}/` — each agent is the sole writer of its own directory.

# Workflow: Designer + Director

Two Claude Code sessions, possibly running at the same time in this checkout:

- **Designer** (`scripts/designer.sh`) — works with the user on the game design, maintains the design docs, and commits its own changes (`artifacts/designer/` only).
- **Director** (`scripts/director.sh`) — implementation lead; plans from the design docs, delegates to sub-agents, reviews, integrates and commits code.

They communicate through files:

| Channel | Written by | Location |
|---|---|---|
| Design change entries | Designer | `artifacts/designer/changes/NNNN-slug.md` |
| Design questions | Director | `artifacts/director/questions/QNNNN-slug.md` |

Answers to questions arrive as change entries marked `Answers: QNNNN`. Each launch script loads a settings file (`.claude/{designer,director}.settings.json`) whose hooks (`scripts/hooks/notify_new.py`) notify that session of new entries at session start, on each user prompt and after tool calls. Each agent records the last entry it has processed in `artifacts/<agent>/state/`.

**Commits**: always use explicit pathspecs (`git commit -- <paths>`) so one agent never commits the other's in-progress work. The Director never commits `artifacts/designer/`.

The previous multi-agent pipeline (forum, messages, scheduler, Telegram bot) was removed; its final state is preserved at git tag `pipeline-archive`.
