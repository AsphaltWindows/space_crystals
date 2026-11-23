# Space Crystals RTS

A real-time strategy game in early development, written in Rust (Edition 2024) with the Bevy 0.20 engine (Cargo workspace).

# Project Structure

- **Design documents (the spec)**: `artifacts/designer/design/**/*.md` — index: `artifacts/designer/design/README.md`. Cross-references are written `path/file.md > Section`.
- **Code**: Cargo workspace at the repo root — `crates/sim` (`sc_sim`: deterministic game simulation, no Bevy dependency) and `crates/game` (`space_crystals`: the Bevy client). Architecture and coding rules: `ARCHITECTURE.md`. `cargo run` starts the game; `cargo test` tests both crates.
- **Old prototype**: deleted when the rebuild started; recoverable from git tag `pipeline-archive` (reference only, diverges from the design).
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
