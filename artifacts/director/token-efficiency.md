# A more token-efficient way of working (adopted 2026-11-28)

The Director proposed this, based on measurements of this session's transcripts. The user adopted it as written: "just implement whatever proposal you write up". New agents had been paused until then.

**The aim** (the user's): "introducing a brand new mechanic into a 300k lines of code project shouldn't [be] that much more expensive than introducing a brand new mechanic into a 30k lines of code project as long as the working practices, architecture and documentation are effectively and economically organized and made available." So an agent should read the mechanic's own module, its system doc and its design doc, never the codebase at large. Every item below serves that aim.

## Where the tokens went (measured)
- **About 16B input tokens in five days.** Agents account for 94% of them (14.9B over 67k calls) and the Director for 6% (1.0B over 4.5k calls). Agents use about 3.5B a day.
- **What multiplies it:**
  - An agent carries about 223k tokens of context into every call. Every M6 slice agent averaged 216–235k and peaked at about 367k, where auto-compaction kicks in.
  - A slice cost 94M–227M input tokens. ST1, for example, made 963 calls at a 235k average: 227M.
  - Input tokens ≈ calls × context, and the context only shrinks at compaction. So the longer an agent runs, the more each of its calls costs.
- **What fills the context:**
  - 80–90% of agents' tool output is code they read.
  - 38% of the source's characters are comments, mostly quotes from the design and notes on the code's history.
  - `ARCHITECTURE.md` is 712 KB, and every brief told agents to read it.
  - Test output is small. Waiting on tests costs clock time, not tokens.

## The proposal
1. **Short agents** (the biggest lever).
   - Each agent does one phase, sized for roughly 150–250 calls, then stops and hands off.
   - Phases come from the mechanic-first split. A mechanic phase touches one sim system with unit tests. An entity phase assembles a unit from existing mechanics: its def, recipe, panel, client, golden and docs.
   - A fresh agent starts at about 30k. Judging by ST1's own curve, a phase this size averages about 100k, so each call should cost less than half of today's.
2. **Precise briefs.** Each brief names the exact files, types and functions to touch, from the previous phase's handoff and the Director's own greps, so the agent doesn't explore.
3. **Lean reading.**
   - Split `ARCHITECTURE.md` into a core of at most 30 KB plus per-system docs read on demand (X3).
   - Agents find lines with `grep -n`, then read only those ranges. No whole files, no logs.
4. **Lighter comments** (adopted). Cite design sections by path; no long quotes and no history. Clean up the existing comments once after M6, so every later read is cheaper.
5. **Tests** (adopted).
   - While working, agents run only the tests for the code they touch.
   - At the end of a phase, they run one gate: fmt, clippy, the workspace tests and the 6 s seed sweep.
   - The integration suites (computer-player games, soaks, perf comparisons) run at milestone boundaries and for pure refactors.
6. **The right model per phase:**
   - Opus for mechanic phases, refactors, audits, and anything tricky (an answer sweep that changes behaviour, for example).
   - Sonnet for entity assembly, doc-only tasks and QA screenshot passes.
   - Fable still only for a problem that resists two serious attempts.
7. **A budget per phase.**
   - Each phase gets a call budget in its brief, for example 250 calls.
   - Every phase is measured with `m6-briefs/tools/ctx_stats.py` against the M6 baseline: 225k average context and 94M–227M input tokens per slice.
   - A phase that runs well over its budget is stopped and re-planned rather than left to grow.
8. **One agent at a time** (adopted). A second runs only when the two share no code. There are no merges of master in mid-flight, and no conflicts.
9. **The Director's own share (6%).** The Director batches its tool calls, and starts a fresh session at milestone boundaries, with `insights.md` and `plan.md` as the handoff.
10. **The design side** (Q0193): shared mechanics defined once in their own `systems/` docs, with unit docs that compose them. One mechanic doc then maps to one module.

## Measurements since adoption
- **SP's first phase** (launched under the old brief, 251 calls): mean context 213k, past 300k within its first 63 calls; 53M input tokens. **GS1** (768 calls): mean 222k, 160M.
  - **What this shows:** shorter phases bound the number of calls, but **what an agent reads in its first 60 calls sets its mean context**. A short phase therefore only gets cheap if its start is lean.
  - **SP's reading:** about 384k tokens of tool output, spread over code files: `orders/run.rs`, `sim/gate/orders.rs`, `orders/resolution.rs`, `movement.rs`, the client's templates and feedback, each 30–60k characters, read whole. `ARCHITECTURE.md` was only about a tenth of it.
- **GS1b** (the first phase briefed under the new rules: GS1's merge with master and its QA, 103 calls): mean context 155k against the M6 baseline of 225k; 15.9M input tokens. It read lean, about 54k tokens of tool output in all. Its own output was the larger part of its context: 109k tokens of reasoning and the merge scripts it wrote.
- **X3** (Sonnet, the `ARCHITECTURE.md` split, 53 calls): 5.1M input tokens, mean context 97k, about 30k tokens of tool output. It scripted the split and the 180 reference rewrites, so its context stayed small. The old file was 727 KB, about 180k tokens: every agent before m6-common's lean rule read it whole at the start, which explains most of the large first contexts.
- **X4** (Opus, the read-only mechanics audit, 125 calls, plus two read-only helpers not counted here): 19.2M input tokens, mean context 154k, about 77k tokens of tool output.
- **A1** (Opus, three small design answers, 207 calls against a budget of 200): 43.6M input tokens, mean context 211k, max 367k, one compaction. It read lean, about 104k tokens of tool output in all. Its own output was 177k tokens, much of it 82 Edit calls that re-pointed citations and marks. Two lessons:
  - **Cost grows with the square of a phase's length,** because every call carries all the earlier ones. Two phases of 100 calls cost about half of one of 200. Budgets are now about 120 calls, a hard limit, and the large refactors are split into a data phase and a behaviour phase.
  - **Mechanical sweeps go in one command,** a `sed -i` per pattern, never one Edit per site (m6-common's new rule). Where a sweep needs no judgement, the Director runs it with a script before the phase starts.
- **A2** (Opus, 105 calls): 17.7M input tokens, mean context 169k, output 130k. A brief that misread the design cost a phase: see insights, 2026-11-28. A wrong reading is the most expensive mistake a brief can carry, because the phase does all its work on it.
- **A2b** (Opus, 96 calls): 12.5M, mean context 130k. **A2c** (Opus, 63 calls of a budget of 80): 6.9M, mean context 110k. A short phase with a narrow brief stays cheap: A2c's cost per call was two thirds of A2's.
- **A3** (Sonnet, tests only, 80 calls, counting a 13-call follow-up): 8.6M, mean context 108k.
- **B1** (Opus, a bug, 82 calls): 7.5M, mean context 91k. A bug phase with a narrow brief and one quick repro stays well under the slices' costs.
- **SP2a** (Opus, a mechanic, 147 calls against a budget of 110): 24.5M, mean context 167k, peak 282k. It finished its whole scope instead of stopping at 80% and handing off; its last 40 calls ran at 250k+ of context. Had it stopped at 110 and handed off to a fresh agent, the phase would have cost about 17M.
- **Budgets are enforced from now on** (2026-11-28, after SP2a): the brief's "hard limit" alone doesn't stop an agent. On each launch the Director arms `m6-briefs/tools/watch_budget.sh <agent> <80% of the budget>` as a background command. When it fires, the Director tells the agent to finish at a green point, commit, and hand off the rest. The wake-up and the message cost the Director about two calls, against several million tokens for an overrun. The Director stops the watcher with TaskStop if the agent finishes first.
- **B2** (Opus, a bug, 104 calls, launched before the watcher): 14.9M, mean context 144k. **SP2b** (Opus, a small mechanic, 61 calls, the first phase watched): 7.0M, mean context 114k. The watcher's message at 49 calls brought it in at 61.
- **SP2c** (Sonnet, an entity phase with QA, 79 calls of 120): 9.8M, mean context 124k. M6-SP's phase 2 cost 41.3M in all, over three phases (24.5M, 7.0M, 9.8M), against 94M–227M for a whole slice before the new way of working.
- **R1a** (Opus, a refactor, 119 calls of 120): 18.2M, mean context 153k. Its watcher fired at 100 calls (it checks once a minute), and R1a spent the rest on fixes without finishing its gate, so the Director ran it. Briefs now tell agents to run the full gate before the 80% mark, not after.
- **SLa** (Opus, a mechanic, 102 calls of 100, four of them a commit asked for afterwards): 13.5M, mean context 132k. It ran its gate before the end, but its last call, a chained commit, was refused as too complex to verify. Launch prompts now have agents commit with simple commands (`git add -- <paths>`, then `git commit -F <file>`).
- **R1b** (Opus, a mechanic and refactor, 85 calls of 120): 12.6M, mean context 149k. It ran its full gate with `--no-fail-fast` before finishing, and handed back green.
- **SLb** (Opus, a small mechanic fix, 44 calls of 50): 4.3M, mean context 97k. The SL chain's two mechanic phases cost 17.8M together.
- **The Director's own calls** average about 228k of context (2026-11-28: 501 calls, 114M input tokens); its share of all input tokens is 7%. So a mechanical job of more than about five calls is cheaper as a small Sonnet agent with a precise brief and the sweep tools, at about 50k a call, than done by the Director. The citation sweeps took the Director about 15 calls (about 3.4M); a small agent would have needed about 0.7M.
- **Hub files.** Of 257 slice commits, about 20 central files were edited by 17–37 slices each: `invariants.rs`, `defs.rs`, the panel templates, `sim.rs`, `phase.rs`, `orders/run.rs`, `resolution.rs`, `events.rs`, `feedback.rs`, `gate/orders.rs`, and others. Each is 300–600 lines of long, comment-heavy lines, and every slice reads them whole.
  - That is how a 300k-line project ends up costing more per mechanic than a 30k-line one: a new mechanic is wired through the same central matches and lists.
  - **X4 now also proposes** how a mechanic can register in one place (one module per order kind or ability, behind a trait or a table). The design side matches it: `systems/mechanics/` has one doc per mechanic, with archetypes (0471, 0472).
- **The cause of the large starts** (found 2026-11-28): m6-common replaced only two items of m5-common's "Read first". So every M6 agent was still told to read all of `ARCHITECTURE.md` ("It is binding"), parts of the M5 plan, and the M3 and M4 plans' technical decisions. With `m6-plan.md`'s §A, §C and §D and about 23 question files on top, that is over 200k tokens before the first line of code, which matches the measured starts. **m6-common's "Read first" now replaces all of it:** read what your brief names, found by grep.
- **Levers, in order:**
  1. lean starts: precise briefs naming the exact functions, and no reading lists;
  2. the hub refactor (from X4);
  3. X3;
  4. the comment cleanup, since comments are 38% of the characters read.

## When agents resume
1. **X3,** the `ARCHITECTURE.md` split. It runs alone, and mostly by script.
2. **X4,** the read-only audit of mechanics built for one unit, with a context budget.
3. **The rest of GS1 and SP,** from their handoffs, as short phases.
4. **The refactors X4 puts first,** then M6's remaining slices as mechanic and entity phases, in chain order.

## Settled
1. **Agents resume under this way of working.** X3 runs alone once GS1 and SP have handed back; the X4 audit, which is read-only, may run beside it.
2. **Models:** Opus or Sonnet by phase, as in item 6.
3. **The call budget:** 250 calls per phase, given in each brief. An agent that reaches it stops at a green, committed point and hands off.
